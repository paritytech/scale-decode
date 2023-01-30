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
    context,
    error::{Error, ErrorKind},
    visitor::{
        self, decode_with_visitor, delegate_visitor_fns, types::*, DecodeItemIterator, Visitor,
    },
    Context, DecodeAsType,
};
use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
    NonZeroU32, NonZeroU64, NonZeroU8,
};
use scale_bits::Bits;
use std::ops::{Range, RangeInclusive};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque},
    marker::PhantomData,
};

pub struct VisitorWithContext<T> {
    pub context: Context,
    _marker: std::marker::PhantomData<T>,
}

impl<T> VisitorWithContext<T> {
    pub fn new(context: Context) -> Self {
        Self { context, _marker: std::marker::PhantomData }
    }
}

/// Generate a DecodeAsType impl for basic types that have a corresponding VisitorWithContext
/// that implements Visitor.
macro_rules! impl_decode_as_type {
    ($ty:ident $(< $($lt:lifetime,)* $($param:ident),* >)? $(where $( $where:tt )* )?) => {
        impl $(< $($lt,)* $($param),* >)? DecodeAsType for $ty $(< $($lt,)* $($param),* >)?
        where
            $( $( VisitorWithContext<$param>: for<'b> Visitor<Error = Error, Value<'b> = $param> ,)* )?
            $( $($where)* )?
        {
            fn decode_as_type(input: &mut &[u8], type_id: u32, types: &scale_info::PortableRegistry, context: Context) -> Result<Self, Error> {
                decode_with_visitor(input, type_id, types, VisitorWithContext::<$ty $(< $($lt,)* $($param),* >)? >::new(context))
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
        let s = value.as_str().map_err(|e| Error::new(self.context, e.into()))?.to_owned();
        Ok(s)
    }
}
impl_decode_as_type!(String);

impl Visitor for VisitorWithContext<Bits> {
    type Error = Error;
    type Value<'scale> = Bits;

    fn visit_bitsequence<'scale>(
        self,
        value: &mut BitSequence<'scale>,
        _type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        value
            .decode()?
            .collect::<Result<Bits, _>>()
            .map_err(|e| Error::new(self.context, ErrorKind::VisitorDecodeError(e.into())))
    }
}
impl_decode_as_type!(Bits);

impl<T> Visitor for VisitorWithContext<PhantomData<T>> {
    type Error = Error;
    type Value<'scale> = PhantomData<T>;

    fn visit_tuple<'scale>(
        self,
        value: &mut Tuple<'scale, '_>,
        _type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        if value.remaining() == 0 {
            Ok(PhantomData)
        } else {
            self.visit_unexpected(visitor::Unexpected::Tuple)
        }
    }
}
impl_decode_as_type!(PhantomData<T>);

// Generate impls to encode things based on some other type.
macro_rules! impl_decode_like {
    ($target:ident $(< $($lt:lifetime,)* $($param:ident),* >)? as $source:ty $( [where $($where:tt)*] )?: $mapper:expr) => {
        impl $(< $($lt,)* $($param),* >)? Visitor for VisitorWithContext<$target $(< $($lt,)* $($param),* >)?>
        where
            $( $( VisitorWithContext<$param>: for<'b> Visitor<Error = Error, Value<'b> = $param> ,)* )?
            $( $($where)* )?
        {
            type Error = Error;
            type Value<'scale> = $target $(< $($lt,)* $($param),* >)?;

            delegate_visitor_fns!(
                |this: Self| VisitorWithContext::<$source>::new(this.context),
                |res: $source| Ok($mapper(res))
            );
        }
        impl_decode_as_type!($target $(< $($lt,)* $($param),* >)? $( where $($where)* )?);
    }
}
impl_decode_like!(Arc<T> as T: |res| Arc::new(res));
impl_decode_like!(Rc<T> as T: |res| Rc::new(res));
impl_decode_like!(Box<T> as T: |res| Box::new(res));
impl_decode_like!(Cow<'a, T> as T [where T: Clone]: |res| Cow::Owned(res));
impl_decode_like!(Duration as (u64, u32): |res: (u64,u32)| Duration::from_secs(res.0) + Duration::from_nanos(res.1 as u64));
impl_decode_like!(Range<T> as (T, T): |res: (T,T)| res.0..res.1);
impl_decode_like!(RangeInclusive<T> as (T, T): |res: (T,T)| res.0..=res.1);

macro_rules! impl_decode_seq_via_collect {
    ($ty:ident<$generic:ident> $(where $($where:tt)*)?) => {
        impl <$generic> Visitor for VisitorWithContext<$ty<$generic>>
        where
            VisitorWithContext<$generic>: for<'b> Visitor<Error = Error, Value<'b> = $generic>,
            $( $($where)* )?
        {
            type Value<'scale> = $ty<$generic>;
            type Error = Error;

            fn visit_tuple<'scale>(
                self,
                value: &mut Tuple<'scale, '_>,
                _type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                decode_items_using::<_, T>(value, self.context).collect()
            }
            fn visit_sequence<'scale>(
                self,
                value: &mut Sequence<'scale, '_>,
                _type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                decode_items_using::<_, T>(value, self.context).collect()
            }
            fn visit_array<'scale>(
                self,
                value: &mut Array<'scale, '_>,
                _type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                decode_items_using::<_, T>(value, self.context).collect()
            }
        }
        impl_decode_as_type!($ty < $generic > $( where $($where)* )?);
    }
}
impl_decode_seq_via_collect!(Vec<T>);
impl_decode_seq_via_collect!(VecDeque<T>);
impl_decode_seq_via_collect!(LinkedList<T>);
impl_decode_seq_via_collect!(BinaryHeap<T> where T: Ord);
impl_decode_seq_via_collect!(BTreeSet<T> where T: Ord);

// For arrays of fixed lengths, we decode to a vec first and then try to turn that into the fixed size array.
// Like vecs, we can decode from tuples, sequences or arrays if the types line up ok.
macro_rules! array_method_impl {
    ($this:ident, $value:ident, $type_id:ident, [$t:ident; $n:ident]) => {{
        let val =
            decode_items_using::<_, $t>($value, $this.context).collect::<Result<Vec<$t>, _>>()?;
        let actual_len = val.len();
        let arr = val.try_into().map_err(|_e| {
            Error::new(
                Context::new(),
                ErrorKind::WrongLength { actual: $type_id.0, actual_len, expected_len: $n },
            )
        })?;
        Ok(arr)
    }};
}
impl<const N: usize, T> Visitor for VisitorWithContext<[T; N]>
where
    VisitorWithContext<T>: for<'b> Visitor<Error = Error, Value<'b> = T>,
{
    type Value<'scale> = [T; N];
    type Error = Error;

    fn visit_tuple<'scale>(
        self,
        value: &mut Tuple<'scale, '_>,
        type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        array_method_impl!(self, value, type_id, [T; N])
    }
    fn visit_sequence<'scale>(
        self,
        value: &mut Sequence<'scale, '_>,
        type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        array_method_impl!(self, value, type_id, [T; N])
    }
    fn visit_array<'scale>(
        self,
        value: &mut Array<'scale, '_>,
        type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        array_method_impl!(self, value, type_id, [T; N])
    }
}
impl<const N: usize, T> DecodeAsType for [T; N]
where
    VisitorWithContext<T>: for<'b> Visitor<Error = Error, Value<'b> = T>,
{
    fn decode_as_type(
        input: &mut &[u8],
        type_id: u32,
        types: &scale_info::PortableRegistry,
        context: Context,
    ) -> Result<Self, Error> {
        decode_with_visitor(input, type_id, types, VisitorWithContext::<[T; N]>::new(context))
    }
}

impl<T> Visitor for VisitorWithContext<BTreeMap<String, T>>
where
    VisitorWithContext<T>: for<'a> Visitor<Error = Error, Value<'a> = T>,
{
    type Error = Error;
    type Value<'scale> = BTreeMap<String, T>;

    fn visit_composite<'scale>(
        self,
        value: &mut Composite<'scale, '_>,
        _type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        let mut map = BTreeMap::new();
        loop {
            // Break if no fields left or we hit an unnamed one:
            // We can't `continue` here; must avoid infinite looping.
            let Some(key) = value.next_item_name() else {
                break
            };

            let key = key.to_owned();
            let ctx = self.context.at_field(key.clone());
            let Some(val) = value.decode_item(VisitorWithContext::<T>::new(ctx)) else {
                break
            };

            map.insert(key, val?);
        }
        Ok(map)
    }
}

impl<T> Visitor for VisitorWithContext<Option<T>>
where
    VisitorWithContext<T>: for<'a> Visitor<Error = Error, Value<'a> = T>,
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
                .transpose()
                .map_err(|e| e.or_context(ctx))?
                .expect("checked for 1 field already so should be ok");
            Ok(Some(val))
        } else if value.name() == "None" && value.fields().remaining() == 0 {
            Ok(None)
        } else {
            Err(Error::new(
                ctx,
                ErrorKind::CannotFindVariant {
                    got: value.name().to_string(),
                    expected: vec!["Some", "None"],
                },
            ))
        }
    }
}
impl_decode_as_type!(Option<T>);

impl<T, E> Visitor for VisitorWithContext<Result<T, E>>
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
                .transpose()
                .map_err(|e| e.or_context(ctx))?
                .expect("checked for 1 field already so should be ok");
            Ok(Ok(val))
        } else if value.name() == "Err" && value.fields().remaining() == 1 {
            let (_name, val) = value
                .fields()
                .decode_item_with_name(VisitorWithContext::<E>::new(ctx.clone()))
                .transpose()
                .map_err(|e| e.or_context(ctx))?
                .expect("checked for 1 field already so should be ok");
            Ok(Err(val))
        } else {
            Err(Error::new(
                ctx,
                ErrorKind::CannotFindVariant {
                    got: value.name().to_string(),
                    expected: vec!["Ok", "Err"],
                },
            ))
        }
    }
}
impl_decode_as_type!(Result<T, E>);

// Impl Visitor/DecodeAsType for all primitive number types
macro_rules! visit_number_fn_impl {
    ($name:ident : $ty:ty where |$res:ident| $expr:expr) => {
        fn $name<'scale>(
            self,
            value: $ty,
            _type_id: visitor::TypeId,
        ) -> Result<Self::Value<'scale>, Self::Error> {
            let $res = value;
            let n = $expr.ok_or_else(|| {
                Error::new(self.context, ErrorKind::NumberOutOfRange { value: value.to_string() })
            })?;
            Ok(n)
        }
    };
}
macro_rules! visit_number_impl {
    ($ty:ident where |$res:ident| $expr:expr) => {
        impl Visitor for VisitorWithContext<$ty> {
            type Error = Error;
            type Value<'scale> = $ty;

            visit_number_fn_impl!(visit_u8: u8 where |$res| $expr);
            visit_number_fn_impl!(visit_u16: u16 where |$res| $expr);
            visit_number_fn_impl!(visit_u32: u32 where |$res| $expr);
            visit_number_fn_impl!(visit_u64: u64 where |$res| $expr);
            visit_number_fn_impl!(visit_u128: u128 where |$res| $expr);
            visit_number_fn_impl!(visit_i8: i8 where |$res| $expr);
            visit_number_fn_impl!(visit_i16: i16 where |$res| $expr);
            visit_number_fn_impl!(visit_i32: i32 where |$res| $expr);
            visit_number_fn_impl!(visit_i64: i64 where |$res| $expr);
            visit_number_fn_impl!(visit_i128: i128 where |$res| $expr);
        }
        impl_decode_as_type!($ty);
    };
}
visit_number_impl!(u8 where |res| res.try_into().ok());
visit_number_impl!(u16 where |res| res.try_into().ok());
visit_number_impl!(u32 where |res| res.try_into().ok());
visit_number_impl!(u64 where |res| res.try_into().ok());
visit_number_impl!(u128 where |res| res.try_into().ok());
visit_number_impl!(i8 where |res| res.try_into().ok());
visit_number_impl!(i16 where |res| res.try_into().ok());
visit_number_impl!(i32 where |res| res.try_into().ok());
visit_number_impl!(i64 where |res| res.try_into().ok());
visit_number_impl!(i128 where |res| res.try_into().ok());
visit_number_impl!(NonZeroU8 where |res| res.try_into().ok().and_then(NonZeroU8::new));
visit_number_impl!(NonZeroU16 where |res| res.try_into().ok().and_then(NonZeroU16::new));
visit_number_impl!(NonZeroU32 where |res| res.try_into().ok().and_then(NonZeroU32::new));
visit_number_impl!(NonZeroU64 where |res| res.try_into().ok().and_then(NonZeroU64::new));
visit_number_impl!(NonZeroU128 where |res| res.try_into().ok().and_then(NonZeroU128::new));
visit_number_impl!(NonZeroI8 where |res| res.try_into().ok().and_then(NonZeroI8::new));
visit_number_impl!(NonZeroI16 where |res| res.try_into().ok().and_then(NonZeroI16::new));
visit_number_impl!(NonZeroI32 where |res| res.try_into().ok().and_then(NonZeroI32::new));
visit_number_impl!(NonZeroI64 where |res| res.try_into().ok().and_then(NonZeroI64::new));
visit_number_impl!(NonZeroI128 where |res| res.try_into().ok().and_then(NonZeroI128::new));

macro_rules! count_idents {
    ($t:ident $($rest:ident)*) => {
        1 + count_idents!( $($rest)* )
    };
    () => {
        0
    }
}

// Decode tuple types from any matching type.
macro_rules! tuple_method_impl {
    (($($t:ident,)*), $self:ident, $value:ident, $type_id:ident) => {{
        const EXPECTED_LEN: usize = count_idents!($($t)*);
        if $value.remaining() != EXPECTED_LEN {
            return Err(Error::new($self.context, ErrorKind::WrongLength {
                actual: $type_id.0,
                actual_len: $value.remaining(),
                expected_len: EXPECTED_LEN
            }))
        }

        #[allow(unused)]
        let mut idx = 0;

        Ok((
            $(
                #[allow(unused_assignments)]
                {
                    let ctx = $self.context.at_idx(idx);
                    idx += 1;
                    $value
                        .decode_item(VisitorWithContext::<$t>::new(ctx.clone()))
                        .transpose()
                        .map_err(|e| e.or_context(ctx))?
                        .expect("length already checked via .remaining()")
                }
            ,)*
        ))
    }}
}
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
                tuple_method_impl!(($($t,)*), self, value, type_id)
            }
            fn visit_tuple<'scale>(
                self,
                value: &mut Tuple<'scale, '_>,
                type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                tuple_method_impl!(($($t,)*), self, value, type_id)
            }
            fn visit_sequence<'scale>(
                self,
                value: &mut Sequence<'scale, '_>,
                type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                tuple_method_impl!(($($t,)*), self, value, type_id)
            }
            fn visit_array<'scale>(
                self,
                value: &mut Array<'scale, '_>,
                type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                tuple_method_impl!(($($t,)*), self, value, type_id)
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

/// This takes anything that can decode a stream if items and return an iterator over them.
fn decode_items_using<'a, 'scale, D: DecodeItemIterator<'scale>, T>(
    decoder: &'a mut D,
    context: Context,
) -> impl Iterator<Item = Result<<VisitorWithContext<T> as Visitor>::Value<'scale>, Error>> + 'a
where
    D: DecodeItemIterator<'scale>,
    VisitorWithContext<T>: Visitor<Error = Error, Value<'scale> = T>,
{
    let mut idx = 0;
    std::iter::from_fn(move || {
        let ctx = context.at_idx(idx);
        let item = decoder
            .decode_item(VisitorWithContext::<T>::new(ctx.clone()))
            .map(|res| res.map_err(|e| e.or_context(ctx)));
        idx += 1;
        item
    })
}

#[cfg(test)]
mod test {
    use super::*;

    /// Given a type definition, return type ID and registry representing it.
    fn make_type<T: scale_info::TypeInfo + 'static>() -> (u32, scale_info::PortableRegistry) {
        let m = scale_info::MetaType::new::<T>();
        let mut types = scale_info::Registry::new();
        let id = types.register_type(&m);
        let portable_registry: scale_info::PortableRegistry = types.into();

        (id.id(), portable_registry)
    }

    // For most of our tests, we'll assert that whatever type we encode, we can decode back again to the given type.
    fn assert_encode_decode_to_with<T, A, B>(a: &A, b: &B)
    where
        A: scale_encode::EncodeAsType,
        B: DecodeAsType + PartialEq + std::fmt::Debug,
        T: scale_info::TypeInfo + 'static,
    {
        let (type_id, types) = make_type::<T>();
        let encoded = a
            .encode_as_type(type_id, &types, scale_encode::Context::new())
            .expect("should be able to encode");
        let decoded = B::decode_as_type(&mut &*encoded, type_id, &types, Context::new())
            .expect("should be able to decode");
        assert_eq!(&decoded, b);
    }

    // Normally, the type info we want to use comes along with the type we're encoding.
    fn assert_encode_decode_to<A, B>(a: &A, b: &B)
    where
        A: scale_encode::EncodeAsType + scale_info::TypeInfo + 'static,
        B: DecodeAsType + PartialEq + std::fmt::Debug,
    {
        assert_encode_decode_to_with::<A, A, B>(a, b);
    }

    // Most of the time we'll just make sure that we can encode and decode back to the same type.
    fn assert_encode_decode_with<T, A>(a: &A)
    where
        A: scale_encode::EncodeAsType + DecodeAsType + PartialEq + std::fmt::Debug,
        T: scale_info::TypeInfo + 'static,
    {
        assert_encode_decode_to_with::<T, A, A>(a, a)
    }

    // Most of the time we'll just make sure that we can encode and decode back to the same type.
    fn assert_encode_decode<A>(a: &A)
    where
        A: scale_encode::EncodeAsType
            + scale_info::TypeInfo
            + 'static
            + DecodeAsType
            + PartialEq
            + std::fmt::Debug,
    {
        assert_encode_decode_to(a, a)
    }

    #[test]
    fn decode_primitives() {
        assert_encode_decode(&true);
        assert_encode_decode(&false);
        assert_encode_decode(&"hello".to_string());
    }

    #[test]
    fn decode_pointer_types() {
        assert_encode_decode_to(&true, &Box::new(true));
        assert_encode_decode_to(&true, &Arc::new(true));
        assert_encode_decode_to(&true, &Rc::new(true));
        assert_encode_decode_to(&true, &Cow::Borrowed(&true));
    }

    #[test]
    fn decode_duration() {
        assert_encode_decode_with::<(u64, u32), _>(&Duration::from_millis(12345));
    }

    #[test]
    fn decode_ranges() {
        assert_encode_decode(&(1..10));
        assert_encode_decode(&(1..=10));
    }

    #[test]
    fn decode_basic_numbers() {
        fn decode_all_types(n: u128) {
            assert_encode_decode_to(&n, &(n as u8));
            assert_encode_decode_to(&n, &(n as u16));
            assert_encode_decode_to(&n, &(n as u32));
            assert_encode_decode_to(&n, &(n as u64));
            assert_encode_decode_to(&n, &n);

            assert_encode_decode_to(&n, &(n as i8));
            assert_encode_decode_to(&n, &(n as i16));
            assert_encode_decode_to(&n, &(n as i32));
            assert_encode_decode_to(&n, &(n as i64));
            assert_encode_decode_to(&n, &(n as i128));
        }

        decode_all_types(0);
        decode_all_types(1);
        decode_all_types(127);
    }

    #[test]
    fn decode_sequences() {
        assert_encode_decode_to(&vec![1u8, 2, 3], &[1u8, 2, 3]);
        assert_encode_decode_to(&vec![1u8, 2, 3], &(1u8, 2u8, 3u8));
        assert_encode_decode_to(&vec![1u8, 2, 3], &vec![1u8, 2, 3]);
        assert_encode_decode_to(&vec![1u8, 2, 3], &LinkedList::from_iter([1u8, 2, 3]));
        assert_encode_decode_to(&vec![1u8, 2, 3], &VecDeque::from_iter([1u8, 2, 3]));
        assert_encode_decode_to(&vec![1u8, 2, 3, 2], &BTreeSet::from_iter([1u8, 2, 3, 2]));
        // assert_encode_decode_to(&vec![1u8,2,3], &BinaryHeap::from_iter([1u8,2,3])); // No partialEq for BinaryHeap
    }

    #[test]
    fn decode_tuples() {
        // Decode to the same:
        assert_encode_decode(&(1u8, 2u16, true));
        // Decode to array:
        assert_encode_decode_to(&(1u8, 2u8, 3u8), &[1u8, 2, 3]);
        // Decode to sequence:
        assert_encode_decode_to(&(1u8, 2u8, 3u8), &vec![1u8, 2, 3]);
    }

    #[test]
    fn decode_composites_tu_tuples() {
        #[derive(scale_encode::EncodeAsType, scale_info::TypeInfo)]
        struct Foo {
            hello: bool,
            other: (u8, u32),
        }

        let input = Foo { hello: true, other: (1, 3) };
        // Same:
        assert_encode_decode_to(&input, &(true, (1u8, 3u32)));
        // Different:
        assert_encode_decode_to(&input, &(true, (1u64, 3u64)));
    }

    #[test]
    fn decode_options_and_results() {
        // These are hardcoded so let's make sure they work..
        assert_encode_decode(&Some(123i128));
        assert_encode_decode(&(None as Option<bool>));
        assert_encode_decode(&Ok::<_, bool>(123i128));
        assert_encode_decode(&Err::<bool, _>(123i128));
    }

    #[test]
    fn decode_bits() {
        assert_encode_decode(&Bits::new());
        assert_encode_decode(&Bits::from_iter([true, false, false, true, false]));
    }
}