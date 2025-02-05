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

//! The [`Visitor`] trait and associated types.

mod decode;
pub mod types;

use alloc::string::String;
use core::marker::PhantomData;
use scale_type_resolver::TypeResolver;
use types::*;

pub use decode::decode_with_visitor;
pub(crate) use decode::decode_with_visitor_maybe_compact;

/// Return the type ID type of some [`Visitor`].
pub type TypeIdFor<V> = <<V as Visitor>::TypeResolver as TypeResolver>::TypeId;

/// An implementation of the [`Visitor`] trait can be passed to the [`decode_with_visitor()`]
/// function, and is handed back values as they are encountered. It's up to the implementation
/// to decide what to do with these values.
pub trait Visitor: Sized {
    /// The type of the value to hand back from the [`decode_with_visitor()`] function.
    type Value<'scale, 'resolver>;
    /// The error type (which we must be able to convert a combination of [`Self`] and [`DecodeError`]s
    /// into, to handle any internal errors that crop up trying to decode things).
    type Error: From<DecodeError>;
    /// The thing we'll use to resolve type IDs into concrete types.
    type TypeResolver: TypeResolver;

    /// This method is called immediately upon running [`decode_with_visitor()`]. By default we ignore
    /// this call and return our visitor back (ie [`DecodeAsTypeResult::Skipped(visitor)`]). If you choose to
    /// do some decoding at this stage, return [`DecodeAsTypeResult::Decoded(result)`]. In either case, any bytes
    /// that you consume from the input (by altering what it points to) will be consumed for any subsequent visiting.
    ///
    /// # Warning
    ///
    /// Unlike the other `visit_*` methods, it is completely up to the implementor to decode and advance the
    /// bytes in a sensible way, and thus also possible for the implementor to screw this up. As a result,
    /// it's suggested that you don't implement this unless you know what you're doing.
    fn unchecked_decode_as_type<'scale, 'resolver>(
        self,
        _input: &mut &'scale [u8],
        _type_id: TypeIdFor<Self>,
        _types: &'resolver Self::TypeResolver,
    ) -> DecodeAsTypeResult<Self, Result<Self::Value<'scale, 'resolver>, Self::Error>> {
        DecodeAsTypeResult::Skipped(self)
    }

    /// This is called when a visitor function that you've not provided an implementation is called.
    /// You are provided an enum value corresponding to the function call, and can decide what to return
    /// in this case. The default is to return an error to announce the unexpected value.
    fn visit_unexpected<'scale, 'resolver>(
        self,
        unexpected: Unexpected,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Err(DecodeError::Unexpected(unexpected).into())
    }

    /// Called when a bool is seen in the input bytes.
    fn visit_bool<'scale, 'resolver>(
        self,
        _value: bool,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::Bool)
    }
    /// Called when a char is seen in the input bytes.
    fn visit_char<'scale, 'resolver>(
        self,
        _value: char,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::Char)
    }
    /// Called when a u8 is seen in the input bytes.
    fn visit_u8<'scale, 'resolver>(
        self,
        _value: u8,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::U8)
    }
    /// Called when a u16 is seen in the input bytes.
    fn visit_u16<'scale, 'resolver>(
        self,
        _value: u16,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::U16)
    }
    /// Called when a u32 is seen in the input bytes.
    fn visit_u32<'scale, 'resolver>(
        self,
        _value: u32,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::U32)
    }
    /// Called when a u64 is seen in the input bytes.
    fn visit_u64<'scale, 'resolver>(
        self,
        _value: u64,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::U64)
    }
    /// Called when a u128 is seen in the input bytes.
    fn visit_u128<'scale, 'resolver>(
        self,
        _value: u128,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::U128)
    }
    /// Called when a u256 is seen in the input bytes.
    fn visit_u256<'resolver>(
        self,
        _value: &[u8; 32],
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'_, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::U256)
    }
    /// Called when an i8 is seen in the input bytes.
    fn visit_i8<'scale, 'resolver>(
        self,
        _value: i8,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::I8)
    }
    /// Called when an i16 is seen in the input bytes.
    fn visit_i16<'scale, 'resolver>(
        self,
        _value: i16,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::I16)
    }
    /// Called when an i32 is seen in the input bytes.
    fn visit_i32<'scale, 'resolver>(
        self,
        _value: i32,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::I32)
    }
    /// Called when an i64 is seen in the input bytes.
    fn visit_i64<'scale, 'resolver>(
        self,
        _value: i64,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::I64)
    }
    /// Called when an i128 is seen in the input bytes.
    fn visit_i128<'scale, 'resolver>(
        self,
        _value: i128,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::I128)
    }
    /// Called when an i256 is seen in the input bytes.
    fn visit_i256<'resolver>(
        self,
        _value: &[u8; 32],
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'_, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::I256)
    }
    /// Called when a sequence of values is seen in the input bytes.
    fn visit_sequence<'scale, 'resolver>(
        self,
        _value: &mut Sequence<'scale, 'resolver, Self::TypeResolver>,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::Sequence)
    }
    /// Called when a composite value is seen in the input bytes.
    fn visit_composite<'scale, 'resolver>(
        self,
        _value: &mut Composite<'scale, 'resolver, Self::TypeResolver>,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::Composite)
    }
    /// Called when a tuple of values is seen in the input bytes.
    fn visit_tuple<'scale, 'resolver>(
        self,
        _value: &mut Tuple<'scale, 'resolver, Self::TypeResolver>,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::Tuple)
    }
    /// Called when a string value is seen in the input bytes.
    fn visit_str<'scale, 'resolver>(
        self,
        _value: &mut Str<'scale>,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::Str)
    }
    /// Called when a variant is seen in the input bytes.
    fn visit_variant<'scale, 'resolver>(
        self,
        _value: &mut Variant<'scale, 'resolver, Self::TypeResolver>,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::Variant)
    }
    /// Called when an array is seen in the input bytes.
    fn visit_array<'scale, 'resolver>(
        self,
        _value: &mut Array<'scale, 'resolver, Self::TypeResolver>,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::Array)
    }
    /// Called when a bit sequence is seen in the input bytes.
    fn visit_bitsequence<'scale, 'resolver>(
        self,
        _value: &mut BitSequence<'scale>,
        _type_id: TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.visit_unexpected(Unexpected::Bitsequence)
    }
}

/// An error decoding SCALE bytes.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DecodeError {
    /// Type ID was not found
    #[error("Could not find type with ID '{0}'")]
    TypeIdNotFound(String),
    /// A low level error trying to resolve a type.
    #[error("Failed to resolve type: {0}")]
    TypeResolvingError(String),
    /// The type we're trying to decode is supposed to be compact encoded, but that is not possible.
    #[error("Could not decode compact encoded type: compact types can only have 1 field")]
    CannotDecodeCompactIntoType,
    /// Failure to decode bytes into a string.
    #[error("Could not decode string: {0}")]
    InvalidStr(alloc::str::Utf8Error),
    /// We could not convert the [`u32`] that we found into a valid [`char`].
    #[error("{_0} is expected to be a valid char, but is not")]
    InvalidChar(u32),
    /// We expected more bytes to finish decoding, but could not find them.
    #[error("Ran out of data during decoding")]
    NotEnoughInput,
    /// We found a variant that does not match with any in the type we're trying to decode from.
    #[error("Could not find variant with index {_0}")]
    VariantNotFound(u8),
    /// Some error emitted from a [`codec::Decode`] impl.
    #[error("Decode error: {0}")]
    CodecError(codec::Error),
    /// This is returned by default if a visitor function is not implemented.
    #[error("Unexpected type {_0}")]
    Unexpected(#[from] Unexpected),
}

// TODO(niklasad1): when `codec::Error` implements `core::error::Error` we can remove this impl
// and use thiserror::Error #[from] instead.
impl From<codec::Error> for DecodeError {
    fn from(e: codec::Error) -> Self {
        DecodeError::CodecError(e)
    }
}

/// This is returned by default when a visitor function isn't implemented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[allow(missing_docs)]
pub enum Unexpected {
    #[error("bool")]
    Bool,
    #[error("char")]
    Char,
    #[error("u8")]
    U8,
    #[error("u16")]
    U16,
    #[error("u32")]
    U32,
    #[error("u64")]
    U64,
    #[error("u128")]
    U128,
    #[error("u256")]
    U256,
    #[error("i8")]
    I8,
    #[error("i16")]
    I16,
    #[error("i32")]
    I32,
    #[error("i64")]
    I64,
    #[error("i128")]
    I128,
    #[error("i256")]
    I256,
    #[error("sequence")]
    Sequence,
    #[error("composite")]
    Composite,
    #[error("tuple")]
    Tuple,
    #[error("str")]
    Str,
    #[error("variant")]
    Variant,
    #[error("array")]
    Array,
    #[error("bitsequence")]
    Bitsequence,
}

/// The response from [`Visitor::unchecked_decode_as_type()`].
pub enum DecodeAsTypeResult<V, R> {
    /// Skip any manual decoding and return the visitor instead.
    Skipped(V),
    /// Some manually decoded result.
    Decoded(R),
}

impl<V, R> DecodeAsTypeResult<V, R> {
    /// If we have a [`DecodeAsTypeResult::Decoded`], the function provided will
    /// map this decoded result to whatever it returns.
    pub fn map_decoded<T, F: FnOnce(R) -> T>(self, f: F) -> DecodeAsTypeResult<V, T> {
        match self {
            DecodeAsTypeResult::Skipped(s) => DecodeAsTypeResult::Skipped(s),
            DecodeAsTypeResult::Decoded(r) => DecodeAsTypeResult::Decoded(f(r)),
        }
    }

    /// If we have a [`DecodeAsTypeResult::Skipped`], the function provided will
    /// map this skipped value to whatever it returns.
    pub fn map_skipped<T, F: FnOnce(V) -> T>(self, f: F) -> DecodeAsTypeResult<T, R> {
        match self {
            DecodeAsTypeResult::Skipped(s) => DecodeAsTypeResult::Skipped(f(s)),
            DecodeAsTypeResult::Decoded(r) => DecodeAsTypeResult::Decoded(r),
        }
    }
}

/// This is implemented for visitor related types which have a `decode_item` method,
/// and allows you to generically talk about decoding unnamed items.
pub trait DecodeItemIterator<'scale, 'resolver, R: TypeResolver> {
    /// Use a visitor to decode a single item.
    fn decode_item<V: Visitor<TypeResolver = R>>(
        &mut self,
        visitor: V,
    ) -> Option<Result<V::Value<'scale, 'resolver>, V::Error>>;
}

/// A [`Visitor`] implementation that just ignores all of the bytes.
pub struct IgnoreVisitor<R>(PhantomData<R>);

impl<R> Default for IgnoreVisitor<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> IgnoreVisitor<R> {
    /// Construct a new [`IgnoreVisitor`].
    pub fn new() -> Self {
        IgnoreVisitor(PhantomData)
    }
}

impl<R: TypeResolver> Visitor for IgnoreVisitor<R> {
    type Value<'scale, 'resolver> = ();
    type Error = DecodeError;
    type TypeResolver = R;

    // Whatever the value we visit is, just ignore it.
    fn visit_unexpected<'scale, 'resolver>(
        self,
        _unexpected: Unexpected,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(())
    }
}

/// Some [`Visitor`] implementations may want to return an error type other than [`crate::Error`], which means
/// that they would not be automatically compatible with [`crate::IntoVisitor`], which requires visitors that do return
/// [`crate::Error`] errors.
///
/// As long as the error type of the visitor implementation can be converted into [`crate::Error`] via [`Into`],
/// the visitor implementation can be wrapped in this [`VisitorWithCrateError`] struct to make it work with
/// [`crate::IntoVisitor`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VisitorWithCrateError<V>(pub V);

impl<V: Visitor> Visitor for VisitorWithCrateError<V>
where
    V::Error: Into<crate::Error>,
{
    type Value<'scale, 'resolver> = V::Value<'scale, 'resolver>;
    type Error = crate::Error;
    type TypeResolver = V::TypeResolver;

    fn unchecked_decode_as_type<'scale, 'resolver>(
        self,
        input: &mut &'scale [u8],
        type_id: TypeIdFor<Self>,
        types: &'resolver Self::TypeResolver,
    ) -> DecodeAsTypeResult<Self, Result<Self::Value<'scale, 'resolver>, Self::Error>> {
        let res = decode_with_visitor(input, type_id, types, self.0).map_err(Into::into);
        DecodeAsTypeResult::Decoded(res)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::borrow::ToOwned;
    use alloc::string::{String, ToString};
    use alloc::vec;
    use alloc::vec::Vec;
    use codec::{self, CompactAs, Encode};
    use scale_info::PortableRegistry;

    /// A silly Value type for testing with a basic Visitor impl
    /// that tries to mirror what is called as best as possible.
    #[derive(Debug, PartialEq)]
    enum Value {
        Bool(bool),
        Char(char),
        U8(u8),
        U16(u16),
        U32(u32),
        U64(u64),
        U128(u128),
        U256([u8; 32]),
        I8(i8),
        I16(i16),
        I32(i32),
        I64(i64),
        I128(i128),
        I256([u8; 32]),
        Sequence(Vec<Value>),
        Composite(Vec<(String, Value)>),
        Tuple(Vec<Value>),
        Str(String),
        Array(Vec<Value>),
        Variant(String, Vec<(String, Value)>),
        BitSequence(scale_bits::Bits),
    }

    struct ValueVisitor<R>(PhantomData<R>);
    impl<R> Clone for ValueVisitor<R> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<R> Copy for ValueVisitor<R> {}

    impl<R> ValueVisitor<R> {
        pub fn new() -> Self {
            Self(PhantomData)
        }
    }

    impl<R: TypeResolver> Visitor for ValueVisitor<R> {
        type Value<'scale, 'resolver> = Value;
        type Error = DecodeError;
        type TypeResolver = R;

        fn visit_bool<'scale, 'resolver>(
            self,
            value: bool,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            Ok(Value::Bool(value))
        }
        fn visit_char<'scale, 'resolver>(
            self,
            value: char,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            Ok(Value::Char(value))
        }
        fn visit_u8<'scale, 'resolver>(
            self,
            value: u8,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            Ok(Value::U8(value))
        }
        fn visit_u16<'scale, 'resolver>(
            self,
            value: u16,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            Ok(Value::U16(value))
        }
        fn visit_u32<'scale, 'resolver>(
            self,
            value: u32,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            Ok(Value::U32(value))
        }
        fn visit_u64<'scale, 'resolver>(
            self,
            value: u64,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            Ok(Value::U64(value))
        }
        fn visit_u128<'scale, 'resolver>(
            self,
            value: u128,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            Ok(Value::U128(value))
        }
        fn visit_u256<'resolver>(
            self,
            value: &[u8; 32],
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'_, 'resolver>, Self::Error> {
            Ok(Value::U256(*value))
        }
        fn visit_i8<'scale, 'resolver>(
            self,
            value: i8,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            Ok(Value::I8(value))
        }
        fn visit_i16<'scale, 'resolver>(
            self,
            value: i16,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            Ok(Value::I16(value))
        }
        fn visit_i32<'scale, 'resolver>(
            self,
            value: i32,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            Ok(Value::I32(value))
        }
        fn visit_i64<'scale, 'resolver>(
            self,
            value: i64,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            Ok(Value::I64(value))
        }
        fn visit_i128<'scale, 'resolver>(
            self,
            value: i128,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            Ok(Value::I128(value))
        }
        fn visit_i256<'resolver>(
            self,
            value: &[u8; 32],
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'_, 'resolver>, Self::Error> {
            Ok(Value::I256(*value))
        }
        fn visit_sequence<'scale, 'resolver>(
            self,
            value: &mut Sequence<'scale, 'resolver, Self::TypeResolver>,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            let mut vals = vec![];
            while let Some(val) = value.decode_item(ValueVisitor::new()) {
                let val = val?;
                vals.push(val);
            }
            Ok(Value::Sequence(vals))
        }
        fn visit_composite<'scale, 'resolver>(
            self,
            value: &mut Composite<'scale, 'resolver, Self::TypeResolver>,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            let mut vals = vec![];
            for item in value.by_ref() {
                let item = item?;
                let val = item.decode_with_visitor(ValueVisitor::new())?;
                let name = item.name().unwrap_or("").to_owned();
                vals.push((name, val));
            }
            Ok(Value::Composite(vals))
        }
        fn visit_tuple<'scale, 'resolver>(
            self,
            value: &mut Tuple<'scale, 'resolver, Self::TypeResolver>,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            let mut vals = vec![];
            while let Some(val) = value.decode_item(ValueVisitor::new()) {
                let val = val?;
                vals.push(val);
            }
            Ok(Value::Tuple(vals))
        }
        fn visit_str<'scale, 'resolver>(
            self,
            value: &mut Str<'scale>,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            Ok(Value::Str(value.as_str()?.to_owned()))
        }
        fn visit_variant<'scale, 'resolver>(
            self,
            value: &mut Variant<'scale, 'resolver, Self::TypeResolver>,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            let mut vals = vec![];
            let fields = value.fields();
            for item in fields.by_ref() {
                let item = item?;
                let val = item.decode_with_visitor(ValueVisitor::new())?;
                let name = item.name().unwrap_or("").to_owned();
                vals.push((name, val));
            }
            Ok(Value::Variant(value.name().to_owned(), vals))
        }
        fn visit_array<'scale, 'resolver>(
            self,
            value: &mut Array<'scale, 'resolver, Self::TypeResolver>,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            let mut vals = vec![];
            while let Some(val) = value.decode_item(ValueVisitor::new()) {
                let val = val?;
                vals.push(val);
            }
            Ok(Value::Array(vals))
        }
        fn visit_bitsequence<'scale, 'resolver>(
            self,
            value: &mut BitSequence<'scale>,
            _type_id: TypeIdFor<Self>,
        ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
            let bools: Result<scale_bits::Bits, _> = value.decode()?.collect();
            Ok(Value::BitSequence(bools?))
        }
    }

    /// Given a type definition, return the PortableType and PortableRegistry
    /// that our decode functions expect.
    fn make_type<T: scale_info::TypeInfo + 'static>() -> (u32, PortableRegistry) {
        let m = scale_info::MetaType::new::<T>();
        let mut types = scale_info::Registry::new();
        let id = types.register_type(&m);
        let portable_registry: PortableRegistry = types.into();

        (id.id, portable_registry)
    }

    /// This just tests that if we try to decode some values we've encoded using a visitor
    /// which just ignores everything by default, that we'll consume all of the bytes.
    fn encode_decode_check_explicit_info<
        Ty: scale_info::TypeInfo + 'static,
        T: Encode,
        V: for<'s, 'i> Visitor<Value<'s, 'i> = Value, Error = E, TypeResolver = PortableRegistry>,
        E: core::fmt::Debug,
    >(
        val: T,
        expected: Value,
        visitor: V,
    ) {
        let encoded = val.encode();
        let (id, types) = make_type::<Ty>();
        let bytes = &mut &*encoded;
        let val =
            decode_with_visitor(bytes, id, &types, visitor).expect("decoding should not error");

        assert_eq!(bytes.len(), 0, "Decoding should consume all bytes");
        assert_eq!(val, expected);
    }

    fn encode_decode_check_with_visitor<
        T: Encode + scale_info::TypeInfo + 'static,
        V: for<'s, 'i> Visitor<Value<'s, 'i> = Value, Error = E, TypeResolver = PortableRegistry>,
        E: core::fmt::Debug,
    >(
        val: T,
        expected: Value,
        visitor: V,
    ) {
        encode_decode_check_explicit_info::<T, T, _, _>(val, expected, visitor);
    }

    fn encode_decode_check<T: Encode + scale_info::TypeInfo + 'static>(val: T, expected: Value) {
        encode_decode_check_explicit_info::<T, T, _, _>(val, expected, ValueVisitor::new());
    }

    #[test]
    fn decode_with_root_error_wrapper_works() {
        use crate::visitor::VisitorWithCrateError;
        let visitor = VisitorWithCrateError(ValueVisitor::new());

        encode_decode_check_with_visitor(123u8, Value::U8(123), visitor);
        encode_decode_check_with_visitor(123u16, Value::U16(123), visitor);
        encode_decode_check_with_visitor(123u32, Value::U32(123), visitor);
        encode_decode_check_with_visitor(123u64, Value::U64(123), visitor);
        encode_decode_check_with_visitor(123u128, Value::U128(123), visitor);
        encode_decode_check_with_visitor(
            "Hello there",
            Value::Str("Hello there".to_owned()),
            visitor,
        );

        #[derive(Encode, scale_info::TypeInfo)]
        struct Unnamed(bool, String, Vec<u8>);
        encode_decode_check_with_visitor(
            Unnamed(true, "James".into(), vec![1, 2, 3]),
            Value::Composite(vec![
                (String::new(), Value::Bool(true)),
                (String::new(), Value::Str("James".to_string())),
                (String::new(), Value::Sequence(vec![Value::U8(1), Value::U8(2), Value::U8(3)])),
            ]),
            visitor,
        );
    }

    #[test]
    fn encode_decode_primitives() {
        encode_decode_check(123u8, Value::U8(123));
        encode_decode_check(123u16, Value::U16(123));
        encode_decode_check(123u32, Value::U32(123));
        encode_decode_check(123u64, Value::U64(123));
        encode_decode_check(123u128, Value::U128(123));
        encode_decode_check(codec::Compact(123u8), Value::U8(123));
        encode_decode_check(codec::Compact(123u16), Value::U16(123));
        encode_decode_check(codec::Compact(123u32), Value::U32(123));
        encode_decode_check(codec::Compact(123u64), Value::U64(123));
        encode_decode_check(codec::Compact(123u128), Value::U128(123));
        encode_decode_check(true, Value::Bool(true));
        encode_decode_check(false, Value::Bool(false));
        encode_decode_check_explicit_info::<char, _, _, _>(
            'c' as u32,
            Value::Char('c'),
            ValueVisitor::new(),
        );
        encode_decode_check("Hello there", Value::Str("Hello there".to_owned()));
        encode_decode_check("Hello there".to_string(), Value::Str("Hello there".to_owned()));
    }

    #[test]
    fn decode_compact_named_wrapper_struct() {
        // A struct that can be compact encoded:
        #[derive(Encode, scale_info::TypeInfo, CompactAs)]
        struct MyWrapper {
            inner: u32,
        }

        encode_decode_check(
            codec::Compact(MyWrapper { inner: 123 }),
            Value::Composite(vec![("inner".into(), Value::U32(123))]),
        );
    }

    #[test]
    fn decode_compact_unnamed_wrapper_struct() {
        // A struct that can be compact encoded:
        #[derive(Encode, scale_info::TypeInfo, CompactAs)]
        struct MyWrapper(u32);

        encode_decode_check(
            codec::Compact(MyWrapper(123)),
            Value::Composite(vec![("".into(), Value::U32(123))]),
        );
    }

    #[test]
    fn decode_nested_compact_structs() {
        // A struct that has a nested field inner1, which can be compact encoded:
        #[derive(Encode, scale_info::TypeInfo)]
        struct Outer {
            #[codec(compact)]
            inner1: Inner1,
            other: u16,
        }

        #[derive(Encode, scale_info::TypeInfo, CompactAs)]
        struct Inner1 {
            inner2: Inner2,
        }

        #[derive(Encode, scale_info::TypeInfo, CompactAs)]
        struct Inner2(u64);

        let struct_to_check = Outer { inner1: Inner1 { inner2: Inner2(123) }, other: 42 };
        let expacted_value = Value::Composite(vec![
            (
                "inner1".into(),
                Value::Composite(vec![(
                    "inner2".into(),
                    Value::Composite(vec![("".into(), Value::U64(123))]),
                )]),
            ),
            ("other".into(), Value::U16(42)),
        ]);
        encode_decode_check(struct_to_check, expacted_value);
    }

    #[test]
    fn decode_compact_single_item_tuple_field() {
        // A struct that can be compact encoded:
        #[derive(Encode, scale_info::TypeInfo, CompactAs)]
        struct Struct {
            a: (u32,),
        }

        encode_decode_check(
            Struct { a: (123,) },
            Value::Composite(vec![("a".into(), Value::Tuple(vec![Value::U32(123)]))]),
        );
    }

    #[test]
    fn decode_sequence_array_tuple_types() {
        encode_decode_check(
            vec![1i32, 2, 3],
            Value::Sequence(vec![Value::I32(1), Value::I32(2), Value::I32(3)]),
        );
        encode_decode_check(
            [1i32, 2, 3], // compile-time length known
            Value::Array(vec![Value::I32(1), Value::I32(2), Value::I32(3)]),
        );
        encode_decode_check(
            (1i32, true, 123456u128),
            Value::Tuple(vec![Value::I32(1), Value::Bool(true), Value::U128(123456)]),
        );
    }

    #[test]
    fn decode_variant_types() {
        #[derive(Encode, scale_info::TypeInfo)]
        enum MyEnum {
            Foo(bool),
            Bar { hi: String, other: u128 },
        }

        encode_decode_check(
            MyEnum::Foo(true),
            Value::Variant("Foo".to_owned(), vec![(String::new(), Value::Bool(true))]),
        );
        encode_decode_check(
            MyEnum::Bar { hi: "hello".to_string(), other: 123 },
            Value::Variant(
                "Bar".to_owned(),
                vec![
                    ("hi".to_string(), Value::Str("hello".to_string())),
                    ("other".to_string(), Value::U128(123)),
                ],
            ),
        );
    }

    #[test]
    fn decode_composite_types() {
        #[derive(Encode, scale_info::TypeInfo)]
        struct Unnamed(bool, String, Vec<u8>);

        #[derive(Encode, scale_info::TypeInfo)]
        struct Named {
            is_valid: bool,
            name: String,
            bytes: Vec<u8>,
        }

        encode_decode_check(
            Unnamed(true, "James".into(), vec![1, 2, 3]),
            Value::Composite(vec![
                (String::new(), Value::Bool(true)),
                (String::new(), Value::Str("James".to_string())),
                (String::new(), Value::Sequence(vec![Value::U8(1), Value::U8(2), Value::U8(3)])),
            ]),
        );
        encode_decode_check(
            Named { is_valid: true, name: "James".into(), bytes: vec![1, 2, 3] },
            Value::Composite(vec![
                ("is_valid".to_string(), Value::Bool(true)),
                ("name".to_string(), Value::Str("James".to_string())),
                (
                    "bytes".to_string(),
                    Value::Sequence(vec![Value::U8(1), Value::U8(2), Value::U8(3)]),
                ),
            ]),
        );
    }

    #[test]
    fn decode_arrays() {
        encode_decode_check(
            [1u8, 2, 3],
            Value::Array(vec![Value::U8(1), Value::U8(2), Value::U8(3)]),
        )
    }

    #[test]
    fn decode_bit_sequence() {
        use bitvec::{
            bitvec,
            order::{Lsb0, Msb0},
        };
        use scale_bits::bits;

        // Check that Bits, as well as all compatible bitvecs, are properly decoded.
        encode_decode_check(bits![0, 1, 1, 0, 1, 0], Value::BitSequence(bits![0, 1, 1, 0, 1, 0]));
        encode_decode_check(
            bitvec![u8, Lsb0; 0, 1, 1, 0, 1, 0],
            Value::BitSequence(bits![0, 1, 1, 0, 1, 0]),
        );
        encode_decode_check(
            bitvec![u16, Lsb0; 0, 1, 1, 0, 1, 0],
            Value::BitSequence(bits![0, 1, 1, 0, 1, 0]),
        );
        encode_decode_check(
            bitvec![u32, Lsb0; 0, 1, 1, 0, 1, 0],
            Value::BitSequence(bits![0, 1, 1, 0, 1, 0]),
        );
        encode_decode_check(
            bitvec![u64, Lsb0; 0, 1, 1, 0, 1, 0],
            Value::BitSequence(bits![0, 1, 1, 0, 1, 0]),
        );
        encode_decode_check(
            bitvec![u8, Msb0; 0, 1, 1, 0, 1, 0],
            Value::BitSequence(bits![0, 1, 1, 0, 1, 0]),
        );
        encode_decode_check(
            bitvec![u16, Msb0; 0, 1, 1, 0, 1, 0],
            Value::BitSequence(bits![0, 1, 1, 0, 1, 0]),
        );
        encode_decode_check(
            bitvec![u32, Msb0; 0, 1, 1, 0, 1, 0],
            Value::BitSequence(bits![0, 1, 1, 0, 1, 0]),
        );
        encode_decode_check(
            bitvec![u64, Msb0; 0, 1, 1, 0, 1, 0],
            Value::BitSequence(bits![0, 1, 1, 0, 1, 0]),
        );
    }

    // We want to make sure that if the visitor returns an error, then that error is propagated
    // up to the user. with some types (Sequence/Composite/Tuple/Array/Variant), we skip over
    // undecoded items after the visitor runs, and want to ensure that any error skipping over
    // things doesn't mask any visitor error.
    //
    // These tests all fail prior to https://github.com/paritytech/scale-decode/pull/58 and pass
    // after it.
    macro_rules! decoding_returns_error_first {
        ($name:ident $expr:expr) => {
            #[test]
            fn $name() {
                fn visitor_err() -> DecodeError {
                    DecodeError::TypeResolvingError("Whoops".to_string())
                }

                #[derive(codec::Encode)]
                struct HasBadTypeInfo;
                impl scale_info::TypeInfo for HasBadTypeInfo {
                    type Identity = Self;
                    fn type_info() -> scale_info::Type {
                        // The actual struct is zero bytes but the type info says it is 1 byte,
                        // so using type info to decode it will lead to failures.
                        scale_info::meta_type::<u8>().type_info()
                    }
                }

                struct VisitorImpl;
                impl Visitor for VisitorImpl {
                    type Value<'scale, 'resolver> = ();
                    type Error = DecodeError;
                    type TypeResolver = PortableRegistry;

                    fn visit_unexpected<'scale, 'resolver>(
                        self,
                        _unexpected: Unexpected,
                    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
                        // Our visitor just returns a specific error, so we can check that
                        // we get it back when trying to decode.
                        Err(visitor_err())
                    }
                }

                fn assert_visitor_err<E: codec::Encode + scale_info::TypeInfo + 'static>(input: E) {
                    let input_encoded = input.encode();
                    let (ty_id, types) = make_type::<E>();
                    let err = decode_with_visitor(&mut &*input_encoded, ty_id, &types, VisitorImpl)
                        .unwrap_err();
                    assert_eq!(err, visitor_err());
                }

                assert_visitor_err($expr);
            }
        };
    }

    decoding_returns_error_first!(decode_composite_returns_error_first {
        #[derive(codec::Encode, scale_info::TypeInfo)]
        struct SomeComposite {
            a: bool,
            b: HasBadTypeInfo,
            c: Vec<u8>
        }

        SomeComposite { a: true, b: HasBadTypeInfo, c: vec![1,2,3] }
    });

    decoding_returns_error_first!(decode_variant_returns_error_first {
        #[derive(codec::Encode, scale_info::TypeInfo)]
        enum SomeVariant {
            Foo(u32, HasBadTypeInfo, String)
        }

        SomeVariant::Foo(32, HasBadTypeInfo, "hi".to_owned())
    });

    decoding_returns_error_first!(decode_array_returns_error_first {
        [HasBadTypeInfo, HasBadTypeInfo]
    });

    decoding_returns_error_first!(decode_sequence_returns_error_first {
        vec![HasBadTypeInfo, HasBadTypeInfo]
    });

    decoding_returns_error_first!(decode_tuple_returns_error_first {
        (32u64, HasBadTypeInfo, true)
    });

    #[test]
    fn zero_copy_string_decoding() {
        let input = ("hello", "world");

        // The SCALE encoded bytes we want to zero-copy-decode from:
        let input_encoded = input.encode();

        // This can just zero-copy decode a string:
        struct ZeroCopyStrVisitor;
        impl Visitor for ZeroCopyStrVisitor {
            type Value<'scale, 'resolver> = &'scale str;
            type Error = DecodeError;
            type TypeResolver = PortableRegistry;

            fn visit_str<'scale, 'resolver>(
                self,
                value: &mut Str<'scale>,
                _type_id: TypeIdFor<Self>,
            ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
                value.as_str()
            }
        }

        // This can zero-copy decode the pair of strings we have as input:
        struct ZeroCopyPairVisitor;
        impl Visitor for ZeroCopyPairVisitor {
            type Value<'scale, 'resolver> = (&'scale str, &'scale str);
            type Error = DecodeError;
            type TypeResolver = PortableRegistry;

            fn visit_tuple<'scale, 'resolver>(
                self,
                value: &mut Tuple<'scale, 'resolver, Self::TypeResolver>,
                _type_id: TypeIdFor<Self>,
            ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
                let fst = value.decode_item(ZeroCopyStrVisitor).unwrap()?;
                let snd = value.decode_item(ZeroCopyStrVisitor).unwrap()?;
                Ok((fst, snd))
            }
        }

        let (ty_id, types) = make_type::<(&str, &str)>();
        let decoded =
            decode_with_visitor(&mut &*input_encoded, ty_id, &types, ZeroCopyPairVisitor).unwrap();
        assert_eq!(decoded, ("hello", "world"));
    }

    #[test]
    fn zero_copy_using_info_and_scale_lifetimes() {
        use alloc::collections::BTreeMap;

        #[derive(codec::Encode, scale_info::TypeInfo)]
        struct Foo {
            hello: String,
            world: String,
        }

        // The SCALE encoded bytes we want to zero-copy-decode from:
        let input_encoded = Foo { hello: "hi".to_string(), world: "planet".to_string() }.encode();

        // This can just zero-copy decode a string:
        struct ZeroCopyStrVisitor;
        impl Visitor for ZeroCopyStrVisitor {
            type Value<'scale, 'resolver> = &'scale str;
            type Error = DecodeError;
            type TypeResolver = PortableRegistry;

            fn visit_str<'scale, 'resolver>(
                self,
                value: &mut Str<'scale>,
                _type_id: TypeIdFor<Self>,
            ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
                value.as_str()
            }
        }

        // This zero-copy decodes a composite into map of strings:
        struct ZeroCopyMapVisitor;
        impl Visitor for ZeroCopyMapVisitor {
            type Value<'scale, 'resolver> =
                alloc::collections::BTreeMap<&'resolver str, &'scale str>;
            type Error = DecodeError;
            type TypeResolver = PortableRegistry;

            fn visit_composite<'scale, 'resolver>(
                self,
                value: &mut Composite<'scale, 'resolver, Self::TypeResolver>,
                _type_id: TypeIdFor<Self>,
            ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
                let mut vals = alloc::collections::BTreeMap::<&'resolver str, &'scale str>::new();
                for item in value {
                    let item = item?;
                    let Some(key) = item.name() else { continue };
                    let val = item.decode_with_visitor(ZeroCopyStrVisitor)?;
                    vals.insert(key, val);
                }
                Ok(vals)
            }
        }

        // Decode and check:
        let (ty_id, types) = make_type::<Foo>();
        let decoded =
            decode_with_visitor(&mut &*input_encoded, ty_id, &types, ZeroCopyMapVisitor).unwrap();
        assert_eq!(decoded, BTreeMap::from_iter([("hello", "hi"), ("world", "planet")]));
    }

    #[test]
    fn bailout_works() {
        let input = ("hello", "world");
        let (ty_id, types) = make_type::<(&str, &str)>();
        let input_encoded = input.encode();

        // Just return the scale encoded bytes and type ID to prove
        // that we can successfully "bail out".
        struct BailOutVisitor;
        impl Visitor for BailOutVisitor {
            type Value<'scale, 'resolver> = (&'scale [u8], u32);
            type Error = DecodeError;
            type TypeResolver = PortableRegistry;

            fn unchecked_decode_as_type<'scale, 'resolver>(
                self,
                input: &mut &'scale [u8],
                type_id: u32,
                _types: &'resolver scale_info::PortableRegistry,
            ) -> DecodeAsTypeResult<Self, Result<Self::Value<'scale, 'resolver>, Self::Error>>
            {
                DecodeAsTypeResult::Decoded(Ok((*input, type_id)))
            }
        }

        let decoded =
            decode_with_visitor(&mut &*input_encoded, ty_id, &types, BailOutVisitor).unwrap();
        assert_eq!(decoded, (&*input_encoded, ty_id));

        // We can also use this functionality to "fall-back" to a Decode impl
        // (though obviously with the caveat that this may be incorrect).
        struct CodecDecodeVisitor<T>(core::marker::PhantomData<T>);
        impl<T: codec::Decode> Visitor for CodecDecodeVisitor<T> {
            type Value<'scale, 'resolver> = T;
            type Error = DecodeError;
            type TypeResolver = PortableRegistry;

            fn unchecked_decode_as_type<'scale, 'resolver>(
                self,
                input: &mut &'scale [u8],
                _type_id: TypeIdFor<Self>,
                _types: &'resolver scale_info::PortableRegistry,
            ) -> DecodeAsTypeResult<Self, Result<Self::Value<'scale, 'resolver>, Self::Error>>
            {
                DecodeAsTypeResult::Decoded(T::decode(input).map_err(|e| e.into()))
            }
        }

        let decoded: (String, String) = decode_with_visitor(
            &mut &*input_encoded,
            ty_id,
            &types,
            CodecDecodeVisitor(core::marker::PhantomData),
        )
        .unwrap();
        assert_eq!(decoded, ("hello".to_string(), "world".to_string()));
    }

    // A couple of tests to check that invalid input doesn't lead to panics
    // when we attempt to decode it to certain types.
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn invalid_strings_dont_panic(bytes in any::<Vec<u8>>()) {
                let (id, types) = make_type::<String>();
                let _ = decode_with_visitor(&mut &*bytes, id, &types, ValueVisitor::new());
            }

            #[test]
            fn invalid_bitvecs_dont_panic(bytes in any::<Vec<u8>>()) {
                use bitvec::{
                    vec::BitVec,
                    order::{Lsb0, Msb0},
                };

                let (id, types) = make_type::<BitVec<u8,Lsb0>>();
                let _ = decode_with_visitor(&mut &*bytes, id, &types, ValueVisitor::new());

                let (id, types) = make_type::<BitVec<u8,Msb0>>();
                let _ = decode_with_visitor(&mut &*bytes, id, &types, ValueVisitor::new());

                let (id, types) = make_type::<BitVec<u32,Lsb0>>();
                let _ = decode_with_visitor(&mut &*bytes, id, &types, ValueVisitor::new());

                let (id, types) = make_type::<BitVec<u32,Msb0>>();
                let _ = decode_with_visitor(&mut &*bytes, id, &types, ValueVisitor::new());
            }
        }
    }
}
