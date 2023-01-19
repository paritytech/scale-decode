// Copyright (C) 2022 Parity Technologies (UK) Ltd. (admin@parity.io)
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

//! The [`Visitor`] trait and associated types.

mod array;
mod bit_sequence;
mod compact;
mod composite;
mod decode;
mod sequence;
mod str;
mod tuple;
mod variant;

use scale_info::form::PortableForm;

pub use self::str::Str;
pub use array::Array;
pub use bit_sequence::BitSequence;
pub use compact::{Compact, CompactLocation};
pub use composite::Composite;
pub use decode::decode_with_visitor;
pub use sequence::Sequence;
pub use tuple::Tuple;
pub use variant::Variant;

/// An implementation of the [`Visitor`] trait can be passed to the [`crate::decode()`]
/// function, and is handed back values as they are encountered. It's up to the implementation
/// to decide what to do with these values.
pub trait Visitor: Sized {
    /// The type of the value to hand back from the [`crate::decode()`] function.
    type Value;
    /// The error type (which we must be able to convert [`DecodeError`]s into, to
    /// handle any internal errors that crop up trying to decode things).
    type Error: From<DecodeError>;

    /// Called when a bool is seen in the input bytes.
    fn visit_bool(self, value: bool, type_id: TypeId) -> Result<Self::Value, Self::Error>;
    /// Called when a bool is seen in the input bytes.
    fn visit_char(self, value: char, type_id: TypeId) -> Result<Self::Value, Self::Error>;
    /// Called when a u8 is seen in the input bytes.
    fn visit_u8(self, value: u8, type_id: TypeId) -> Result<Self::Value, Self::Error>;
    /// Called when a u16 is seen in the input bytes.
    fn visit_u16(self, value: u16, type_id: TypeId) -> Result<Self::Value, Self::Error>;
    /// Called when a u32 is seen in the input bytes.
    fn visit_u32(self, value: u32, type_id: TypeId) -> Result<Self::Value, Self::Error>;
    /// Called when a u64 is seen in the input bytes.
    fn visit_u64(self, value: u64, type_id: TypeId) -> Result<Self::Value, Self::Error>;
    /// Called when a u128 is seen in the input bytes.
    fn visit_u128(self, value: u128, type_id: TypeId) -> Result<Self::Value, Self::Error>;
    /// Called when a u256 is seen in the input bytes.
    fn visit_u256(self, value: &[u8; 32], type_id: TypeId) -> Result<Self::Value, Self::Error>;
    /// Called when an i8 is seen in the input bytes.
    fn visit_i8(self, value: i8, type_id: TypeId) -> Result<Self::Value, Self::Error>;
    /// Called when an i16 is seen in the input bytes.
    fn visit_i16(self, value: i16, type_id: TypeId) -> Result<Self::Value, Self::Error>;
    /// Called when an i32 is seen in the input bytes.
    fn visit_i32(self, value: i32, type_id: TypeId) -> Result<Self::Value, Self::Error>;
    /// Called when an i64 is seen in the input bytes.
    fn visit_i64(self, value: i64, type_id: TypeId) -> Result<Self::Value, Self::Error>;
    /// Called when an i128 is seen in the input bytes.
    fn visit_i128(self, value: i128, type_id: TypeId) -> Result<Self::Value, Self::Error>;
    /// Called when an i256 is seen in the input bytes.
    fn visit_i256(self, value: &[u8; 32], type_id: TypeId) -> Result<Self::Value, Self::Error>;
    /// Called when a sequence of values is seen in the input bytes.
    fn visit_sequence(
        self,
        value: &mut Sequence,
        type_id: TypeId,
    ) -> Result<Self::Value, Self::Error>;
    /// Called when a composite value is seen in the input bytes.
    fn visit_composite(
        self,
        value: &mut Composite,
        type_id: TypeId,
    ) -> Result<Self::Value, Self::Error>;
    /// Called when a tuple of values is seen in the input bytes.
    fn visit_tuple(self, value: &mut Tuple, type_id: TypeId) -> Result<Self::Value, Self::Error>;
    /// Called when a string value is seen in the input bytes.
    fn visit_str(self, value: Str, type_id: TypeId) -> Result<Self::Value, Self::Error>;
    /// Called when a variant is seen in the input bytes.
    fn visit_variant(
        self,
        value: &mut Variant,
        type_id: TypeId,
    ) -> Result<Self::Value, Self::Error>;
    /// Called when an array is seen in the input bytes.
    fn visit_array(self, value: &mut Array, type_id: TypeId) -> Result<Self::Value, Self::Error>;
    /// Called when a bit sequence is seen in the input bytes.
    fn visit_bitsequence(
        self,
        value: &mut BitSequence,
        type_id: TypeId,
    ) -> Result<Self::Value, Self::Error>;

    // Default implementations for visiting compact values just delegate and
    // ignore the compactness, but they are here if decoders would like to know
    // that the thing was compact encoded:

    /// Called when a compact encoded u8 is seen in the input bytes.
    fn visit_compact_u8(
        self,
        value: Compact<u8>,
        type_id: TypeId,
    ) -> Result<Self::Value, Self::Error> {
        self.visit_u8(value.value(), type_id)
    }
    /// Called when a compact encoded u16 is seen in the input bytes.
    fn visit_compact_u16(
        self,
        value: Compact<u16>,
        type_id: TypeId,
    ) -> Result<Self::Value, Self::Error> {
        self.visit_u16(value.value(), type_id)
    }
    /// Called when a compact encoded u32 is seen in the input bytes.
    fn visit_compact_u32(
        self,
        value: Compact<u32>,
        type_id: TypeId,
    ) -> Result<Self::Value, Self::Error> {
        self.visit_u32(value.value(), type_id)
    }
    /// Called when a compact encoded u64 is seen in the input bytes.
    fn visit_compact_u64(
        self,
        value: Compact<u64>,
        type_id: TypeId,
    ) -> Result<Self::Value, Self::Error> {
        self.visit_u64(value.value(), type_id)
    }
    /// Called when a compact encoded u128 is seen in the input bytes.
    fn visit_compact_u128(
        self,
        value: Compact<u128>,
        type_id: TypeId,
    ) -> Result<Self::Value, Self::Error> {
        self.visit_u128(value.value(), type_id)
    }
}

/// An error decoding SCALE bytes.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum DecodeError {
    /// We ran into an error trying to decode a bit sequence.
    #[error("Cannot decode bit sequence: {0}")]
    BitSequenceError(#[from] BitSequenceError),
    /// The type we're trying to decode is supposed to be compact encoded, but that is not possible.
    #[error("Could not decode compact encoded type into {0:?}")]
    CannotDecodeCompactIntoType(scale_info::Type<PortableForm>),
    /// Failure to decode bytes into a string.
    #[error("Could not decode string: {0}")]
    InvalidStr(#[from] std::str::Utf8Error),
    /// We could not convert the [`u32`] that we found into a valid [`char`].
    #[error("{0} is expected to be a valid char, but is not")]
    InvalidChar(u32),
    /// We expected more bytes to finish decoding, but could not find them.
    #[error("Ran out of data during decoding")]
    Eof,
    /// We found a variant that does not match with any in the type we're trying to decode from.
    #[error("Could not find variant with index {0} in {1:?}")]
    VariantNotFound(u8, scale_info::TypeDefVariant<PortableForm>),
    /// Some error emitted from a [`codec::Decode`] impl.
    #[error("{0}")]
    CodecError(#[from] codec::Error),
    /// We could not find the type given in the type registry provided.
    #[error("Cannot find type with ID {0}")]
    TypeIdNotFound(u32),
    /// You hit this is you try to decode a field from an
    #[error("No fields left to decode")]
    NothingLeftToDecode,
}

/// An error that can occur trying to decode a bit sequence.
pub type BitSequenceError = scale_bits::scale::format::FromMetadataError;

/// The ID of the type being decoded.
#[derive(Clone, Copy, Debug, Default)]
pub struct TypeId(pub u32);

/// A [`Visitor`] implementation that just ignores all of the bytes.
pub struct IgnoreVisitor;

impl Visitor for IgnoreVisitor {
    type Value = ();
    type Error = DecodeError;

    fn visit_bool(self, _value: bool, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_char(self, _value: char, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_u8(self, _value: u8, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_u16(self, _value: u16, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_u32(self, _value: u32, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_u64(self, _value: u64, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_u128(self, _value: u128, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_u256(self, _value: &[u8; 32], _type_id: TypeId) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_i8(self, _value: i8, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_i16(self, _value: i16, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_i32(self, _value: i32, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_i64(self, _value: i64, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_i128(self, _value: i128, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_i256(self, _value: &[u8; 32], _type_id: TypeId) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_sequence(
        self,
        _value: &mut Sequence,
        _type_id: TypeId,
    ) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_composite(
        self,
        _value: &mut Composite,
        _type_id: TypeId,
    ) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_tuple(self, _value: &mut Tuple, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_str(self, _value: Str, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_array(self, _value: &mut Array, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_variant(
        self,
        _value: &mut Variant,
        _type_id: TypeId,
    ) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_bitsequence(
        self,
        _value: &mut BitSequence,
        _type_id: TypeId,
    ) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use crate::visitor::TypeId;

    use super::*;
    use codec::{self, Encode};
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
        CompactU8(Vec<Loc>, u8),
        CompactU16(Vec<Loc>, u16),
        CompactU32(Vec<Loc>, u32),
        CompactU64(Vec<Loc>, u64),
        CompactU128(Vec<Loc>, u128),
        Sequence(Vec<Value>),
        Composite(Vec<(String, Value)>),
        Tuple(Vec<Value>),
        Str(String),
        Array(Vec<Value>),
        Variant(String, Vec<(String, Value)>),
        BitSequence(scale_bits::Bits),
    }

    #[derive(Clone, Debug, PartialEq)]
    enum Loc {
        Unnamed,
        Named(String),
        Primitive,
    }

    impl<'a> From<CompactLocation<'a>> for Loc {
        fn from(l: CompactLocation) -> Self {
            match l {
                CompactLocation::UnnamedComposite(_) => Loc::Unnamed,
                CompactLocation::NamedComposite(_, s) => Loc::Named(s.to_owned()),
                CompactLocation::Primitive(_) => Loc::Primitive,
            }
        }
    }

    struct ValueVisitor;
    impl Visitor for ValueVisitor {
        type Value = Value;
        type Error = DecodeError;

        fn visit_bool(self, value: bool, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
            Ok(Value::Bool(value))
        }
        fn visit_char(self, value: char, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
            Ok(Value::Char(value))
        }
        fn visit_u8(self, value: u8, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
            Ok(Value::U8(value))
        }
        fn visit_u16(self, value: u16, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
            Ok(Value::U16(value))
        }
        fn visit_u32(self, value: u32, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
            Ok(Value::U32(value))
        }
        fn visit_u64(self, value: u64, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
            Ok(Value::U64(value))
        }
        fn visit_u128(self, value: u128, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
            Ok(Value::U128(value))
        }
        fn visit_u256(
            self,
            value: &[u8; 32],
            _type_id: TypeId,
        ) -> Result<Self::Value, Self::Error> {
            Ok(Value::U256(*value))
        }
        fn visit_i8(self, value: i8, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
            Ok(Value::I8(value))
        }
        fn visit_i16(self, value: i16, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
            Ok(Value::I16(value))
        }
        fn visit_i32(self, value: i32, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
            Ok(Value::I32(value))
        }
        fn visit_i64(self, value: i64, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
            Ok(Value::I64(value))
        }
        fn visit_i128(self, value: i128, _type_id: TypeId) -> Result<Self::Value, Self::Error> {
            Ok(Value::I128(value))
        }
        fn visit_i256(
            self,
            value: &[u8; 32],
            _type_id: TypeId,
        ) -> Result<Self::Value, Self::Error> {
            Ok(Value::I256(*value))
        }
        fn visit_compact_u8(
            self,
            value: Compact<u8>,
            _type_id: TypeId,
        ) -> Result<Self::Value, Self::Error> {
            let locs = value.locations().iter().map(|&l| l.into()).collect();
            Ok(Value::CompactU8(locs, value.value()))
        }
        fn visit_compact_u16(
            self,
            value: Compact<u16>,
            _type_id: TypeId,
        ) -> Result<Self::Value, Self::Error> {
            let locs = value.locations().iter().map(|&l| l.into()).collect();
            Ok(Value::CompactU16(locs, value.value()))
        }
        fn visit_compact_u32(
            self,
            value: Compact<u32>,
            _type_id: TypeId,
        ) -> Result<Self::Value, Self::Error> {
            let locs = value.locations().iter().map(|&l| l.into()).collect();
            Ok(Value::CompactU32(locs, value.value()))
        }
        fn visit_compact_u64(
            self,
            value: Compact<u64>,
            _type_id: TypeId,
        ) -> Result<Self::Value, Self::Error> {
            let locs = value.locations().iter().map(|&l| l.into()).collect();
            Ok(Value::CompactU64(locs, value.value()))
        }
        fn visit_compact_u128(
            self,
            value: Compact<u128>,
            _type_id: TypeId,
        ) -> Result<Self::Value, Self::Error> {
            let locs = value.locations().iter().map(|&l| l.into()).collect();
            Ok(Value::CompactU128(locs, value.value()))
        }
        fn visit_sequence(
            self,
            value: &mut Sequence,
            _type_id: TypeId,
        ) -> Result<Self::Value, Self::Error> {
            let mut vals = vec![];
            while let Some(val) = value.decode_item(ValueVisitor)? {
                vals.push(val);
            }
            Ok(Value::Sequence(vals))
        }
        fn visit_composite(
            self,
            value: &mut Composite,
            _type_id: TypeId,
        ) -> Result<Self::Value, Self::Error> {
            let mut vals = vec![];
            while let Some((name, val)) = value.decode_item_with_name(ValueVisitor)? {
                vals.push((name.to_owned(), val));
            }
            Ok(Value::Composite(vals))
        }
        fn visit_tuple(
            self,
            value: &mut Tuple,
            _type_id: TypeId,
        ) -> Result<Self::Value, Self::Error> {
            let mut vals = vec![];
            while let Some(val) = value.decode_item(ValueVisitor)? {
                vals.push(val);
            }
            Ok(Value::Tuple(vals))
        }
        fn visit_str(
            self,
            value: Str,
            _type_id: TypeId,
        ) -> Result<Self::Value, Self::Error> {
            Ok(Value::Str(value.as_str()?.to_owned()))
        }
        fn visit_variant(
            self,
            value: &mut Variant,
            _type_id: TypeId,
        ) -> Result<Self::Value, Self::Error> {
            let mut vals = vec![];
            let fields = value.fields();
            while let Some((name, val)) = fields.decode_item_with_name(ValueVisitor)? {
                vals.push((name.to_owned(), val));
            }
            Ok(Value::Variant(value.name().to_owned(), vals))
        }
        fn visit_array(
            self,
            value: &mut Array,
            _type_id: TypeId,
        ) -> Result<Self::Value, Self::Error> {
            let mut vals = vec![];
            while let Some(val) = value.decode_item(ValueVisitor)? {
                vals.push(val);
            }
            Ok(Value::Array(vals))
        }
        fn visit_bitsequence(
            self,
            value: &mut BitSequence,
            _type_id: TypeId,
        ) -> Result<Self::Value, Self::Error> {
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

        (id.id(), portable_registry)
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
        encode_decode_check(codec::Compact(123u8), Value::CompactU8(vec![Loc::Primitive], 123));
        encode_decode_check(codec::Compact(123u16), Value::CompactU16(vec![Loc::Primitive], 123));
        encode_decode_check(codec::Compact(123u32), Value::CompactU32(vec![Loc::Primitive], 123));
        encode_decode_check(codec::Compact(123u64), Value::CompactU64(vec![Loc::Primitive], 123));
        encode_decode_check(codec::Compact(123u128), Value::CompactU128(vec![Loc::Primitive], 123));
        encode_decode_check(true, Value::Bool(true));
        encode_decode_check(false, Value::Bool(false));
        encode_decode_check_explicit_info::<char, _>('c' as u32, Value::Char('c'));
        encode_decode_check("Hello there", Value::Str("Hello there".to_owned()));
        encode_decode_check("Hello there".to_string(), Value::Str("Hello there".to_owned()));
    }

    #[test]
    fn decode_compact_named_wrapper_struct() {
        // A struct that can be compact encoded:
        #[derive(Encode, scale_info::TypeInfo)]
        struct MyWrapper {
            inner: u32,
        }
        impl From<codec::Compact<MyWrapper>> for MyWrapper {
            fn from(val: codec::Compact<MyWrapper>) -> MyWrapper {
                val.0
            }
        }
        impl codec::CompactAs for MyWrapper {
            type As = u32;

            fn encode_as(&self) -> &Self::As {
                &self.inner
            }
            fn decode_from(inner: Self::As) -> Result<Self, codec::Error> {
                Ok(MyWrapper { inner })
            }
        }

        encode_decode_check(
            codec::Compact(MyWrapper { inner: 123 }),
            // Currently we ignore any composite types and just give back
            // the compact value directly:
            Value::CompactU32(vec![Loc::Named("inner".to_owned()), Loc::Primitive], 123),
        );
    }

    #[test]
    fn decode_compact_unnamed_wrapper_struct() {
        // A struct that can be compact encoded:
        #[derive(Encode, scale_info::TypeInfo)]
        struct MyWrapper(u32);
        impl From<codec::Compact<MyWrapper>> for MyWrapper {
            fn from(val: codec::Compact<MyWrapper>) -> MyWrapper {
                val.0
            }
        }
        impl codec::CompactAs for MyWrapper {
            type As = u32;

            // Node the requirement to return something with a lifetime tied
            // to self here. This means that we can't implement this for things
            // more complex than wrapper structs (eg `Foo(u32,u32,u32,u32)`) without
            // shenanigans, meaning that (hopefully) supporting wrapper struct
            // decoding and nothing fancier is sufficient.
            fn encode_as(&self) -> &Self::As {
                &self.0
            }
            fn decode_from(inner: Self::As) -> Result<Self, codec::Error> {
                Ok(MyWrapper(inner))
            }
        }

        encode_decode_check(
            codec::Compact(MyWrapper(123)),
            // Currently we ignore any composite types and just give back
            // the compact value directly:
            Value::CompactU32(vec![Loc::Unnamed, Loc::Primitive], 123),
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
}