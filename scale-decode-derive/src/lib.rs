// Copyright (C) 2023 Parity Technologies (UK) Ltd. (admin@parity.io)
// This file is a part of the scale-decode crate.
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

extern crate alloc;

use alloc::string::ToString;
use darling::FromAttributes;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, DeriveInput};

const ATTR_NAME: &str = "decode_as_type";

// Macro docs in main crate; don't add any docs here.
#[proc_macro_derive(DecodeAsType, attributes(decode_as_type, codec))]
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
    let visibility = &input.vis;
    // what type is the derive macro declared on?
    match &input.data {
        syn::Data::Enum(details) => generate_enum_impl(attrs, visibility, &input, details),
        syn::Data::Struct(details) => generate_struct_impl(attrs, visibility, &input, details),
        syn::Data::Union(_) => syn::Error::new(
            input.ident.span(),
            "Unions are not supported by the DecodeAsType macro",
        )
        .into_compile_error(),
    }
}

fn generate_enum_impl(
    attrs: TopLevelAttrs,
    visibility: &syn::Visibility,
    input: &DeriveInput,
    details: &syn::DataEnum,
) -> TokenStream2 {
    let path_to_scale_decode = &attrs.crate_path;
    let path_to_type: syn::Path = input.ident.clone().into();
    let variant_names = details.variants.iter().map(|v| v.ident.to_string());

    let generic_types = handle_generics(&attrs, input.generics.clone());
    let ty_generics = generic_types.ty_generics();
    let impl_generics = generic_types.impl_generics();
    let visitor_where_clause = generic_types.visitor_where_clause();
    let visitor_ty_generics = generic_types.visitor_ty_generics();
    let visitor_impl_generics = generic_types.visitor_impl_generics();
    let visitor_phantomdata_type = generic_types.visitor_phantomdata_type();
    let type_resolver_ident = generic_types.type_resolver_ident();

    // determine what the body of our visitor functions will be based on the type of enum fields
    // that we're trying to generate output for.
    let variant_ifs = details.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let variant_name = variant_ident.to_string();

        let visit_one_variant_body = match &variant.fields {
            syn::Fields::Named(fields) => {
                let (
                    field_count,
                    field_composite_keyvals,
                    field_tuple_keyvals
                ) = named_field_keyvals(path_to_scale_decode, fields);

                quote!{
                    let fields = value.fields();
                    return if fields.has_unnamed_fields() {
                        if fields.remaining() != #field_count {
                            return Err(#path_to_scale_decode::Error::new(#path_to_scale_decode::error::ErrorKind::WrongLength {
                                actual_len: fields.remaining(),
                                expected_len: #field_count
                            }));
                        }
                        let vals = fields;
                        Ok(#path_to_type::#variant_ident { #(#field_tuple_keyvals),* })
                    } else {
                        let vals: #path_to_scale_decode::BTreeMap<Option<&str>, _> = fields
                            .map(|res| res.map(|item| (item.name(), item)))
                            .collect::<Result<_, _>>()?;
                        Ok(#path_to_type::#variant_ident { #(#field_composite_keyvals),* })
                    }
                }
            },
            syn::Fields::Unnamed(fields) => {
                let (
                    field_count,
                    field_vals
                ) = unnamed_field_vals(path_to_scale_decode, fields);

                quote!{
                    let fields = value.fields();
                    if fields.remaining() != #field_count {
                        return Err(#path_to_scale_decode::Error::new(#path_to_scale_decode::error::ErrorKind::WrongLength {
                            actual_len: fields.remaining(),
                            expected_len: #field_count
                        }));
                    }
                    let vals = fields;
                    return Ok(#path_to_type::#variant_ident ( #(#field_vals),* ))
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
        const _: () = {
            #visibility struct Visitor #visitor_impl_generics (
                ::core::marker::PhantomData<#visitor_phantomdata_type>
            );

            use #path_to_scale_decode::vec;
            use #path_to_scale_decode::ToString;

            impl #impl_generics #path_to_scale_decode::IntoVisitor for #path_to_type #ty_generics #visitor_where_clause {
                type AnyVisitor<#type_resolver_ident: #path_to_scale_decode::TypeResolver> = Visitor #visitor_ty_generics;
                fn into_visitor<#type_resolver_ident: #path_to_scale_decode::TypeResolver>() -> Self::AnyVisitor<#type_resolver_ident> {
                    Visitor(::core::marker::PhantomData)
                }
            }

            impl #visitor_impl_generics #path_to_scale_decode::Visitor for Visitor #visitor_ty_generics #visitor_where_clause {
                type Error = #path_to_scale_decode::Error;
                type Value<'scale, 'info> = #path_to_type #ty_generics;
                type TypeResolver = #type_resolver_ident;

                fn visit_variant<'scale, 'info>(
                    self,
                    value: &mut #path_to_scale_decode::visitor::types::Variant<'scale, 'info, Self::TypeResolver>,
                    type_id: &<Self::TypeResolver as #path_to_scale_decode::TypeResolver>::TypeId,
                ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
                    #(
                        #variant_ifs
                    )*
                    Err(#path_to_scale_decode::Error::new(#path_to_scale_decode::error::ErrorKind::CannotFindVariant {
                        got: value.name().to_string(),
                        expected: vec![#(#variant_names),*]
                    }))
                }
                // Allow an enum to be decoded through nested 1-field composites and tuples:
                fn visit_composite<'scale, 'info>(
                    self,
                    value: &mut #path_to_scale_decode::visitor::types::Composite<'scale, 'info, Self::TypeResolver>,
                    _type_id: &<Self::TypeResolver as #path_to_scale_decode::TypeResolver>::TypeId,
                ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
                    if value.remaining() != 1 {
                        return self.visit_unexpected(#path_to_scale_decode::visitor::Unexpected::Composite);
                    }
                    value.decode_item(self).unwrap()
                }
                fn visit_tuple<'scale, 'info>(
                    self,
                    value: &mut #path_to_scale_decode::visitor::types::Tuple<'scale, 'info, Self::TypeResolver>,
                    _type_id: &<Self::TypeResolver as #path_to_scale_decode::TypeResolver>::TypeId,
                ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
                    if value.remaining() != 1 {
                        return self.visit_unexpected(#path_to_scale_decode::visitor::Unexpected::Tuple);
                    }
                    value.decode_item(self).unwrap()
                }
            }
        };
    )
}

fn generate_struct_impl(
    attrs: TopLevelAttrs,
    visibility: &syn::Visibility,
    input: &DeriveInput,
    details: &syn::DataStruct,
) -> TokenStream2 {
    let path_to_scale_decode = &attrs.crate_path;
    let path_to_type: syn::Path = input.ident.clone().into();

    let generic_types = handle_generics(&attrs, input.generics.clone());
    let ty_generics = generic_types.ty_generics();
    let impl_generics = generic_types.impl_generics();
    let visitor_where_clause = generic_types.visitor_where_clause();
    let visitor_ty_generics = generic_types.visitor_ty_generics();
    let visitor_impl_generics = generic_types.visitor_impl_generics();
    let visitor_phantomdata_type = generic_types.visitor_phantomdata_type();
    let type_resolver_ident = generic_types.type_resolver_ident();

    // determine what the body of our visitor functions will be based on the type of struct
    // that we're trying to generate output for.
    let (visit_composite_body, visit_tuple_body) = match &details.fields {
        syn::Fields::Named(fields) => {
            let (field_count, field_composite_keyvals, field_tuple_keyvals) =
                named_field_keyvals(path_to_scale_decode, fields);

            (
                quote! {
                    if value.has_unnamed_fields() {
                       return self.visit_tuple(&mut value.as_tuple(), type_id)
                    }

                    let vals: #path_to_scale_decode::BTreeMap<Option<&str>, _> =
                        value.map(|res| res.map(|item| (item.name(), item))).collect::<Result<_, _>>()?;

                    Ok(#path_to_type { #(#field_composite_keyvals),* })
                },
                quote! {
                    if value.remaining() != #field_count {
                        return Err(#path_to_scale_decode::Error::new(#path_to_scale_decode::error::ErrorKind::WrongLength { actual_len: value.remaining(), expected_len: #field_count }));
                    }

                    let vals = value;

                    Ok(#path_to_type { #(#field_tuple_keyvals),* })
                },
            )
        }
        syn::Fields::Unnamed(fields) => {
            let (field_count, field_vals) = unnamed_field_vals(path_to_scale_decode, fields);

            (
                quote! {
                    self.visit_tuple(&mut value.as_tuple(), type_id)
                },
                quote! {
                    if value.remaining() != #field_count {
                        return Err(#path_to_scale_decode::Error::new(#path_to_scale_decode::error::ErrorKind::WrongLength { actual_len: value.remaining(), expected_len: #field_count }));
                    }

                    let vals = value;

                    Ok(#path_to_type ( #( #field_vals ),* ))
                },
            )
        }
        syn::Fields::Unit => (
            quote! {
                self.visit_tuple(&mut value.as_tuple(), type_id)
            },
            quote! {
                if value.remaining() > 0 {
                    return Err(#path_to_scale_decode::Error::new(#path_to_scale_decode::error::ErrorKind::WrongLength { actual_len: value.remaining(), expected_len: 0 }));
                }
                Ok(#path_to_type)
            },
        ),
    };

    quote!(
        const _: () = {
            #visibility struct Visitor #visitor_impl_generics (
                ::core::marker::PhantomData<#visitor_phantomdata_type>
            );

            use #path_to_scale_decode::vec;
            use #path_to_scale_decode::ToString;

            impl #impl_generics #path_to_scale_decode::IntoVisitor for #path_to_type #ty_generics #visitor_where_clause {
                type AnyVisitor<#type_resolver_ident: #path_to_scale_decode::TypeResolver> = Visitor #visitor_ty_generics;
                fn into_visitor<#type_resolver_ident: #path_to_scale_decode::TypeResolver>() -> Self::AnyVisitor<#type_resolver_ident> {
                    Visitor(::core::marker::PhantomData)
                }
            }

            impl #visitor_impl_generics #path_to_scale_decode::Visitor for Visitor #visitor_ty_generics #visitor_where_clause {
                type Error = #path_to_scale_decode::Error;
                type Value<'scale, 'info> = #path_to_type #ty_generics;
                type TypeResolver = #type_resolver_ident;

                fn visit_composite<'scale, 'info>(
                    self,
                    value: &mut #path_to_scale_decode::visitor::types::Composite<'scale, 'info, Self::TypeResolver>,
                    type_id: &<Self::TypeResolver as #path_to_scale_decode::TypeResolver>::TypeId,
                ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
                    #visit_composite_body
                }
                fn visit_tuple<'scale, 'info>(
                    self,
                    value: &mut #path_to_scale_decode::visitor::types::Tuple<'scale, 'info, Self::TypeResolver>,
                    type_id: &<Self::TypeResolver as #path_to_scale_decode::TypeResolver>::TypeId,
                ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
                    #visit_tuple_body
                }
            }

            impl #impl_generics #path_to_scale_decode::DecodeAsFields for #path_to_type #ty_generics #visitor_where_clause  {
                fn decode_as_fields<'info, R: #path_to_scale_decode::TypeResolver>(
                    input: &mut &[u8],
                    fields: &mut dyn #path_to_scale_decode::FieldIter<'info, R::TypeId>,
                    types: &'info R
                ) -> Result<Self, #path_to_scale_decode::Error>
                {
                    let mut composite = #path_to_scale_decode::visitor::types::Composite::new(input, fields, types, false);
                    use #path_to_scale_decode::{ Visitor, IntoVisitor };
                    let val = <#path_to_type #ty_generics>::into_visitor().visit_composite(&mut composite, &Default::default());

                    // Consume any remaining bytes and update input:
                    composite.skip_decoding()?;
                    *input = composite.bytes_from_undecoded();

                    val.map_err(From::from)
                }
            }
        };
    )
}

// Given some named fields, generate impls like `field_name: get_field_value()` for each field. Do this for the composite and tuple impls.
fn named_field_keyvals<'f>(
    path_to_scale_decode: &'f syn::Path,
    fields: &'f syn::FieldsNamed,
) -> (usize, impl Iterator<Item = TokenStream2> + 'f, impl Iterator<Item = TokenStream2> + 'f) {
    let field_keyval_impls = fields.named.iter().map(move |f| {
        let field_attrs = FieldAttrs::from_attributes(&f.attrs).unwrap_or_default();
        let field_ident = f.ident.as_ref().expect("named field has ident");
        let field_name = field_ident.to_string();
        let skip_field = field_attrs.skip;

        // If a field is skipped, we expect it to have a Default impl to use to populate it instead.
        if skip_field {
            return (
                false,
                quote!(#field_ident: ::core::default::Default::default()),
                quote!(#field_ident: ::core::default::Default::default())
            )
        }

        (
            // Should we use this field (false means we'll not count it):
            true,
            // For turning named fields in scale typeinfo into named fields on struct like type:
            quote!(#field_ident: {
                let val = *vals
                    .get(&Some(#field_name))
                    .ok_or_else(|| #path_to_scale_decode::Error::new(#path_to_scale_decode::error::ErrorKind::CannotFindField { name: #field_name.to_string() }))?;
                val.decode_as_type().map_err(|e| e.at_field(#field_name))?
            }),
            // For turning named fields in scale typeinfo into unnamed fields on tuple like type:
            quote!(#field_ident: {
                let val = vals.next().expect("field count should have been checked already on tuple type; please file a bug report")?;
                val.decode_as_type().map_err(|e| e.at_field(#field_name))?
            })
        )
    });

    // if we skip any fields, we won't expect that field to exist in some tuple that's being given back.
    let field_count = field_keyval_impls.clone().filter(|f| f.0).count();
    let field_composite_keyvals = field_keyval_impls.clone().map(|v| v.1);
    let field_tuple_keyvals = field_keyval_impls.map(|v| v.2);

    (field_count, field_composite_keyvals, field_tuple_keyvals)
}

// Given some unnamed fields, generate impls like `get_field_value()` for each field. Do this for a tuple style impl.
fn unnamed_field_vals<'f>(
    _path_to_scale_decode: &'f syn::Path,
    fields: &'f syn::FieldsUnnamed,
) -> (usize, impl Iterator<Item = TokenStream2> + 'f) {
    let field_val_impls = fields.unnamed.iter().enumerate().map(|(idx, f)| {
        let field_attrs = FieldAttrs::from_attributes(&f.attrs).unwrap_or_default();
        let skip_field = field_attrs.skip;

        // If a field is skipped, we expect it to have a Default impl to use to populate it instead.
        if skip_field {
            return (false, quote!(::core::default::Default::default()));
        }

        (
            // Should we use this field (false means we'll not count it):
            true,
            // For turning unnamed fields in scale typeinfo into unnamed fields on tuple like type:
            quote!({
                let val = vals.next().expect("field count should have been checked already on tuple type; please file a bug report")?;
                val.decode_as_type().map_err(|e| e.at_idx(#idx))?
            }),
        )
    });

    // if we skip any fields, we won't expect that field to exist in some tuple that's being given back.
    let field_count = field_val_impls.clone().filter(|f| f.0).count();
    let field_vals = field_val_impls.map(|v| v.1);

    (field_count, field_vals)
}

fn handle_generics(attrs: &TopLevelAttrs, generics: syn::Generics) -> GenericTypes {
    let path_to_crate = &attrs.crate_path;

    let type_resolver_ident =
        syn::Ident::new(GenericTypes::TYPE_RESOLVER_IDENT_STR, Span::call_site());

    // Where clause to use on Visitor/IntoVisitor
    let visitor_where_clause = {
        let (_, _, where_clause) = generics.split_for_impl();
        let mut where_clause = where_clause.cloned().unwrap_or(syn::parse_quote!(where));
        if let Some(where_predicates) = &attrs.trait_bounds {
            // if custom trait bounds are given, append those to the where clause.
            where_clause.predicates.extend(where_predicates.clone());
        } else {
            // else, append our default bounds to each parameter to ensure that it all lines up with our generated impls and such:
            for param in generics.type_params() {
                let ty = &param.ident;
                where_clause.predicates.push(syn::parse_quote!(#ty: #path_to_crate::IntoVisitor));
            }
        }
        where_clause
    };

    // (A, B, C, ScaleDecodeTypeResolver) style PhantomData type to use in Visitor struct.
    let visitor_phantomdata_type = {
        let tys = generics.params.iter().filter_map::<syn::Type, _>(|p| match p {
            syn::GenericParam::Type(ty) => {
                let ty = &ty.ident;
                Some(syn::parse_quote!(#ty))
            }
            syn::GenericParam::Lifetime(lt) => {
                let lt = &lt.lifetime;
                Some(syn::parse_quote!(& #lt ()))
            }
            // We don't need to mention const's in the PhantomData type.
            syn::GenericParam::Const(_) => None,
        });

        // Add a param for the type resolver generic.
        let tys = tys.chain(core::iter::once(syn::parse_quote!(#type_resolver_ident)));

        syn::parse_quote!( (#( #tys, )*) )
    };

    // generics for our Visitor/IntoVisitor; we just add the type resolver param to the list.
    let visitor_generics = {
        let mut type_generics = generics.clone();
        let type_resolver_generic_param: syn::GenericParam =
            syn::parse_quote!(#type_resolver_ident: #path_to_crate::TypeResolver);

        type_generics.params.push(type_resolver_generic_param);
        type_generics
    };

    // generics for the type itself
    let type_generics = generics;

    GenericTypes {
        type_generics,
        type_resolver_ident,
        visitor_generics,
        visitor_phantomdata_type,
        visitor_where_clause,
    }
}

struct GenericTypes {
    type_resolver_ident: syn::Ident,
    type_generics: syn::Generics,
    visitor_generics: syn::Generics,
    visitor_where_clause: syn::WhereClause,
    visitor_phantomdata_type: syn::Type,
}

impl GenericTypes {
    const TYPE_RESOLVER_IDENT_STR: &'static str = "ScaleDecodeTypeResolver";

    pub fn ty_generics(&self) -> syn::TypeGenerics<'_> {
        let (_, ty_generics, _) = self.type_generics.split_for_impl();
        ty_generics
    }
    pub fn impl_generics(&self) -> syn::ImplGenerics<'_> {
        let (impl_generics, _, _) = self.type_generics.split_for_impl();
        impl_generics
    }
    pub fn visitor_where_clause(&self) -> &syn::WhereClause {
        &self.visitor_where_clause
    }
    pub fn visitor_ty_generics(&self) -> syn::TypeGenerics<'_> {
        let (_, ty_generics, _) = self.visitor_generics.split_for_impl();
        ty_generics
    }
    pub fn visitor_impl_generics(&self) -> syn::ImplGenerics<'_> {
        let (impl_generics, _, _) = self.visitor_generics.split_for_impl();
        impl_generics
    }
    pub fn visitor_phantomdata_type(&self) -> &syn::Type {
        &self.visitor_phantomdata_type
    }
    pub fn type_resolver_ident(&self) -> &syn::Ident {
        &self.type_resolver_ident
    }
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

/// Parse the attributes attached to some field
#[derive(Debug, FromAttributes, Default)]
#[darling(attributes(decode_as_type, codec))]
struct FieldAttrs {
    #[darling(default)]
    skip: bool,
}
