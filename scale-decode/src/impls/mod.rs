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
    error::{Error, ErrorKind},
    visitor::{self, ext, types::*, DecodeItemIterator, Visitor, VisitorExt},
    IntoVisitor,
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

pub struct BasicVisitor<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> BasicVisitor<T> {
    pub fn new() -> Self {
        BasicVisitor { _marker: std::marker::PhantomData }
    }
}

/// Generate an [`IntoVisitor`] impl for basic types `T` where `BasicVisitor<T>` impls `Visitor`.
macro_rules! impl_into_visitor {
    ($ty:ident $(< $($lt:lifetime,)* $($param:ident),* >)? $(where $( $where:tt )* )?) => {
        impl $(< $($lt,)* $($param),* >)? crate::IntoVisitor for $ty $(< $($lt,)* $($param),* >)?
        where
            BasicVisitor<$ty $(< $($lt,)* $($param),* >)?>: for<'b> Visitor<Error = Error, Value<'b> = Self>,
            $( $($where)* )?
        {
            type Visitor = BasicVisitor<$ty $(< $($lt,)* $($param),* >)?>;
            fn into_visitor() -> Self::Visitor {
                BasicVisitor::new()
            }
        }
    };
}

impl Visitor for BasicVisitor<char> {
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
impl_into_visitor!(char);

impl Visitor for BasicVisitor<bool> {
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
impl_into_visitor!(bool);

impl Visitor for BasicVisitor<String> {
    type Error = Error;
    type Value<'scale> = String;

    fn visit_str<'scale>(
        self,
        value: &mut Str<'scale>,
        _type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        let s = value.as_str()?.to_owned();
        Ok(s)
    }
}
impl_into_visitor!(String);

impl Visitor for BasicVisitor<Bits> {
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
            .map_err(|e| Error::new(ErrorKind::VisitorDecodeError(e.into())))
    }
}
impl_into_visitor!(Bits);

impl<T> Visitor for BasicVisitor<PhantomData<T>> {
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
impl_into_visitor!(PhantomData<T>);

// Generate impls to encode things based on some other type. We do this by implementing
// `IntoVisitor` and using the `AndThen` combinator to map from an existing one to the desired output.
macro_rules! impl_into_visitor_like {
    ($target:ident $(< $($lt:lifetime,)* $($param:ident),* >)? as $source:ty $( [where $($where:tt)*] )?: $mapper:expr) => {
        impl $(< $($lt,)* $($param),* >)? IntoVisitor for $target $(< $($lt,)* $($param),* >)?
        where
            $source: IntoVisitor,
            $( $($where)* )?
        {
            type Visitor = ext::AndThen<
                // The input visitor:
                <$source as IntoVisitor>::Visitor,
                // The function signature to do the result transformation:
                Box<dyn FnOnce(Result<$source, <<$source as IntoVisitor>::Visitor as Visitor>::Error>)
                    -> Result<$target $(< $($lt,)* $($param),* >)?, <<$source as IntoVisitor>::Visitor as Visitor>::Error>>,
                // The output Visitor::Value:
                $target $(< $($lt,)* $($param),* >)?,
                // The output Visitor::Error (same as before):
                <<$source as IntoVisitor>::Visitor as Visitor>::Error
            >;
            fn into_visitor() -> Self::Visitor {
                // We need to box the function because we can't otherwise name it in the associated type.
                let f = |res: Result<$source, <<$source as IntoVisitor>::Visitor as Visitor>::Error>| res.map($mapper);
                <$source as IntoVisitor>::into_visitor().and_then(Box::new(f))
            }
        }
    }
}

impl_into_visitor_like!(Arc<T> as T: |res| Arc::new(res));
impl_into_visitor_like!(Rc<T> as T: |res| Rc::new(res));
impl_into_visitor_like!(Box<T> as T: |res| Box::new(res));
impl_into_visitor_like!(Duration as (u64, u32): |res: (u64,u32)| Duration::from_secs(res.0) + Duration::from_nanos(res.1 as u64));
impl_into_visitor_like!(Range<T> as (T, T): |res: (T,T)| res.0..res.1);
impl_into_visitor_like!(RangeInclusive<T> as (T, T): |res: (T,T)| res.0..=res.1);

// A custom implementation for `Cow` because it's rather tricky; the visitor we want is whatever the
// `ToOwned` value for the Cow is, and Cow's have specific constraints, too.
type CowVisitor<T> = <<T as ToOwned>::Owned as IntoVisitor>::Visitor;
type CowVisitorError<T> = <CowVisitor<T> as Visitor>::Error;
impl<'a, T> IntoVisitor for Cow<'a, T>
where
    T: 'a + ToOwned + ?Sized,
    <T as ToOwned>::Owned: IntoVisitor,
{
    type Visitor = ext::AndThen<
        CowVisitor<T>,
        Box<
            dyn FnOnce(
                Result<<T as ToOwned>::Owned, CowVisitorError<T>>,
            ) -> Result<Cow<'a, T>, CowVisitorError<T>>,
        >,
        Cow<'a, T>,
        CowVisitorError<T>,
    >;
    fn into_visitor() -> Self::Visitor {
        let f = |res: Result<<T as ToOwned>::Owned, _>| res.map(|val| Cow::Owned(val));
        <<T as ToOwned>::Owned>::into_visitor().and_then(Box::new(f))
    }
}

macro_rules! impl_decode_seq_via_collect {
    ($ty:ident<$generic:ident> $(where $($where:tt)*)?) => {
        impl <$generic> Visitor for BasicVisitor<$ty<$generic>>
        where
            $generic: IntoVisitor,
            Error: From<<$generic::Visitor as Visitor>::Error>,
            $( $($where)* )?
        {
            type Value<'scale> = $ty<$generic>;
            type Error = Error;

            fn visit_tuple<'scale>(
                self,
                value: &mut Tuple<'scale, '_>,
                _type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                decode_items_using::<_, $generic>(value).collect()
            }
            fn visit_sequence<'scale>(
                self,
                value: &mut Sequence<'scale, '_>,
                _type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                decode_items_using::<_, $generic>(value).collect()
            }
            fn visit_array<'scale>(
                self,
                value: &mut Array<'scale, '_>,
                _type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                decode_items_using::<_, $generic>(value).collect()
            }
        }
        impl_into_visitor!($ty < $generic > $( where $($where)* )?);
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
    ($value:ident, $type_id:ident, [$t:ident; $n:ident]) => {{
        let val = decode_items_using::<_, $t>($value).collect::<Result<Vec<$t>, _>>()?;
        let actual_len = val.len();
        let arr = val.try_into().map_err(|_e| {
            Error::new(ErrorKind::WrongLength { actual: $type_id.0, actual_len, expected_len: $n })
        })?;
        Ok(arr)
    }};
}
impl<const N: usize, T> Visitor for BasicVisitor<[T; N]>
where
    T: IntoVisitor,
    Error: From<<T::Visitor as Visitor>::Error>,
{
    type Value<'scale> = [T; N];
    type Error = Error;

    fn visit_tuple<'scale>(
        self,
        value: &mut Tuple<'scale, '_>,
        type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        array_method_impl!(value, type_id, [T; N])
    }
    fn visit_sequence<'scale>(
        self,
        value: &mut Sequence<'scale, '_>,
        type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        array_method_impl!(value, type_id, [T; N])
    }
    fn visit_array<'scale>(
        self,
        value: &mut Array<'scale, '_>,
        type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        array_method_impl!(value, type_id, [T; N])
    }
}
impl<const N: usize, T> IntoVisitor for [T; N]
where
    T: IntoVisitor,
    Error: From<<T::Visitor as Visitor>::Error>,
{
    type Visitor = BasicVisitor<[T; N]>;
    fn into_visitor() -> Self::Visitor {
        BasicVisitor::new()
    }
}

impl<T> Visitor for BasicVisitor<BTreeMap<String, T>>
where
    T: IntoVisitor,
    Error: From<<T::Visitor as Visitor>::Error>,
{
    type Error = Error;
    type Value<'scale> = BTreeMap<String, T>;

    fn visit_composite<'scale>(
        self,
        value: &mut Composite<'scale, '_>,
        _type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        let mut map = BTreeMap::new();
        while value.remaining() > 0 {
            // Get the name. If no name, skip over the corresponding value.
            let Some(key) = value.peek_name() else {
                value.decode_item(crate::visitor::IgnoreVisitor).transpose()?;
                continue;
            };
            // Decode the value now that we have a valid name.
            let Some(val) = value.decode_item(T::into_visitor()) else {
                break
            };
            // Save to the map.
            let val = val.map_err(|e| Error::from(e).at_field(key.to_owned()))?;
            map.insert(key.to_owned(), val);
        }
        Ok(map)
    }
}
impl_into_visitor!(BTreeMap<String, T>);

impl<T> Visitor for BasicVisitor<Option<T>>
where
    T: IntoVisitor,
    Error: From<<T::Visitor as Visitor>::Error>,
{
    type Error = Error;
    type Value<'scale> = Option<T>;

    fn visit_variant<'scale>(
        self,
        value: &mut Variant<'scale, '_>,
        _type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        if value.name() == "Some" && value.fields().remaining() == 1 {
            let val = value
                .fields()
                .decode_item(T::into_visitor())
                .transpose()
                .map_err(|e| Error::from(e).at_variant("Some"))?
                .expect("checked for 1 field already so should be ok");
            Ok(Some(val))
        } else if value.name() == "None" && value.fields().remaining() == 0 {
            Ok(None)
        } else {
            Err(Error::new(ErrorKind::CannotFindVariant {
                got: value.name().to_string(),
                expected: vec!["Some", "None"],
            }))
        }
    }
}
impl_into_visitor!(Option<T>);

impl<T, E> Visitor for BasicVisitor<Result<T, E>>
where
    T: IntoVisitor,
    Error: From<<T::Visitor as Visitor>::Error>,
    E: IntoVisitor,
    Error: From<<E::Visitor as Visitor>::Error>,
{
    type Error = Error;
    type Value<'scale> = Result<T, E>;

    fn visit_variant<'scale>(
        self,
        value: &mut Variant<'scale, '_>,
        _type_id: visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        if value.name() == "Ok" && value.fields().remaining() == 1 {
            let val = value
                .fields()
                .decode_item(T::into_visitor())
                .transpose()
                .map_err(|e| Error::from(e).at_variant("Ok"))?
                .expect("checked for 1 field already so should be ok");
            Ok(Ok(val))
        } else if value.name() == "Err" && value.fields().remaining() == 1 {
            let val = value
                .fields()
                .decode_item(E::into_visitor())
                .transpose()
                .map_err(|e| Error::from(e).at_variant("Err"))?
                .expect("checked for 1 field already so should be ok");
            Ok(Err(val))
        } else {
            Err(Error::new(ErrorKind::CannotFindVariant {
                got: value.name().to_string(),
                expected: vec!["Ok", "Err"],
            }))
        }
    }
}
impl_into_visitor!(Result<T, E>);

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
                Error::new(ErrorKind::NumberOutOfRange { value: value.to_string() })
            })?;
            Ok(n)
        }
    };
}
macro_rules! visit_number_impl {
    ($ty:ident where |$res:ident| $expr:expr) => {
        impl Visitor for BasicVisitor<$ty> {
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
        impl_into_visitor!($ty);
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
    (($($t:ident,)*), $value:ident, $type_id:ident) => {{
        const EXPECTED_LEN: usize = count_idents!($($t)*);
        if $value.remaining() != EXPECTED_LEN {
            return Err(Error::new(ErrorKind::WrongLength {
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
                    idx += 1;
                    $value
                        .decode_item($t::into_visitor())
                        .transpose()
                        .map_err(|e| Error::from(e).at_idx(idx))?
                        .expect("length already checked via .remaining()")
                }
            ,)*
        ))
    }}
}
macro_rules! impl_decode_tuple {
    ($($t:ident)*) => {
        impl < $($t),* > Visitor for BasicVisitor<($($t,)*)>
        where $(
            $t: IntoVisitor,
            Error: From<<$t::Visitor as Visitor>::Error>,
        )*
        {
            type Value<'scale> = ($($t,)*);
            type Error = Error;

            fn visit_composite<'scale>(
                self,
                value: &mut Composite<'scale, '_>,
                type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                tuple_method_impl!(($($t,)*), value, type_id)
            }
            fn visit_tuple<'scale>(
                self,
                value: &mut Tuple<'scale, '_>,
                type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                tuple_method_impl!(($($t,)*), value, type_id)
            }
            fn visit_sequence<'scale>(
                self,
                value: &mut Sequence<'scale, '_>,
                type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                tuple_method_impl!(($($t,)*), value, type_id)
            }
            fn visit_array<'scale>(
                self,
                value: &mut Array<'scale, '_>,
                type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale>, Self::Error> {
                tuple_method_impl!(($($t,)*), value, type_id)
            }
        }
        impl < $($t),* > IntoVisitor for ($($t,)*)
        where $( $t: IntoVisitor, Error: From<<$t::Visitor as Visitor>::Error>, )*
        {
            type Visitor = BasicVisitor<($($t,)*)>;
            fn into_visitor() -> Self::Visitor {
                BasicVisitor::new()
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
) -> impl Iterator<Item = Result<T, Error>> + 'a
where
    T: IntoVisitor,
    Error: From<<T::Visitor as Visitor>::Error>,
    D: DecodeItemIterator<'scale>,
{
    let mut idx = 0;
    std::iter::from_fn(move || {
        let item = decoder
            .decode_item(T::into_visitor())
            .map(|res| res.map_err(|e| Error::from(e).at_idx(idx)));
        idx += 1;
        item
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::DecodeAsType;

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
        let encoded = a.encode_as_type(type_id, &types).expect("should be able to encode");
        let decoded =
            B::decode_as_type(&mut &*encoded, type_id, &types).expect("should be able to decode");
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
