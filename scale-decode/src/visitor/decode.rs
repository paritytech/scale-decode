// Copyright (C) 2022 Parity Technologies (UK) Ltd. (admin@parity.io)
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
use crate::visitor::{
    Array, BitSequence, Composite, DecodeAsTypeResult, DecodeError, Sequence, Str, Tuple,
    TypeIdFor, Variant, Visitor,
};
use crate::Field;
use alloc::format;
use alloc::string::ToString;
use codec::{self, Decode};
use scale_type_resolver::{
    BitsOrderFormat, BitsStoreFormat, FieldIter, Primitive, ResolvedTypeVisitor, TypeResolver,
    UnhandledKind, VariantIter,
};

/// Decode data according to the type ID and type resolver provided.
/// The provided pointer to the data slice will be moved forwards as needed
/// depending on what was decoded, and a method on the provided [`Visitor`]
/// will be called depending on the type that needs to be decoded.
pub fn decode_with_visitor<'scale, 'resolver, V: Visitor>(
    data: &mut &'scale [u8],
    ty_id: &TypeIdFor<V>,
    types: &'resolver V::TypeResolver,
    visitor: V,
) -> Result<V::Value<'scale, 'resolver>, V::Error> {
    decode_with_visitor_maybe_compact(data, ty_id, types, visitor, false)
}

pub fn decode_with_visitor_maybe_compact<'scale, 'resolver, V: Visitor>(
    data: &mut &'scale [u8],
    ty_id: &TypeIdFor<V>,
    types: &'resolver V::TypeResolver,
    visitor: V,
    is_compact: bool,
) -> Result<V::Value<'scale, 'resolver>, V::Error> {
    // Provide option to "bail out" and do something custom first.
    let visitor = match visitor.unchecked_decode_as_type(data, ty_id, types) {
        DecodeAsTypeResult::Decoded(r) => return r,
        DecodeAsTypeResult::Skipped(v) => v,
    };

    let decoder = Decoder::new(data, types, ty_id, visitor, is_compact);
    let res = types.resolve_type(ty_id, decoder);

    match res {
        // We got a value back; return it
        Ok(Ok(val)) => Ok(val),
        // We got a visitor error back; return it
        Ok(Err(e)) => Err(e),
        // We got a TypeResolver error back; turn it into a DecodeError and then visitor error to return.
        Err(resolve_type_error) => {
            Err(DecodeError::TypeResolvingError(resolve_type_error.to_string()).into())
        }
    }
}

/// This struct implements `ResolvedTypeVisitor`. One of those methods fired depending on the type that
/// we resolve from the given TypeId, and then based on the information handed to that method we decode
/// the SCALE encoded bytes as needed and then call the relevant method on the `scale_decode::Visitor` to
/// hand back the decoded value (or some nice interface to allow the user to decode the value).
struct Decoder<'a, 'scale, 'resolver, V: Visitor> {
    data: &'a mut &'scale [u8],
    type_id: &'a TypeIdFor<V>,
    types: &'resolver V::TypeResolver,
    visitor: V,
    is_compact: bool,
}

impl<'a, 'scale, 'resolver, V: Visitor> Decoder<'a, 'scale, 'resolver, V> {
    fn new(
        data: &'a mut &'scale [u8],
        types: &'resolver V::TypeResolver,
        type_id: &'a TypeIdFor<V>,
        visitor: V,
        is_compact: bool,
    ) -> Self {
        Decoder { data, type_id, types, is_compact, visitor }
    }
}

impl<'a, 'scale, 'resolver, V: Visitor> ResolvedTypeVisitor<'resolver>
    for Decoder<'a, 'scale, 'resolver, V>
{
    type TypeId = TypeIdFor<V>;
    type Value = Result<V::Value<'scale, 'resolver>, V::Error>;

    fn visit_unhandled(self, kind: UnhandledKind) -> Self::Value {
        let type_id = self.type_id;
        Err(DecodeError::TypeIdNotFound(format!(
            "Kind {kind:?} (type ID {type_id:?}) has not been properly handled"
        ))
        .into())
    }

    fn visit_not_found(self) -> Self::Value {
        let type_id = self.type_id;
        Err(DecodeError::TypeIdNotFound(format!("{type_id:?}")).into())
    }

    fn visit_composite<Fields>(self, mut fields: Fields) -> Self::Value
    where
        Fields: FieldIter<'resolver, Self::TypeId>,
    {
        // guard against invalid compact types: only composites with 1 field can be compact encoded
        if self.is_compact && fields.len() != 1 {
            return Err(DecodeError::CannotDecodeCompactIntoType.into());
        }

        let mut items = Composite::new(self.data, &mut fields, self.types, self.is_compact);
        let res = self.visitor.visit_composite(&mut items, self.type_id);

        // Skip over any bytes that the visitor chose not to decode:
        items.skip_decoding()?;
        *self.data = items.bytes_from_undecoded();

        res
    }

    fn visit_variant<Fields, Var>(self, variants: Var) -> Self::Value
    where
        Fields: FieldIter<'resolver, Self::TypeId> + 'resolver,
        Var: VariantIter<'resolver, Fields>,
    {
        if self.is_compact {
            return Err(DecodeError::CannotDecodeCompactIntoType.into());
        }

        let mut variant = Variant::new(self.data, variants, self.types)?;
        let res = self.visitor.visit_variant(&mut variant, self.type_id);

        // Skip over any bytes that the visitor chose not to decode:
        variant.skip_decoding()?;
        *self.data = variant.bytes_from_undecoded();

        res
    }

    fn visit_sequence(self, inner_type_id: &'resolver Self::TypeId) -> Self::Value {
        if self.is_compact {
            return Err(DecodeError::CannotDecodeCompactIntoType.into());
        }

        let mut items = Sequence::new(self.data, inner_type_id, self.types)?;
        let res = self.visitor.visit_sequence(&mut items, self.type_id);

        // Skip over any bytes that the visitor chose not to decode:
        items.skip_decoding()?;
        *self.data = items.bytes_from_undecoded();

        res
    }

    fn visit_array(self, inner_type_id: &'resolver Self::TypeId, len: usize) -> Self::Value {
        if self.is_compact {
            return Err(DecodeError::CannotDecodeCompactIntoType.into());
        }

        let mut arr = Array::new(self.data, inner_type_id, len, self.types);
        let res = self.visitor.visit_array(&mut arr, self.type_id);

        // Skip over any bytes that the visitor chose not to decode:
        arr.skip_decoding()?;
        *self.data = arr.bytes_from_undecoded();

        res
    }

    fn visit_tuple<TypeIds>(self, type_ids: TypeIds) -> Self::Value
    where
        TypeIds: ExactSizeIterator<Item = &'resolver Self::TypeId>,
    {
        // guard against invalid compact types: only composites with 1 field can be compact encoded
        if self.is_compact && type_ids.len() != 1 {
            return Err(DecodeError::CannotDecodeCompactIntoType.into());
        }

        let mut fields = type_ids.map(Field::unnamed);
        let mut items = Tuple::new(self.data, &mut fields, self.types, self.is_compact);
        let res = self.visitor.visit_tuple(&mut items, self.type_id);

        // Skip over any bytes that the visitor chose not to decode:
        items.skip_decoding()?;
        *self.data = items.bytes_from_undecoded();

        res
    }

    fn visit_primitive(self, primitive: Primitive) -> Self::Value {
        macro_rules! err_if_compact {
            ($is_compact:expr) => {
                if $is_compact {
                    return Err(DecodeError::CannotDecodeCompactIntoType.into());
                }
            };
        }

        fn decode_32_bytes<'scale>(
            data: &mut &'scale [u8],
        ) -> Result<&'scale [u8; 32], DecodeError> {
            // Pull an array from the data if we can, preserving the lifetime.
            let arr: &'scale [u8; 32] = match (*data).try_into() {
                Ok(arr) => arr,
                Err(_) => return Err(DecodeError::NotEnoughInput),
            };
            // If we successfully read the bytes, then advance the pointer past them.
            *data = &data[32..];
            Ok(arr)
        }

        let data = self.data;
        let is_compact = self.is_compact;
        let visitor = self.visitor;
        let type_id = self.type_id;

        match primitive {
            Primitive::Bool => {
                err_if_compact!(is_compact);
                let b = bool::decode(data).map_err(|e| e.into())?;
                visitor.visit_bool(b, type_id)
            }
            Primitive::Char => {
                err_if_compact!(is_compact);
                // Treat chars as u32's
                let val = u32::decode(data).map_err(|e| e.into())?;
                let c = char::from_u32(val).ok_or(DecodeError::InvalidChar(val))?;
                visitor.visit_char(c, type_id)
            }
            Primitive::Str => {
                err_if_compact!(is_compact);
                // Avoid allocating; don't decode into a String. instead, pull the bytes
                // and let the visitor decide whether to use them or not.
                let mut s = Str::new(data)?;
                // Since we aren't decoding here, shift our bytes along to after the str:
                *data = s.bytes_after();
                visitor.visit_str(&mut s, type_id)
            }
            Primitive::U8 => {
                let n = if is_compact {
                    codec::Compact::<u8>::decode(data).map(|c| c.0)
                } else {
                    u8::decode(data)
                }
                .map_err(Into::into)?;
                visitor.visit_u8(n, type_id)
            }
            Primitive::U16 => {
                let n = if is_compact {
                    codec::Compact::<u16>::decode(data).map(|c| c.0)
                } else {
                    u16::decode(data)
                }
                .map_err(Into::into)?;
                visitor.visit_u16(n, type_id)
            }
            Primitive::U32 => {
                let n = if is_compact {
                    codec::Compact::<u32>::decode(data).map(|c| c.0)
                } else {
                    u32::decode(data)
                }
                .map_err(Into::into)?;
                visitor.visit_u32(n, type_id)
            }
            Primitive::U64 => {
                let n = if is_compact {
                    codec::Compact::<u64>::decode(data).map(|c| c.0)
                } else {
                    u64::decode(data)
                }
                .map_err(Into::into)?;
                visitor.visit_u64(n, type_id)
            }
            Primitive::U128 => {
                let n = if is_compact {
                    codec::Compact::<u128>::decode(data).map(|c| c.0)
                } else {
                    u128::decode(data)
                }
                .map_err(Into::into)?;
                visitor.visit_u128(n, type_id)
            }
            Primitive::U256 => {
                err_if_compact!(is_compact);
                let arr = decode_32_bytes(data)?;
                visitor.visit_u256(arr, type_id)
            }
            Primitive::I8 => {
                err_if_compact!(is_compact);
                let n = i8::decode(data).map_err(|e| e.into())?;
                visitor.visit_i8(n, type_id)
            }
            Primitive::I16 => {
                err_if_compact!(is_compact);
                let n = i16::decode(data).map_err(|e| e.into())?;
                visitor.visit_i16(n, type_id)
            }
            Primitive::I32 => {
                err_if_compact!(is_compact);
                let n = i32::decode(data).map_err(|e| e.into())?;
                visitor.visit_i32(n, type_id)
            }
            Primitive::I64 => {
                err_if_compact!(is_compact);
                let n = i64::decode(data).map_err(|e| e.into())?;
                visitor.visit_i64(n, type_id)
            }
            Primitive::I128 => {
                err_if_compact!(is_compact);
                let n = i128::decode(data).map_err(|e| e.into())?;
                visitor.visit_i128(n, type_id)
            }
            Primitive::I256 => {
                err_if_compact!(is_compact);
                let arr = decode_32_bytes(data)?;
                visitor.visit_i256(arr, type_id)
            }
        }
    }

    fn visit_compact(self, inner_type_id: &'resolver Self::TypeId) -> Self::Value {
        decode_with_visitor_maybe_compact(self.data, inner_type_id, self.types, self.visitor, true)
    }

    fn visit_bit_sequence(
        self,
        store_format: BitsStoreFormat,
        order_format: BitsOrderFormat,
    ) -> Self::Value {
        if self.is_compact {
            return Err(DecodeError::CannotDecodeCompactIntoType.into());
        }

        let format = scale_bits::Format::new(store_format, order_format);
        let mut bitseq = BitSequence::new(format, self.data);
        let res = self.visitor.visit_bitsequence(&mut bitseq, self.type_id);

        // Move to the bytes after the bit sequence.
        *self.data = bitseq.bytes_after()?;

        res
    }
}
