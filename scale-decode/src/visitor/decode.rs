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
    Variant, Visitor,
};
use crate::Field;
use codec::{self, Decode};
use scale_type_resolver::{
    BitsOrderFormat,
    BitsStoreFormat,
    Primitive,
    TypeResolver,
    ResolvedTypeVisitor,
    FieldIter,
    VariantIter
};

/// Return the type ID type of some [`Visitor`].
type TypeIdFor<V: Visitor> = <V::TypeResolver as TypeResolver>::TypeId;

/// Decode data according to the type ID and [`PortableRegistry`] provided.
/// The provided pointer to the data slice will be moved forwards as needed
/// depending on what was decoded, and a method on the provided [`Visitor`]
/// will be called depending on the type that needs to be decoded.
pub fn decode_with_visitor<'scale, 'info, V: Visitor>(
    data: &mut &'scale [u8],
    ty_id: TypeIdFor<V>,
    types: &'info V::TypeResolver,
    visitor: V,
) -> Result<V::Value<'scale, 'info>, V::Error> {
    decode_with_visitor_maybe_compact(data, ty_id, types, visitor, false)
}

pub fn decode_with_visitor_maybe_compact<'scale, 'info, V: Visitor>(
    data: &mut &'scale [u8],
    ty_id: TypeIdFor<V>,
    types: &'info V::TypeResolver,
    visitor: V,
    is_compact: bool,
) -> Result<V::Value<'scale, 'info>, V::Error> {
    // Provide option to "bail out" and do something custom first.
    let visitor = match visitor.unchecked_decode_as_type(data, ty_id, types) {
        DecodeAsTypeResult::Decoded(r) => return r,
        DecodeAsTypeResult::Skipped(v) => v,
    };

    let decoder = Decoder::new(data, types, visitor, is_compact);
    let res = types.resolve_type(ty_id, decoder);

    match res {
        // We got a value back; return it
        Ok(Ok(val)) => Ok(val),
        // We got a visitor error back; return it
        Ok(Err(e)) => Err(e),
        // We got a TypeResolver error back; turn it into a DecodeError and then visitor error to return.
        Err(resolve_type_error) => Err(DecodeError::TypeResolvingError(resolve_type_error.to_string()).into()),
    }
}

struct Decoder<'a, 'scale, 'info, V: Visitor> {
    data: &'a mut &'scale [u8],
    types: &'info V::TypeResolver,
    visitor: V,
    is_compact: bool,
}

impl <'a, 'scale, 'info, V: Visitor> Decoder<'a, 'scale, 'info, V> {
    fn new(data: &'a mut &'scale [u8], types: &'info V::TypeResolver, visitor: V, is_compact: bool) -> Self {
        Decoder {
            data,
            types,
            is_compact,
            visitor,
        }
    }
}

impl <'a, 'scale, 'info, V: Visitor> ResolvedTypeVisitor for Decoder<'a, 'scale, 'info, V> {
    type TypeId = TypeIdFor<V>;
    type Value<'info2> = Result<V::Value<'scale, 'info2>, V::Error>;

    fn visit_not_found<'info2>(self, type_id: Self::TypeId) -> Self::Value<'info2> {
        Err(DecodeError::TypeIdNotFound(type_id))
    }

    fn visit_composite<'info2, Fields>(self, type_id: Self::TypeId, mut fields: Fields) -> Self::Value<'info2>
    where
        Fields: FieldIter<'info2, Self::TypeId>
    {
        // guard against invalid compact types: only composites with 1 field can be compact encoded
        if self.is_compact && fields.len() != 1 {
            return Err(DecodeError::CannotDecodeCompactIntoType.into());
        }

        // let mut fields = fields.map(|f| Field::new(f.ty.id, f.name.as_deref()));
        let mut items = Composite::new(self.data, &mut fields, self.types, self.is_compact);
        let res = self.visitor.visit_composite(&mut items, type_id);

        // Skip over any bytes that the visitor chose not to decode:
        items.skip_decoding()?;
        *self.data = items.bytes_from_undecoded();

        res
    }

    fn visit_variant<'info2, Fields, Var>(self, type_id: Self::TypeId, variants: Var) -> Self::Value<'info2>
    where
        Fields: FieldIter<'info2, Self::TypeId>,
        Var: VariantIter<'info2, Fields>
    {
        if self.is_compact{
            return Err(DecodeError::CannotDecodeCompactIntoType.into());
        }

        let mut variant = Variant::new(self.data, variants, self.types)?;
        let res = self.visitor.visit_variant(&mut variant, type_id);

        // Skip over any bytes that the visitor chose not to decode:
        variant.skip_decoding()?;
        *self.data = variant.bytes_from_undecoded();

        res
    }

    fn visit_sequence<'info2>(self, type_id: Self::TypeId) -> Self::Value<'info2> {
        if self.is_compact{
            return Err(DecodeError::CannotDecodeCompactIntoType.into());
        }

        let mut items = Sequence::new(self.data, type_id.clone(), self.types)?;
        let res = self.visitor.visit_sequence(&mut items, type_id);

        // Skip over any bytes that the visitor chose not to decode:
        items.skip_decoding()?;
        *self.data = items.bytes_from_undecoded();

        res
    }

    fn visit_array<'info2>(self, type_id: Self::TypeId, len: usize) -> Self::Value<'info2> {
        if self.is_compact{
            return Err(DecodeError::CannotDecodeCompactIntoType.into());
        }

        let mut arr = Array::new(self.data, type_id.clone(), len, self.types);
        let res = self.visitor.visit_array(&mut arr, type_id);

        // Skip over any bytes that the visitor chose not to decode:
        arr.skip_decoding()?;
        *self.data = arr.bytes_from_undecoded();

        res
    }

    fn visit_tuple<'info2, TypeIds>(self, type_id: Self::TypeId, type_ids: TypeIds) -> Self::Value<'info2>
    where
        TypeIds: ExactSizeIterator<Item=Self::TypeId>
    {
        // guard against invalid compact types: only composites with 1 field can be compact encoded
        if self.is_compact && type_ids.len() != 1 {
            return Err(DecodeError::CannotDecodeCompactIntoType);
        }

        let mut fields = type_ids.iter().map(|id| Field::unnamed(id));
        let mut items = Tuple::new(self.data, &mut fields, self.types, self.is_compact);
        let res = self.visitor.visit_tuple(&mut items, type_id);

        // Skip over any bytes that the visitor chose not to decode:
        items.skip_decoding()?;
        *self.data = items.bytes_from_undecoded();

        res
    }

    fn visit_primitive<'info2>(self, type_id: Self::TypeId, primitive: Primitive) -> Self::Value<'info2> {
        macro_rules! err_if_compact {
            ($is_compact:expr) => {
                if $is_compact {
                    return Err(DecodeError::CannotDecodeCompactIntoType.into());
                }
            };
        }

        fn decode_32_bytes<'scale>(data: &mut &'scale [u8]) -> Result<&'scale [u8; 32], DecodeError> {
            // Pull an array from the data if we can, preserving the lifetime.
            let arr: &'scale [u8; 32] = match (*data).try_into() {
                Ok(arr) => arr,
                Err(_) => return Err(DecodeError::NotEnoughInput),
            };
            // If this works out, remember to shift data 32 bytes forward.
            *data = &data[32..];
            Ok(arr)
        }

        let data = self.data;
        let is_compact = self.is_compact;
        let visitor = self.visitor;

        match primitive {
            Primitive::Bool => {
                err_if_compact!(self.is_compact);
                let b = bool::decode(data).map_err(|e| e.into())?;
                visitor.visit_bool(b, type_id)
            }
            Primitive::Char => {
                err_if_compact!(self.is_compact);
                // Treat chars as u32's
                let val = u32::decode(data).map_err(|e| e.into())?;
                let c = char::from_u32(val).ok_or(DecodeError::InvalidChar(val))?;
                visitor.visit_char(c, type_id)
            }
            Primitive::Str => {
                err_if_compact!(self.is_compact);
                // Avoid allocating; don't decode into a String. instead, pull the bytes
                // and let the visitor decide whether to use them or not.
                let mut s = Str::new(data)?;
                // Since we aren't decoding here, shift our bytes along to after the str:
                *data = s.bytes_after();
                visitor.visit_str(&mut s, type_id)
            }
            Primitive::U8 => {
                let n = if self.is_compact {
                    codec::Compact::<u8>::decode(data).map(|c| c.0)
                } else {
                    u8::decode(data)
                }
                .map_err(Into::into)?;
                visitor.visit_u8(n, type_id)
            }
            Primitive::U16 => {
                let n = if self.is_compact {
                    codec::Compact::<u16>::decode(data).map(|c| c.0)
                } else {
                    u16::decode(data)
                }
                .map_err(Into::into)?;
                visitor.visit_u16(n, type_id)
            }
            Primitive::U32 => {
                let n = if self.is_compact {
                    codec::Compact::<u32>::decode(data).map(|c| c.0)
                } else {
                    u32::decode(data)
                }
                .map_err(Into::into)?;
                visitor.visit_u32(n, type_id)
            }
            Primitive::U64 => {
                let n = if self.is_compact {
                    codec::Compact::<u64>::decode(data).map(|c| c.0)
                } else {
                    u64::decode(data)
                }
                .map_err(Into::into)?;
                visitor.visit_u64(n, type_id)
            }
            Primitive::U128 => {
                let n = if self.is_compact {
                    codec::Compact::<u128>::decode(data).map(|c| c.0)
                } else {
                    u128::decode(data)
                }
                .map_err(Into::into)?;
                visitor.visit_u128(n, type_id)
            }
            Primitive::U256 => {
                err_if_compact!(self.is_compact);
                let arr = decode_32_bytes(data)?;
                visitor.visit_u256(arr, type_id)
            }
            Primitive::I8 => {
                err_if_compact!(self.is_compact);
                let n = i8::decode(data).map_err(|e| e.into())?;
                visitor.visit_i8(n, type_id)
            }
            Primitive::I16 => {
                err_if_compact!(self.is_compact);
                let n = i16::decode(data).map_err(|e| e.into())?;
                visitor.visit_i16(n, type_id)
            }
            Primitive::I32 => {
                err_if_compact!(self.is_compact);
                let n = i32::decode(data).map_err(|e| e.into())?;
                visitor.visit_i32(n, type_id)
            }
            Primitive::I64 => {
                err_if_compact!(self.is_compact);
                let n = i64::decode(data).map_err(|e| e.into())?;
                visitor.visit_i64(n, type_id)
            }
            Primitive::I128 => {
                err_if_compact!(self.is_compact);
                let n = i128::decode(data).map_err(|e| e.into())?;
                visitor.visit_i128(n, type_id)
            }
            Primitive::I256 => {
                err_if_compact!(self.is_compact);
                let arr = decode_32_bytes(data)?;
                visitor.visit_i256(arr, type_id)
            }
        }
    }

    fn visit_compact<'info2>(self, type_id: Self::TypeId) -> Self::Value<'info2> {
        decode_with_visitor_maybe_compact(self.data, type_id, self.types, self.visitor, true)
    }

    fn visit_bit_sequence<'info2>(self, type_id: Self::TypeId, store_format: BitsStoreFormat, order_format: BitsOrderFormat) -> Self::Value<'info2> {
        if self.is_compact{
            return Err(DecodeError::CannotDecodeCompactIntoType.into());
        }

        let format = scale_bits::Format::new(store_format, order_format);
        let mut bitseq = BitSequence::new(format, self.data);
        let res = self.visitor.visit_bitsequence(&mut bitseq, type_id);

        // Move to the bytes after the bit sequence.
        *self.data = bitseq.bytes_after()?;

        res
    }
}
