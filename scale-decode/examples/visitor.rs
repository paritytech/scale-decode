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

use codec::Encode;
use scale_decode::visitor::{
    self,
    types::{Array, BitSequence, Compact, Composite, Sequence, Str, Tuple, Variant},
    TypeId,
};

// A custom type we'd like to decode into:
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
    CompactU8(u8),
    CompactU16(u16),
    CompactU32(u32),
    CompactU64(u64),
    CompactU128(u128),
    Sequence(Vec<Value>),
    Composite(Vec<(String, Value)>),
    Tuple(Vec<Value>),
    Str(String),
    Array(Vec<Value>),
    Variant(String, Vec<(String, Value)>),
    BitSequence(scale_bits::Bits),
}

// Implement the `Visitor` trait to define how to go from SCALE
// values into this type:
struct ValueVisitor;
impl visitor::Visitor for ValueVisitor {
    type Value<'scale> = Value;
    type Error = visitor::DecodeError;

    fn visit_bool<'scale>(
        self,
        value: bool,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::Bool(value))
    }
    fn visit_char<'scale>(
        self,
        value: char,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::Char(value))
    }
    fn visit_u8<'scale>(
        self,
        value: u8,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::U8(value))
    }
    fn visit_u16<'scale>(
        self,
        value: u16,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::U16(value))
    }
    fn visit_u32<'scale>(
        self,
        value: u32,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::U32(value))
    }
    fn visit_u64<'scale>(
        self,
        value: u64,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::U64(value))
    }
    fn visit_u128<'scale>(
        self,
        value: u128,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::U128(value))
    }
    fn visit_u256<'scale>(
        self,
        value: &'scale [u8; 32],
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::U256(*value))
    }
    fn visit_i8<'scale>(
        self,
        value: i8,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::I8(value))
    }
    fn visit_i16<'scale>(
        self,
        value: i16,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::I16(value))
    }
    fn visit_i32<'scale>(
        self,
        value: i32,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::I32(value))
    }
    fn visit_i64<'scale>(
        self,
        value: i64,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::I64(value))
    }
    fn visit_i128<'scale>(
        self,
        value: i128,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::I128(value))
    }
    fn visit_i256<'scale>(
        self,
        value: &'scale [u8; 32],
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::I256(*value))
    }
    fn visit_compact_u8<'scale>(
        self,
        value: Compact<u8>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::CompactU8(value.value()))
    }
    fn visit_compact_u16<'scale>(
        self,
        value: Compact<u16>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::CompactU16(value.value()))
    }
    fn visit_compact_u32<'scale>(
        self,
        value: Compact<u32>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::CompactU32(value.value()))
    }
    fn visit_compact_u64<'scale>(
        self,
        value: Compact<u64>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::CompactU64(value.value()))
    }
    fn visit_compact_u128<'scale>(
        self,
        value: Compact<u128>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::CompactU128(value.value()))
    }
    fn visit_sequence<'scale>(
        self,
        value: &mut Sequence<'scale, '_>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        let mut vals = vec![];
        while let Some(val) = value.decode_item(ValueVisitor)? {
            vals.push(val);
        }
        Ok(Value::Sequence(vals))
    }
    fn visit_composite<'scale>(
        self,
        value: &mut Composite<'scale, '_>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        let mut vals = vec![];
        while let Some((name, val)) = value.decode_item_with_name(ValueVisitor)? {
            vals.push((name.to_owned(), val));
        }
        Ok(Value::Composite(vals))
    }
    fn visit_tuple<'scale>(
        self,
        value: &mut Tuple<'scale, '_>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        let mut vals = vec![];
        while let Some(val) = value.decode_item(ValueVisitor)? {
            vals.push(val);
        }
        Ok(Value::Tuple(vals))
    }
    fn visit_str<'scale>(
        self,
        value: &mut Str<'scale>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        Ok(Value::Str(value.as_str()?.to_owned()))
    }
    fn visit_variant<'scale>(
        self,
        value: &mut Variant<'scale, '_>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        let mut vals = vec![];
        let fields = value.fields();
        while let Some((name, val)) = fields.decode_item_with_name(ValueVisitor)? {
            vals.push((name.to_owned(), val));
        }
        Ok(Value::Variant(value.name().to_owned(), vals))
    }
    fn visit_array<'scale>(
        self,
        value: &mut Array<'scale, '_>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        let mut vals = vec![];
        while let Some(val) = value.decode_item(ValueVisitor)? {
            vals.push(val);
        }
        Ok(Value::Array(vals))
    }
    fn visit_bitsequence<'scale>(
        self,
        value: &mut BitSequence<'scale>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        let bools: Result<scale_bits::Bits, _> = value.decode()?.collect();
        Ok(Value::BitSequence(bools?))
    }
}

// Now we can decode arbitratry SCALE encoded types into these values (provided we have
// a type registry to hand)..
fn main() {
    // Some type that we want to decode from:
    #[derive(Encode, scale_info::TypeInfo)]
    enum MyEnum {
        Bar { hi: String, other: u128 },
    }

    // Make a type registry since we don't have one to hand (if you're working with things
    // like substrate based nodes, this will be a part of the metadata you can obtain from it),
    // so the static type information above may not necessarily be available when decoding:
    let (type_id, types) = make_type::<MyEnum>();

    // SCALE encode our type:
    let bytes = MyEnum::Bar { hi: "hello".to_string(), other: 123 }.encode();

    // Use scale_decode + type information to decode these bytes into our Value type:
    assert_eq!(
        scale_decode::visitor::decode_with_visitor(&mut &*bytes, type_id, &types, ValueVisitor)
            .unwrap(),
        Value::Variant(
            "Bar".to_owned(),
            vec![
                ("hi".to_string(), Value::Str("hello".to_string())),
                ("other".to_string(), Value::U128(123)),
            ],
        )
    )
}

// Normally we'd have a type registry to hand already, but if not, we can build our own:
fn make_type<T: scale_info::TypeInfo + 'static>() -> (u32, scale_info::PortableRegistry) {
    let m = scale_info::MetaType::new::<T>();
    let mut types = scale_info::Registry::new();
    let id = types.register_type(&m);
    let portable_registry: scale_info::PortableRegistry = types.into();

    (id.id(), portable_registry)
}
