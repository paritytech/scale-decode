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

use crate::{
    DecodeAsType,
    Context,
    context,
    error::{
        Error,
        ErrorKind,
    },
    visitor::{
        self,
        Visitor,
        decode_with_visitor,
        types::*
    }
};

pub struct VisitorWithContext<T> {
    pub context: Context,
    _marker: std::marker::PhantomData<T>
}

impl <T> VisitorWithContext<T> {
    pub fn new(context: Context) -> Self {
        Self { context, _marker: std::marker::PhantomData }
    }
}

// Impl DecodeAsType for some VisitorWithContext<$ty>. Ideally we'd impl DecodeAtType on
// all T where a VisitorWithContext<T>: Visitor impl exists, but that would prevent third
// parties from implementing DecodeAsType for fear of conflicting with our impls.
macro_rules! impl_decode_as_type {
    ($ty:ty) => {
        impl DecodeAsType for $ty {
            fn decode_as_type(input: &mut &[u8], type_id: u32, types: &scale_info::PortableRegistry, context: Context) -> Result<Self, Error> {
                decode_with_visitor(input, type_id, types, VisitorWithContext::<$ty>::new(context))
            }
        }
    };
}

impl Visitor for VisitorWithContext<char> {
    type Error = Error;
    type Value<'scale> = char;

    fn visit_char<'scale>(
        self,
        value: char,
        _type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(value)
    }
}
impl_decode_as_type!(char);

impl Visitor for VisitorWithContext<bool> {
    type Error = Error;
    type Value<'scale> = bool;

    fn visit_bool<'scale>(
        self,
        value: bool,
        _type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(value)
    }
}
impl_decode_as_type!(bool);

impl Visitor for VisitorWithContext<String> {
    type Error = Error;
    type Value<'scale> = String;

    fn visit_str<'scale>(
        self,
        value: &mut Str<'scale>,
        _type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        let s = value
            .as_str()
            .map_err(|e| Error::new(self.context, e.into()))?
            .to_owned();
        Ok(s)
    }
}
impl_decode_as_type!(String);

impl <'a, T> DecodeAsType for std::borrow::Cow<'a, T>
where
    VisitorWithContext<T>: for<'b> Visitor<Error = Error, Value<'b> = T>,
    T: Clone,
{
    fn decode_as_type(input: &mut &[u8], type_id: u32, types: &scale_info::PortableRegistry, context: Context) -> Result<Self, Error> {
        decode_with_visitor(input, type_id, types, VisitorWithContext::<T>::new(context))
            .map(std::borrow::Cow::Owned)
    }
}

impl <T> Visitor for VisitorWithContext<Option<T>>
where
    VisitorWithContext<T>: for<'a> Visitor<Error = Error, Value<'a> = T>
{
    type Error = Error;
    type Value<'scale> = Option<T>;

    fn visit_variant<'scale>(
        self,
        value: &mut Variant<'scale, '_>,
        _type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        let ctx = self.context.at(context::Location::variant(value.name().to_string()));

        if value.name() == "Some" && value.fields().remaining() == 1 {
            let (_name, val) = value
                .fields()
                .decode_item_with_name(VisitorWithContext::<T>::new(ctx.clone()))
                .map_err(|e| e.or_context(ctx))?
                .expect("checked for 1 field already so should be ok");
            Ok(Some(val))
        } else if value.name() == "None" && value.fields().remaining() == 0 {
            Ok(None)
        } else {
            Err(Error::new(ctx, ErrorKind::CannotFindVariant {
                got: value.name().to_string(),
                expected: vec!["Some", "None"]
            }))
        }
    }
}
impl <T> DecodeAsType for Option<T>
where
    VisitorWithContext<T>: for<'a> Visitor<Error = Error, Value<'a> = T>
{
    fn decode_as_type(input: &mut &[u8], type_id: u32, types: &scale_info::PortableRegistry, context: Context) -> Result<Self, Error> {
        decode_with_visitor(input, type_id, types, VisitorWithContext::<Option<T>>::new(context))
    }
}

impl <T, E> Visitor for VisitorWithContext<Result<T, E>>
where
    VisitorWithContext<T>: for<'a> Visitor<Error = Error, Value<'a> = T>,
    VisitorWithContext<E>: for<'a> Visitor<Error = Error, Value<'a> = E>,
{
    type Error = Error;
    type Value<'scale> = Result<T, E>;

    fn visit_variant<'scale>(
        self,
        value: &mut Variant<'scale, '_>,
        _type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        let ctx = self.context.at(context::Location::variant(value.name().to_string()));

        if value.name() == "Ok" && value.fields().remaining() == 1 {
            let (_name, val) = value
                .fields()
                .decode_item_with_name(VisitorWithContext::<T>::new(ctx.clone()))
                .map_err(|e| e.or_context(ctx))?
                .expect("checked for 1 field already so should be ok");
            Ok(Ok(val))
        } else if value.name() == "Err" && value.fields().remaining() == 0 {
            let (_name, val) = value
                .fields()
                .decode_item_with_name(VisitorWithContext::<E>::new(ctx.clone()))
                .map_err(|e| e.or_context(ctx))?
                .expect("checked for 1 field already so should be ok");
            Ok(Err(val))
        } else {
            Err(Error::new(ctx, ErrorKind::CannotFindVariant {
                got: value.name().to_string(),
                expected: vec!["Some", "None"]
            }))
        }
    }
}
impl <T, E> DecodeAsType for Result<T, E>
where
    VisitorWithContext<T>: for<'a> Visitor<Error = Error, Value<'a> = T>,
    VisitorWithContext<E>: for<'a> Visitor<Error = Error, Value<'a> = E>,
{
    fn decode_as_type(input: &mut &[u8], type_id: u32, types: &scale_info::PortableRegistry, context: Context) -> Result<Self, Error> {
        decode_with_visitor(input, type_id, types, VisitorWithContext::<Result<T, E>>::new(context))
    }
}

// Impl Visitor/DecodeAsType for all primitive number types
macro_rules! visit_number_fn_impl {
    ($name:ident : $ty:ty) => {
        fn $name<'scale>(
            self,
            value: $ty,
            _type_id: visitor::TypeId,
        ) -> Result<Self::Value<'scale>, Self::Error> {
            let n = value
                .try_into()
                .map_err(|_| Error::new(self.context, ErrorKind::NumberOutOfRange { value: value.to_string() }))?;
            Ok(n)
        }
    }
}
macro_rules! visit_number_impl {
    ($ty:ty) => {
        impl Visitor for VisitorWithContext<$ty> {
            type Error = Error;
            type Value<'scale> = $ty;

            visit_number_fn_impl!(visit_u8: u8);
            visit_number_fn_impl!(visit_u16: u16);
            visit_number_fn_impl!(visit_u32: u32);
            visit_number_fn_impl!(visit_u64: u64);
            visit_number_fn_impl!(visit_u128: u128);
            visit_number_fn_impl!(visit_i8: i8);
            visit_number_fn_impl!(visit_i16: i16);
            visit_number_fn_impl!(visit_i32: i32);
            visit_number_fn_impl!(visit_i64: i64);
            visit_number_fn_impl!(visit_i128: i128);
        }
        impl_decode_as_type!($ty);
    };
}
visit_number_impl!(u8);
visit_number_impl!(u16);
visit_number_impl!(u32);
visit_number_impl!(u64);
visit_number_impl!(u128);
visit_number_impl!(i8);
visit_number_impl!(i16);
visit_number_impl!(i32);
visit_number_impl!(i64);
visit_number_impl!(i128);

macro_rules! count_idents {
    ($t:ident $($rest:ident)*) => {
        1 + count_idents!( $($rest)* )
    };
    () => {
        0
    }
}

// Decode tuple types from any matching type.
macro_rules! impl_decode_tuple {
    ($($t:ident)*) => {
        impl < $($t),* > Visitor for VisitorWithContext<($($t,)*)>
        where $( VisitorWithContext<$t>: for<'a> Visitor<Error = Error, Value<'a> = $t>  ),*
        {
            type Value<'scale> = ($($t,)*);
            type Error = Error;

            fn visit_composite<'scale>(
                self,
                value: &mut Composite<'scale, '_>,
                type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                const EXPECTED_LEN: usize = count_idents!($($t)*);
                if value.remaining() != EXPECTED_LEN {
                    return Err(Error::new(self.context, ErrorKind::WrongLength {
                        actual: type_id.0,
                        actual_len: value.remaining(),
                        expected_len: EXPECTED_LEN
                    }))
                }

                #[allow(unused)]
                let mut idx = 0;

                Ok((
                    $(
                        #[allow(unused_assignments)]
                        {
                            let ctx = self.context.at_idx(idx);
                            idx += 1;
                            value
                                .decode_item(VisitorWithContext::<$t>::new(ctx.clone()))
                                .map_err(|e| e.or_context(ctx))?
                                .expect("length already checked via .remaining()")
                        }
                    ,)*
                ))
            }

            fn visit_tuple<'scale>(
                self,
                value: &mut Tuple<'scale, '_>,
                type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                const EXPECTED_LEN: usize = count_idents!($($t)*);
                if value.remaining() != EXPECTED_LEN {
                    return Err(Error::new(self.context, ErrorKind::WrongLength {
                        actual: type_id.0,
                        actual_len: value.remaining(),
                        expected_len: EXPECTED_LEN
                    }))
                }

                #[allow(unused)]
                let mut idx = 0;

                Ok((
                    $(
                        #[allow(unused_assignments)]
                        {
                            let ctx = self.context.at_idx(idx);
                            idx += 1;
                            value
                                .decode_item(VisitorWithContext::<$t>::new(ctx.clone()))
                                .map_err(|e| e.or_context(ctx))?
                                .expect("length already checked via .remaining()")
                        }
                    ,)*
                ))
            }
        }
        impl < $($t),* > DecodeAsType for ($($t,)*)
        where $( VisitorWithContext<$t>: for<'a> Visitor<Error = Error, Value<'a> = $t>  ),*
        {
            fn decode_as_type(input: &mut &[u8], type_id: u32, types: &scale_info::PortableRegistry, context: Context) -> Result<Self, Error> {
                decode_with_visitor(input, type_id, types, VisitorWithContext::<($($t,)*)>::new(context))
            }
        }
    }
}
impl_decode_tuple!();
impl_decode_tuple!(A);
impl_decode_tuple!(A B);
impl_decode_tuple!(A B C);
impl_decode_tuple!(A B C D);
impl_decode_tuple!(A B C D E);
impl_decode_tuple!(A B C D E F);
impl_decode_tuple!(A B C D E F G);
impl_decode_tuple!(A B C D E F G H);
impl_decode_tuple!(A B C D E F G H I);
impl_decode_tuple!(A B C D E F G H I J);
impl_decode_tuple!(A B C D E F G H I J K);
impl_decode_tuple!(A B C D E F G H I J K L);
impl_decode_tuple!(A B C D E F G H I J K L M);
impl_decode_tuple!(A B C D E F G H I J K L M N);
impl_decode_tuple!(A B C D E F G H I J K L M N O);
impl_decode_tuple!(A B C D E F G H I J K L M N O P);
impl_decode_tuple!(A B C D E F G H I J K L M N O P Q);
impl_decode_tuple!(A B C D E F G H I J K L M N O P Q R);
impl_decode_tuple!(A B C D E F G H I J K L M N O P Q R S);
impl_decode_tuple!(A B C D E F G H I J K L M N O P Q R S T);
// ^ Note: We make sure to support as many as parity-scale-codec's impls do.







