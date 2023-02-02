// Copyright (C) 2023 Parity Technologies (UK) Ltd. (admin@parity.io)
// This file is a part of the scale-value crate.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//         http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{parse_macro_input, punctuated::Punctuated, DeriveInput};

const ATTR_NAME: &str = "decode_as_type";

/// The `DecodeAsType` derive macro can be used to implement `DecodeAsType`
/// on structs and enums whose fields all implement `DecodeAsType`.
///
/// # Example
///
/// ```rust
/// use scale_decode as alt_path;
/// use scale_decode::DecodeAsType;
///
/// #[derive(DecodeAsType)]
/// #[decode_as_type(trait_bounds = "", crate_path = "alt_path")]
/// struct Foo<T> {
///    a: u64,
///    b: bool,
/// }
/// ```
///
/// # Attributes
///
/// - `#[decode_as_type(crate_path = "::path::to::scale_decode")]`:
///   By default, the macro expects `scale_decode` to be a top level dependency,
///   available as `::scale_decode`. If this is not the case, you can provide the
///   crate path here.
/// - `#[decode_as_type(trait_bounds = "T: Foo, U::Input: DecodeAsType")]`:
///   By default, for each generate type parameter, the macro will add trait bounds such
///   that these type parameters must implement `DecodeAsType` too. You can override this
///   behaviour and provide your own trait bounds instead using this option.
#[proc_macro_derive(DecodeAsType, attributes(decode_as_type))]
pub fn derive_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // parse top level attrs.
    let attrs = match TopLevelAttrs::parse(&input.attrs) {
        Ok(attrs) => attrs,
        Err(e) => return e.write_errors().into(),
    };

    derive_with_attrs(attrs, input).into()
}

fn derive_with_attrs(attrs: TopLevelAttrs, input: DeriveInput) -> TokenStream2 {
    // what type is the derive macro declared on?
    match &input.data {
        syn::Data::Enum(details) => generate_enum_impl(attrs, &input, details).into(),
        syn::Data::Struct(details) => generate_struct_impl(attrs, &input, details).into(),
        syn::Data::Union(_) => syn::Error::new(
            input.ident.span(),
            "Unions are not supported by the DecodeAsType macro",
        )
        .into_compile_error()
        .into(),
    }
}

fn generate_enum_impl(
    attrs: TopLevelAttrs,
    input: &DeriveInput,
    details: &syn::DataEnum,
) -> TokenStream2 {
    let path_to_scale_decode = &attrs.crate_path;
    let path_to_type: syn::Path = input.ident.clone().into();
    let (impl_generics, ty_generics, where_clause, phantomdata_type) = handle_generics(&attrs, &input.generics);
    let variant_names = details.variants.iter().map(|v| v.ident.to_string());
    let visitor_struct_name = format_ident!("{}Visitor", input.ident);

    // determine what the body of our visitor functions will be based on the type of enum fields
    // that we're trying to generate output for.
    let variant_ifs = details.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let variant_name = variant_ident.to_string();

        let visit_one_variant_body = match &variant.fields {
            syn::Fields::Named(fields) => {
                let field_ident: Vec<&syn::Ident> = fields.named.iter().map(|f| f.ident.as_ref().expect("named field")).collect();
                let field_name: Vec<String> = field_ident.iter().map(|ident| ident.to_string()).collect();
                let field_count = field_ident.len();

                quote!{
                    let fields = value.fields();
                    return if fields.has_unnamed_fields() {
                        if fields.remaining() != #field_count {
                            return Err(#path_to_scale_decode::Error::new(#path_to_scale_decode::error::ErrorKind::WrongLength {
                                actual: type_id.0,
                                actual_len: fields.remaining(),
                                expected_len: #field_count
                            }));
                        }
                        #( let #field_ident = fields.next().unwrap()?; )*
                        Ok(#path_to_type::#variant_ident { #( #field_ident: #field_ident.decode_as_type().map_err(|e| e.at_field(#field_name))? ),* })
                    } else {
                        let vals: ::std::collections::HashMap<Option<&str>, _> = fields
                            .map(|res| res.map(|item| (item.name(), item)))
                            .collect::<Result<_, _>>()?;
                        #(
                            let #field_ident = *vals
                                .get(&Some(#field_name))
                                .ok_or_else(|| #path_to_scale_decode::Error::new(#path_to_scale_decode::error::ErrorKind::CannotFindField { name: #field_name.to_owned() }))?;
                        )*
                        Ok(#path_to_type::#variant_ident { #( #field_ident: #field_ident.decode_as_type().map_err(|e| e.at_field(#field_name))? ),* })
                    }
                }
            },
            syn::Fields::Unnamed(fields) => {
                let field_idx: Vec<usize> = (0..fields.unnamed.len()).collect();
                let field_ident: Vec<syn::Ident> = field_idx
                    .iter()
                    .map(|n| format_ident!("field_{n}", span = Span::call_site()))
                    .collect();
                let field_count = field_ident.len();

                quote!{
                    let fields = value.fields();
                    if fields.remaining() != #field_count {
                        return Err(#path_to_scale_decode::Error::new(#path_to_scale_decode::error::ErrorKind::WrongLength {
                            actual: type_id.0,
                            actual_len: fields.remaining(),
                            expected_len: #field_count
                        }));
                    }
                    #( let #field_ident = fields.next().unwrap()?; )*
                    return Ok(#path_to_type::#variant_ident (
                        #( #field_ident.decode_as_type().map_err(|e| e.at_idx(#field_idx))? ),*
                    ))
                }
            },
            syn::Fields::Unit => {
                quote!{
                    return Ok(#path_to_type::#variant_ident)
                }
            },
        };

        quote!{
            if value.name() == #variant_name {
                #visit_one_variant_body
            }
        }
    });

    quote!(
        struct #visitor_struct_name #impl_generics (
            ::std::marker::PhantomData<#phantomdata_type>
        );

        impl #impl_generics #path_to_scale_decode::IntoVisitor for #path_to_type #ty_generics #where_clause {
            type Visitor = #visitor_struct_name #ty_generics;
            fn into_visitor() -> Self::Visitor {
                #visitor_struct_name(::std::marker::PhantomData)
            }
        }

        impl #impl_generics #path_to_scale_decode::Visitor for #visitor_struct_name #ty_generics #where_clause {
            type Error = #path_to_scale_decode::Error;
            type Value<'scale> = #path_to_type #ty_generics;

            fn visit_variant<'scale>(
                self,
                value: &mut #path_to_scale_decode::visitor::types::Variant<'scale, '_>,
                type_id: #path_to_scale_decode::visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                #(
                    #variant_ifs
                )*
                Err(#path_to_scale_decode::Error::new(#path_to_scale_decode::error::ErrorKind::CannotFindVariant {
                    got: value.name().to_string(),
                    expected: vec![#(#variant_names),*]
                }))
            }
            // Allow an enum to be decoded through nested 1-field composites and tuples:
            fn visit_composite<'scale>(
                self,
                value: &mut #path_to_scale_decode::visitor::types::Composite<'scale, '_>,
                _type_id: #path_to_scale_decode::visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                if value.remaining() != 1 {
                    return self.visit_unexpected(#path_to_scale_decode::visitor::Unexpected::Composite);
                }
                value.decode_item(self).unwrap()
            }
            fn visit_tuple<'scale>(
                self,
                value: &mut #path_to_scale_decode::visitor::types::Tuple<'scale, '_>,
                _type_id: #path_to_scale_decode::visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                if value.remaining() != 1 {
                    return self.visit_unexpected(#path_to_scale_decode::visitor::Unexpected::Tuple);
                }
                value.decode_item(self).unwrap()
            }
        }
    )
}

fn generate_struct_impl(
    attrs: TopLevelAttrs,
    input: &DeriveInput,
    details: &syn::DataStruct,
) -> TokenStream2 {
    let path_to_scale_decode = &attrs.crate_path;
    let path_to_type: syn::Path = input.ident.clone().into();
    let (impl_generics, ty_generics, where_clause, phantomdata_type) = handle_generics(&attrs, &input.generics);
    let visitor_struct_name = format_ident!("{}DecodeAsTypeVisitor", input.ident);

    // determine what the body of our visitor functions will be based on the type of struct
    // that we're trying to generate output for.
    let (visit_composite_body, visit_tuple_body) = match &details.fields {
        syn::Fields::Named(fields) => {
            let field_ident: Vec<&syn::Ident> =
                fields.named.iter().map(|f| f.ident.as_ref().expect("named field")).collect();
            let field_name: Vec<String> =
                field_ident.iter().map(|ident| ident.to_string()).collect();
            let field_count = field_ident.len();

            (
                quote! {
                    if value.has_unnamed_fields() {
                       return self.visit_tuple(&mut value.as_tuple(), type_id)
                    }

                    let vals: ::std::collections::HashMap<Option<&str>, _> =
                        value.map(|res| res.map(|item| (item.name(), item))).collect::<Result<_, _>>()?;

                    #(
                        let #field_ident = *vals
                            .get(&Some(#field_name))
                            .ok_or_else(|| #path_to_scale_decode::Error::new(#path_to_scale_decode::error::ErrorKind::CannotFindField { name: #field_name.to_owned() }))?;
                    )*

                    Ok(#path_to_type { #( #field_ident: #field_ident.decode_as_type().map_err(|e| e.at_field(#field_name))? ),* })
                },
                quote! {
                    if value.remaining() != #field_count {
                        return Err(#path_to_scale_decode::Error::new(#path_to_scale_decode::error::ErrorKind::WrongLength { actual: type_id.0, actual_len: value.remaining(), expected_len: #field_count }));
                    }

                    #(
                        let #field_ident = value.next().unwrap()?;
                    )*

                    Ok(#path_to_type { #( #field_ident: #field_ident.decode_as_type().map_err(|e| e.at_field(#field_name))? ),* })
                },
            )
        }
        syn::Fields::Unnamed(fields) => {
            let field_idx: Vec<usize> = (0..fields.unnamed.len()).collect();
            let field_ident: Vec<syn::Ident> = field_idx
                .iter()
                .map(|n| format_ident!("field_{n}", span = Span::call_site()))
                .collect();
            let field_count = field_ident.len();

            (
                quote! {
                    self.visit_tuple(&mut value.as_tuple(), type_id)
                },
                quote! {
                    if value.remaining() != #field_count {
                        return Err(#path_to_scale_decode::Error::new(#path_to_scale_decode::error::ErrorKind::WrongLength { actual: type_id.0, actual_len: value.remaining(), expected_len: #field_count }));
                    }

                    #(
                        let #field_ident = value.next().unwrap()?;
                    )*

                    Ok(#path_to_type ( #( #field_ident.decode_as_type().map_err(|e| e.at_idx(#field_idx))? ),* ))
                },
            )
        }
        syn::Fields::Unit => (
            quote! {
                self.visit_tuple(&mut value.as_tuple(), type_id)
            },
            quote! {
                if value.remaining() > 0 {
                    return Err(#path_to_scale_decode::Error::new(#path_to_scale_decode::error::ErrorKind::WrongLength { actual: type_id.0, actual_len: value.remaining(), expected_len: 0 }));
                }
                Ok(#path_to_type)
            },
        ),
    };

    quote!(
        struct #visitor_struct_name #impl_generics (
            ::std::marker::PhantomData<#phantomdata_type>
        );

        impl #impl_generics #path_to_scale_decode::IntoVisitor for #path_to_type #ty_generics #where_clause {
            type Visitor = #visitor_struct_name #ty_generics;
            fn into_visitor() -> Self::Visitor {
                #visitor_struct_name(::std::marker::PhantomData)
            }
        }

        impl #impl_generics #path_to_scale_decode::Visitor for #visitor_struct_name #ty_generics #where_clause {
            type Error = #path_to_scale_decode::Error;
            type Value<'scale> = #path_to_type #ty_generics;

            fn visit_composite<'scale>(
                self,
                value: &mut #path_to_scale_decode::visitor::types::Composite<'scale, '_>,
                type_id: #path_to_scale_decode::visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                #visit_composite_body
            }
            fn visit_tuple<'scale>(
                self,
                value: &mut #path_to_scale_decode::visitor::types::Tuple<'scale, '_>,
                type_id: #path_to_scale_decode::visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                #visit_tuple_body
            }
        }
    )
}

fn handle_generics<'a>(
    attrs: &TopLevelAttrs,
    generics: &'a syn::Generics,
) -> (syn::ImplGenerics<'a>, syn::TypeGenerics<'a>, syn::WhereClause, syn::Type) {
    let path_to_crate = &attrs.crate_path;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut where_clause = where_clause.cloned().unwrap_or(syn::parse_quote!(where));

    if let Some(where_predicates) = &attrs.trait_bounds {
        // if custom trait bounds are given, append those to the where clause.
        where_clause.predicates.extend(where_predicates.clone());
    } else {
        // else, append our default bounds to each parameter to ensure that it all lines up with our generated impls and such:
        for param in generics.type_params() {
            let ty = &param.ident;
            where_clause.predicates.push(syn::parse_quote!(#ty: #path_to_crate::IntoVisitor));
            where_clause.predicates.push(syn::parse_quote!(#ty: #path_to_crate::IntoVisitor));
            where_clause.predicates.push(syn::parse_quote!(#path_to_crate::Error: From<<<#ty as #path_to_crate::IntoVisitor>::Visitor as #path_to_crate::Visitor>::Error>));
        }
    }

    // Construct a type to put into PhantomData<$ty>. This takes lifetimes into account too.
    let phantomdata_type: syn::Type = {
        let tys = generics
            .params
            .iter()
            .filter_map::<syn::Type, _>(|p| match p {
                syn::GenericParam::Type(ty) => {
                    let ty = &ty.ident;
                    Some(syn::parse_quote!(#ty))
                },
                syn::GenericParam::Lifetime(lt) => {
                    let lt = &lt.lifetime;
                    // [jsdw]: This is dumb, but for some reason `#lt ()` leads to
                    // an error (seems to output `'a, ()`) whereas this does not:
                    Some(syn::parse_quote!(::std::borrow::Cow<#lt, str>))
                },
                // We don't need to mention const's in the PhantomData type.
                syn::GenericParam::Const(_) => None,
            });
        syn::parse_quote!( (#( #tys, )*) )
    };

    (impl_generics, ty_generics, where_clause, phantomdata_type)
}

struct TopLevelAttrs {
    // path to the scale_decode crate, in case it's not a top level dependency.
    crate_path: syn::Path,
    // allow custom trait bounds to be used instead of the defaults.
    trait_bounds: Option<Punctuated<syn::WherePredicate, syn::Token!(,)>>,
}

impl TopLevelAttrs {
    fn parse(attrs: &[syn::Attribute]) -> darling::Result<Self> {
        use darling::FromMeta;

        #[derive(FromMeta)]
        struct TopLevelAttrsInner {
            #[darling(default)]
            crate_path: Option<syn::Path>,
            #[darling(default)]
            trait_bounds: Option<Punctuated<syn::WherePredicate, syn::Token!(,)>>,
        }

        let mut res =
            TopLevelAttrs { crate_path: syn::parse_quote!(::scale_decode), trait_bounds: None };

        // look at each top level attr. parse any for decode_as_type.
        for attr in attrs {
            if !attr.path.is_ident(ATTR_NAME) {
                continue;
            }
            let meta = attr.parse_meta()?;
            let parsed_attrs = TopLevelAttrsInner::from_meta(&meta)?;

            res.trait_bounds = parsed_attrs.trait_bounds;
            if let Some(crate_path) = parsed_attrs.crate_path {
                res.crate_path = crate_path;
            }
        }

        Ok(res)
    }
}
