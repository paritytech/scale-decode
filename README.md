# scale-decode

This crate attempts to simplify the process of decoding SCALE encoded bytes into a custom data structure
given a type registry (from `scale-info`), a type ID that you'd like to decode the bytes into, and a `Visitor`
implementation which determines how you'd like to map the decoded values onto your own custom type.

The crate attempts to avoid any allocations in the `decode` function, so that the only allocations introduced
are those that are part of your `Visitor` implementation.

Here's an example of implementing `Visitor` to decode bytes into a custom `Value` type:

```rust
use scale_decode::visitor;

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
    Composite(Vec<(Option<String>, Value)>),
    Tuple(Vec<Value>),
    Str(String),
    Array(Vec<Value>),
    Variant(String, Vec<(Option<String>, Value)>),
    BitSequence(visitor::types::BitSequenceValue),
}

struct ValueVisitor;
impl visitor::Visitor for ValueVisitor {
    type Value = Value;
    type Error = visitor::DecodeError;

    fn visit_bool(self, value: bool) -> Result<Self::Value, Self::Error> {
        Ok(Value::Bool(value))
    }
    fn visit_char(self, value: char) -> Result<Self::Value, Self::Error> {
        Ok(Value::Char(value))
    }
    fn visit_u8(self, value: u8) -> Result<Self::Value, Self::Error> {
        Ok(Value::U8(value))
    }
    fn visit_u16(self, value: u16) -> Result<Self::Value, Self::Error> {
        Ok(Value::U16(value))
    }
    fn visit_u32(self, value: u32) -> Result<Self::Value, Self::Error> {
        Ok(Value::U32(value))
    }
    fn visit_u64(self, value: u64) -> Result<Self::Value, Self::Error> {
        Ok(Value::U64(value))
    }
    fn visit_u128(self, value: u128) -> Result<Self::Value, Self::Error> {
        Ok(Value::U128(value))
    }
    fn visit_u256(self, value: &[u8; 32]) -> Result<Self::Value, Self::Error> {
        Ok(Value::U256(*value))
    }
    fn visit_i8(self, value: i8) -> Result<Self::Value, Self::Error> {
        Ok(Value::I8(value))
    }
    fn visit_i16(self, value: i16) -> Result<Self::Value, Self::Error> {
        Ok(Value::I16(value))
    }
    fn visit_i32(self, value: i32) -> Result<Self::Value, Self::Error> {
        Ok(Value::I32(value))
    }
    fn visit_i64(self, value: i64) -> Result<Self::Value, Self::Error> {
        Ok(Value::I64(value))
    }
    fn visit_i128(self, value: i128) -> Result<Self::Value, Self::Error> {
        Ok(Value::I128(value))
    }
    fn visit_i256(self, value: &[u8; 32]) -> Result<Self::Value, Self::Error> {
        Ok(Value::I256(*value))
    }
    fn visit_compact_u8(self, value: u8) -> Result<Self::Value, Self::Error> {
        Ok(Value::CompactU8(value))
    }
    fn visit_compact_u16(self, value: u16) -> Result<Self::Value, Self::Error> {
        Ok(Value::CompactU16(value))
    }
    fn visit_compact_u32(self, value: u32) -> Result<Self::Value, Self::Error> {
        Ok(Value::CompactU32(value))
    }
    fn visit_compact_u64(self, value: u64) -> Result<Self::Value, Self::Error> {
        Ok(Value::CompactU64(value))
    }
    fn visit_compact_u128(self, value: u128) -> Result<Self::Value, Self::Error> {
        Ok(Value::CompactU128(value))
    }
    fn visit_sequence(self, value: &mut visitor::types::Sequence<'_>) -> Result<Self::Value, Self::Error> {
        let mut vals = vec![];
        while let Some(val) = value.decode_item(ValueVisitor)? {
            vals.push(val);
        }
        Ok(Value::Sequence(vals))
    }
    fn visit_composite(self, value: &mut visitor::types::Composite<'_>) -> Result<Self::Value, Self::Error> {
        let mut vals = vec![];
        while let Some((name, val)) = value.decode_item(ValueVisitor)? {
            vals.push((name.map(|s| s.to_owned()), val));
        }
        Ok(Value::Composite(vals))
    }
    fn visit_tuple(self, value: &mut visitor::types::Tuple<'_>) -> Result<Self::Value, Self::Error> {
        let mut vals = vec![];
        while let Some(val) = value.decode_item(ValueVisitor)? {
            vals.push(val);
        }
        Ok(Value::Tuple(vals))
    }
    fn visit_str(self, value: &visitor::types::Str<'_>) -> Result<Self::Value, Self::Error> {
        Ok(Value::Str(value.as_str()?.to_owned()))
    }
    fn visit_variant(self, value: &mut visitor::types::Variant<'_>) -> Result<Self::Value, Self::Error> {
        let mut vals = vec![];
        while let Some((name, val)) = value.decode_item(ValueVisitor)? {
            vals.push((name.map(|s| s.to_owned()), val));
        }
        Ok(Value::Variant(value.name().to_owned(), vals))
    }
    fn visit_array(self, value: &mut visitor::types::Array<'_>) -> Result<Self::Value, Self::Error> {
        let mut vals = vec![];
        while let Some(val) = value.decode_item(ValueVisitor)? {
            vals.push(val);
        }
        Ok(Value::Array(vals))
    }
    fn visit_bitsequence(self, value: &mut visitor::types::BitSequence<'_>) -> Result<Self::Value, Self::Error> {
        Ok(Value::BitSequence(value.decode_bitsequence()?))
    }
}
```

This can then be passed to a decode function like so:

```rust
let value: Value = scale_decode::decode(scale_bytes, type_id, types, ValueVisitor)?;
```

Where `scale_bytes` are the bytes you'd like to decode, `type_id` is the type stored in the `types` registry
that you'd like to try and decode the bytes into, and `types` is a `scale_info::PortableRegistry` containing
information about the various types in use.