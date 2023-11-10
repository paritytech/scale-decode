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

use scale_info::form::PortableForm;
use types::*;

pub use decode::decode_with_visitor;
pub(crate) use decode::decode_with_visitor_maybe_compact;

/// An implementation of the [`Visitor`] trait can be passed to the [`decode_with_visitor()`]
/// function, and is handed back values as they are encountered. It's up to the implementation
/// to decide what to do with these values.
pub trait Visitor: Sized {
    /// The type of the value to hand back from the [`decode_with_visitor()`] function.
    type Value<'scale, 'info>;

    /// The error type that will be returned when things go wrong. This can be an arbitrary type, but:
    /// - [`DecodeError`]s must be convertible into it (this allows the default implementations to work)
    /// - It must be convertible into our root [`crate::Error`] (this makes it easy for visitors to
    ///   implement [`crate::DecodeAsType`], and works around limitations in the rust trait system).
    // Dev note: Ideally, this `Into<crate::Error>` restriction would only exist on the `DecodeAsType`
    // trait, but there's no nice way to do this _and_ have `DecodeAsType` automatically imply these
    // things (the user would have to duplicate an `Error: From<Visitor::Error>` type bound everywhere).
    type Error: From<DecodeError> + Into<crate::Error>;

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
    fn unchecked_decode_as_type<'scale, 'info>(
        self,
        _input: &mut &'scale [u8],
        _type_id: TypeId,
        _types: &'info scale_info::PortableRegistry,
    ) -> DecodeAsTypeResult<Self, Result<Self::Value<'scale, 'info>, Self::Error>> {
        DecodeAsTypeResult::Skipped(self)
    }

    /// This is called when a visitor function that you've not provided an implementation is called.
    /// You are provided an enum value corresponding to the function call, and can decide what to return
    /// in this case. The default is to return an error to announce the unexpected value.
    fn visit_unexpected<'scale, 'info>(
        self,
        unexpected: Unexpected,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        Err(DecodeError::Unexpected(unexpected).into())
    }

    /// Called when a bool is seen in the input bytes.
    fn visit_bool<'scale, 'info>(
        self,
        _value: bool,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::Bool)
    }
    /// Called when a char is seen in the input bytes.
    fn visit_char<'scale, 'info>(
        self,
        _value: char,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::Char)
    }
    /// Called when a u8 is seen in the input bytes.
    fn visit_u8<'scale, 'info>(
        self,
        _value: u8,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::U8)
    }
    /// Called when a u16 is seen in the input bytes.
    fn visit_u16<'scale, 'info>(
        self,
        _value: u16,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::U16)
    }
    /// Called when a u32 is seen in the input bytes.
    fn visit_u32<'scale, 'info>(
        self,
        _value: u32,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::U32)
    }
    /// Called when a u64 is seen in the input bytes.
    fn visit_u64<'scale, 'info>(
        self,
        _value: u64,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::U64)
    }
    /// Called when a u128 is seen in the input bytes.
    fn visit_u128<'scale, 'info>(
        self,
        _value: u128,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::U128)
    }
    /// Called when a u256 is seen in the input bytes.
    fn visit_u256<'info>(
        self,
        _value: &'_ [u8; 32],
        _type_id: TypeId,
    ) -> Result<Self::Value<'_, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::U256)
    }
    /// Called when an i8 is seen in the input bytes.
    fn visit_i8<'scale, 'info>(
        self,
        _value: i8,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::I8)
    }
    /// Called when an i16 is seen in the input bytes.
    fn visit_i16<'scale, 'info>(
        self,
        _value: i16,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::I16)
    }
    /// Called when an i32 is seen in the input bytes.
    fn visit_i32<'scale, 'info>(
        self,
        _value: i32,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::I32)
    }
    /// Called when an i64 is seen in the input bytes.
    fn visit_i64<'scale, 'info>(
        self,
        _value: i64,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::I64)
    }
    /// Called when an i128 is seen in the input bytes.
    fn visit_i128<'scale, 'info>(
        self,
        _value: i128,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::I128)
    }
    /// Called when an i256 is seen in the input bytes.
    fn visit_i256<'info>(
        self,
        _value: &'_ [u8; 32],
        _type_id: TypeId,
    ) -> Result<Self::Value<'_, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::I256)
    }
    /// Called when a sequence of values is seen in the input bytes.
    fn visit_sequence<'scale, 'info>(
        self,
        _value: &mut Sequence<'scale, 'info>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::Sequence)
    }
    /// Called when a composite value is seen in the input bytes.
    fn visit_composite<'scale, 'info>(
        self,
        _value: &mut Composite<'scale, 'info>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::Composite)
    }
    /// Called when a tuple of values is seen in the input bytes.
    fn visit_tuple<'scale, 'info>(
        self,
        _value: &mut Tuple<'scale, 'info>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::Tuple)
    }
    /// Called when a string value is seen in the input bytes.
    fn visit_str<'scale, 'info>(
        self,
        _value: &mut Str<'scale>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::Str)
    }
    /// Called when a variant is seen in the input bytes.
    fn visit_variant<'scale, 'info>(
        self,
        _value: &mut Variant<'scale, 'info>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::Variant)
    }
    /// Called when an array is seen in the input bytes.
    fn visit_array<'scale, 'info>(
        self,
        _value: &mut Array<'scale, 'info>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::Array)
    }
    /// Called when a bit sequence is seen in the input bytes.
    fn visit_bitsequence<'scale, 'info>(
        self,
        _value: &mut BitSequence<'scale>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.visit_unexpected(Unexpected::Bitsequence)
    }
}

/// An error decoding SCALE bytes.
#[derive(Debug, Clone, PartialEq, Eq, derive_more::From, derive_more::Display)]
pub enum DecodeError {
    /// We ran into an error trying to decode a bit sequence.
    #[from]
    #[display(fmt = "Cannot decode bit sequence: {_0}")]
    BitSequenceError(BitSequenceError),
    /// The type we're trying to decode is supposed to be compact encoded, but that is not possible.
    #[display(fmt = "Could not decode compact encoded type into {_0:?}")]
    CannotDecodeCompactIntoType(scale_info::Type<PortableForm>),
    /// Failure to decode bytes into a string.
    #[from]
    #[display(fmt = "Could not decode string: {_0}")]
    InvalidStr(alloc::str::Utf8Error),
    /// We could not convert the [`u32`] that we found into a valid [`char`].
    #[display(fmt = "{_0} is expected to be a valid char, but is not")]
    InvalidChar(u32),
    /// We expected more bytes to finish decoding, but could not find them.
    #[display(fmt = "Ran out of data during decoding")]
    NotEnoughInput,
    /// We found a variant that does not match with any in the type we're trying to decode from.
    #[display(fmt = "Could not find variant with index {_0} in {_1:?}")]
    VariantNotFound(u8, scale_info::TypeDefVariant<PortableForm>),
    /// Some error emitted from a [`codec::Decode`] impl.
    #[from]
    CodecError(codec::Error),
    /// We could not find the type given in the type registry provided.
    #[display(fmt = "Cannot find type with ID {_0}")]
    TypeIdNotFound(u32),
    /// This is returned by default if a visitor function is not implemented.
    #[display(fmt = "Unexpected type {_0}")]
    Unexpected(Unexpected),
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}

/// This is returned by default when a visitor function isn't implemented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
#[allow(missing_docs)]
pub enum Unexpected {
    #[display(fmt = "bool")]
    Bool,
    #[display(fmt = "char")]
    Char,
    #[display(fmt = "u8")]
    U8,
    #[display(fmt = "u16")]
    U16,
    #[display(fmt = "u32")]
    U32,
    #[display(fmt = "u64")]
    U64,
    #[display(fmt = "u128")]
    U128,
    #[display(fmt = "u256")]
    U256,
    #[display(fmt = "i8")]
    I8,
    #[display(fmt = "i16")]
    I16,
    #[display(fmt = "i32")]
    I32,
    #[display(fmt = "i64")]
    I64,
    #[display(fmt = "i128")]
    I128,
    #[display(fmt = "i256")]
    I256,
    #[display(fmt = "sequence")]
    Sequence,
    #[display(fmt = "composite")]
    Composite,
    #[display(fmt = "tuple")]
    Tuple,
    #[display(fmt = "str")]
    Str,
    #[display(fmt = "variant")]
    Variant,
    #[display(fmt = "array")]
    Array,
    #[display(fmt = "bitsequence")]
    Bitsequence,
}

/// The response from [`Visitor::unchecked_decode_as_type()`].
pub enum DecodeAsTypeResult<V, R> {
    /// Skip any manual decoding and return the visitor instead.
    Skipped(V),
    /// Some manually decoded result.
    Decoded(R),
}

/// This is implemented for visitor related types which have a `decode_item` method,
/// and allows you to generically talk about decoding unnamed items.
pub trait DecodeItemIterator<'scale, 'info> {
    /// Use a visitor to decode a single item.
    fn decode_item<V: Visitor>(
        &mut self,
        visitor: V,
    ) -> Option<Result<V::Value<'scale, 'info>, V::Error>>;
}

/// An error that can occur trying to decode a bit sequence.
pub type BitSequenceError = scale_bits::scale::format::FromMetadataError;

/// The ID of the type being decoded.
#[derive(Clone, Copy, Debug, Default)]
pub struct TypeId(pub u32);

/// A [`Visitor`] implementation that just ignores all of the bytes.
pub struct IgnoreVisitor;
impl Visitor for IgnoreVisitor {
    type Value<'scale, 'info> = ();
    type Error = DecodeError;

    // Whatever the value we visit is, just ignore it.
    fn visit_unexpected<'scale, 'info>(
        self,
        _unexpected: Unexpected,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::visitor::TypeId;

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

    struct ValueVisitor;
    impl Visitor for ValueVisitor {
        type Value<'scale, 'info> = Value;
        type Error = DecodeError;

        fn visit_bool<'scale, 'info>(
            self,
            value: bool,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            Ok(Value::Bool(value))
        }
        fn visit_char<'scale, 'info>(
            self,
            value: char,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            Ok(Value::Char(value))
        }
        fn visit_u8<'scale, 'info>(
            self,
            value: u8,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            Ok(Value::U8(value))
        }
        fn visit_u16<'scale, 'info>(
            self,
            value: u16,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            Ok(Value::U16(value))
        }
        fn visit_u32<'scale, 'info>(
            self,
            value: u32,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            Ok(Value::U32(value))
        }
        fn visit_u64<'scale, 'info>(
            self,
            value: u64,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            Ok(Value::U64(value))
        }
        fn visit_u128<'scale, 'info>(
            self,
            value: u128,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            Ok(Value::U128(value))
        }
        fn visit_u256<'info>(
            self,
            value: &'_ [u8; 32],
            _type_id: TypeId,
        ) -> Result<Self::Value<'_, 'info>, Self::Error> {
            Ok(Value::U256(*value))
        }
        fn visit_i8<'scale, 'info>(
            self,
            value: i8,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            Ok(Value::I8(value))
        }
        fn visit_i16<'scale, 'info>(
            self,
            value: i16,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            Ok(Value::I16(value))
        }
        fn visit_i32<'scale, 'info>(
            self,
            value: i32,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            Ok(Value::I32(value))
        }
        fn visit_i64<'scale, 'info>(
            self,
            value: i64,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            Ok(Value::I64(value))
        }
        fn visit_i128<'scale, 'info>(
            self,
            value: i128,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            Ok(Value::I128(value))
        }
        fn visit_i256<'info>(
            self,
            value: &'_ [u8; 32],
            _type_id: TypeId,
        ) -> Result<Self::Value<'_, 'info>, Self::Error> {
            Ok(Value::I256(*value))
        }
        fn visit_sequence<'scale, 'info>(
            self,
            value: &mut Sequence<'scale, 'info>,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            let mut vals = vec![];
            while let Some(val) = value.decode_item(ValueVisitor) {
                let val = val?;
                vals.push(val);
            }
            Ok(Value::Sequence(vals))
        }
        fn visit_composite<'scale, 'info>(
            self,
            value: &mut Composite<'scale, 'info>,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            let mut vals = vec![];
            for item in value.by_ref() {
                let item = item?;
                let val = item.decode_with_visitor(ValueVisitor)?;
                let name = item.name().unwrap_or("").to_owned();
                vals.push((name, val));
            }
            Ok(Value::Composite(vals))
        }
        fn visit_tuple<'scale, 'info>(
            self,
            value: &mut Tuple<'scale, 'info>,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            let mut vals = vec![];
            while let Some(val) = value.decode_item(ValueVisitor) {
                let val = val?;
                vals.push(val);
            }
            Ok(Value::Tuple(vals))
        }
        fn visit_str<'scale, 'info>(
            self,
            value: &mut Str<'scale>,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            Ok(Value::Str(value.as_str()?.to_owned()))
        }
        fn visit_variant<'scale, 'info>(
            self,
            value: &mut Variant<'scale, 'info>,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            let mut vals = vec![];
            let fields = value.fields();
            for item in fields.by_ref() {
                let item = item?;
                let val = item.decode_with_visitor(ValueVisitor)?;
                let name = item.name().unwrap_or("").to_owned();
                vals.push((name, val));
            }
            Ok(Value::Variant(value.name().to_owned(), vals))
        }
        fn visit_array<'scale, 'info>(
            self,
            value: &mut Array<'scale, 'info>,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
            let mut vals = vec![];
            while let Some(val) = value.decode_item(ValueVisitor) {
                let val = val?;
                vals.push(val);
            }
            Ok(Value::Array(vals))
        }
        fn visit_bitsequence<'scale, 'info>(
            self,
            value: &mut BitSequence<'scale>,
            _type_id: TypeId,
        ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
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
    fn encode_decode_check_explicit_info<Ty: scale_info::TypeInfo + 'static, T: Encode>(
        val: T,
        expected: Value,
    ) {
        let encoded = val.encode();
        let (id, types) = make_type::<Ty>();
        let bytes = &mut &*encoded;
        let val = decode_with_visitor(bytes, id, &types, ValueVisitor)
            .expect("decoding should not error");

        assert_eq!(bytes.len(), 0, "Decoding should consume all bytes");
        assert_eq!(val, expected);
    }

    fn encode_decode_check<T: Encode + scale_info::TypeInfo + 'static>(val: T, expected: Value) {
        encode_decode_check_explicit_info::<T, T>(val, expected);
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
        encode_decode_check_explicit_info::<char, _>('c' as u32, Value::Char('c'));
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

    #[test]
    fn zero_copy_string_decoding() {
        let input = ("hello", "world");

        // The SCALE encoded bytes we want to zero-copy-decode from:
        let input_encoded = input.encode();

        // This can just zero-copy decode a string:
        struct ZeroCopyStrVisitor;
        impl Visitor for ZeroCopyStrVisitor {
            type Value<'scale, 'info> = &'scale str;
            type Error = DecodeError;

            fn visit_str<'scale, 'info>(
                self,
                value: &mut Str<'scale>,
                _type_id: TypeId,
            ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
                value.as_str()
            }
        }

        // This can zero-copy decode the pair of strings we have as input:
        struct ZeroCopyPairVisitor;
        impl Visitor for ZeroCopyPairVisitor {
            type Value<'scale, 'info> = (&'scale str, &'scale str);
            type Error = DecodeError;

            fn visit_tuple<'scale, 'info>(
                self,
                value: &mut Tuple<'scale, 'info>,
                _type_id: TypeId,
            ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
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
            type Value<'scale, 'info> = &'scale str;
            type Error = DecodeError;

            fn visit_str<'scale, 'info>(
                self,
                value: &mut Str<'scale>,
                _type_id: TypeId,
            ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
                value.as_str()
            }
        }

        // This zero-copy decodes a composite into map of strings:
        struct ZeroCopyMapVisitor;
        impl Visitor for ZeroCopyMapVisitor {
            type Value<'scale, 'info> = alloc::collections::BTreeMap<&'info str, &'scale str>;
            type Error = DecodeError;

            fn visit_composite<'scale, 'info>(
                self,
                value: &mut Composite<'scale, 'info>,
                _type_id: TypeId,
            ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
                let mut vals = alloc::collections::BTreeMap::<&'info str, &'scale str>::new();
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
            type Value<'scale, 'info> = (&'scale [u8], u32);
            type Error = DecodeError;

            fn unchecked_decode_as_type<'scale, 'info>(
                self,
                input: &mut &'scale [u8],
                type_id: TypeId,
                _types: &'info scale_info::PortableRegistry,
            ) -> DecodeAsTypeResult<Self, Result<Self::Value<'scale, 'info>, Self::Error>>
            {
                DecodeAsTypeResult::Decoded(Ok((*input, type_id.0)))
            }
        }

        let decoded =
            decode_with_visitor(&mut &*input_encoded, ty_id, &types, BailOutVisitor).unwrap();
        assert_eq!(decoded, (&*input_encoded, ty_id));

        // We can also use this functionality to "fall-back" to a Decode impl
        // (though obviously with the caveat that this may be incorrect).
        struct CodecDecodeVisitor<T>(core::marker::PhantomData<T>);
        impl<T: codec::Decode> Visitor for CodecDecodeVisitor<T> {
            type Value<'scale, 'info> = T;
            type Error = DecodeError;

            fn unchecked_decode_as_type<'scale, 'info>(
                self,
                input: &mut &'scale [u8],
                _type_id: TypeId,
                _types: &'info scale_info::PortableRegistry,
            ) -> DecodeAsTypeResult<Self, Result<Self::Value<'scale, 'info>, Self::Error>>
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
}
