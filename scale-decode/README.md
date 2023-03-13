# scale-decode

This crate makes it easy to decode SCALE encoded bytes into a custom data structure with the help of `scale_info` types.
By using this type information to guide decoding (instead of just trying to decode bytes based on the shape of the target type),
it's possible to be much more flexible in how data is decoded and mapped to some target type.

The main trait used to decode types is a `Visitor` trait (example below). By implementing this trait, you can describe how to
take SCALE decoded values and map them to some custom type of your choosing (whether it is a dynamically shaped type or some
static type you'd like to decode into). Implementations of this `Visitor` trait exist for many existing Rust types in the standard
library.

There also exists an `IntoVisitor` trait, which is implemented on many existing rust types and maps a given type to some visitor
implementation capable of decoding into it.

Finally, a wrapper trait, `DecodeAsType`, is auto-implemented for all types that have an `IntoVisitor` implementation,
and whose visitor errors can be turned into a standard `crate::Error`.

For custom structs and enums, one can use the `DecodeAsType` derive macro to have a `DecodeAsType` implementation automatically
generated.

Here's an example of implementing `Visitor` to decode bytes into a custom `Value` type:

```rust
use scale_decode::visitor::{self, TypeId};

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
    fn visit_u256(
        self,
        value: &'_ [u8; 32],
        _type_id: TypeId,
    ) -> Result<Self::Value<'_>, Self::Error> {
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
    fn visit_i256(
        self,
        value: &'_ [u8; 32],
        _type_id: TypeId,
    ) -> Result<Self::Value<'_>, Self::Error> {
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
        while let Some(val) = value.decode_item(ValueVisitor) {
            let val = val?;
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
        for item in value.by_ref() {
            let item = item?;
            let val = item.decode_with_visitor(ValueVisitor)?;
            let name = item.name().unwrap_or("").to_owned();
            vals.push((name, val));
        }
        Ok(Value::Composite(vals))
    }
    fn visit_tuple<'scale>(
        self,
        value: &mut Tuple<'scale, '_>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        let mut vals = vec![];
        while let Some(val) = value.decode_item(ValueVisitor) {
            let val = val?;
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
        for item in fields.by_ref() {
            let item = item?;
            let val = item.decode_with_visitor(ValueVisitor)?;
            let name = item.name().unwrap_or("").to_owned();
            vals.push((name, val));
        }
        Ok(Value::Variant(value.name().to_owned(), vals))
    }
    fn visit_array<'scale>(
        self,
        value: &mut Array<'scale, '_>,
        _type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        let mut vals = vec![];
        while let Some(val) = value.decode_item(ValueVisitor) {
            let val = val?;
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
```

This can then be passed to a decode function like so:

```rust
let value: Value = scale_decode::visitor::decode_with_visitor(scale_bytes, type_id, types, ValueVisitor)?;
```

Where `scale_bytes` are the bytes you'd like to decode, `type_id` is the type stored in the `types` registry
that you'd like to try and decode the bytes into, and `types` is a `scale_info::PortableRegistry` containing
information about the various types in use.

If we were to then write an `IntoVisitor` implementation like so:

```rust
impl scale_decode::IntoVisitor for Value {
    type Visitor = ValueVisitor;
    fn into_visitor() -> Self::Visitor {
        ValueVisitor
    }
}
```

We can then also decode via tha automatic `DecodeAsType` impl like so:

```rust
use scale_decode::DecodeAsType;

let value = Value::decode_as_type(scale_bytes, type_id, types)?;
```

With an `IntoVisitor` impl, you'd also benefit from being able to decode things like `Vec<Value>`, `(Value, bool)`, `Arc<Value>`
and so on in the same way.