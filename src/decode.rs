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

use crate::bit_sequence::get_bitsequence_details;
use crate::visitor::{
	types::{Array, BitSequence, Composite, Sequence, Str, Tuple, Variant},
	DecodeError, Visitor,
};
use codec::{Compact, Decode};
use scale_info::{
	form::PortableForm, PortableRegistry, TypeDef, TypeDefArray, TypeDefBitSequence,
	TypeDefCompact, TypeDefComposite, TypeDefPrimitive, TypeDefSequence, TypeDefTuple,
	TypeDefVariant,
};

/// Decode data according to the type ID and [`PortableRegistry`] provided.
/// The provided pointer to the data slice will be moved forwards as needed
/// depending on what was decoded, and a method on the provided [`Visitor`]
/// will be called depending on the type that needs to be decoded.
pub fn decode<'a, V: Visitor>(
	data: &mut &'a [u8],
	ty_id: u32,
	types: &'a PortableRegistry,
	visitor: V,
) -> Result<V::Value, V::Error> {
	let ty = types.resolve(ty_id).ok_or(DecodeError::TypeIdNotFound(ty_id))?;

	match ty.type_def() {
		TypeDef::Composite(inner) => decode_composite_value(data, inner, types, visitor),
		TypeDef::Sequence(inner) => decode_sequence_value(data, inner, types, visitor),
		TypeDef::Variant(inner) => decode_variant_value(data, inner, types, visitor),
		TypeDef::Array(inner) => decode_array_value(data, inner, types, visitor),
		TypeDef::Tuple(inner) => decode_tuple_value(data, inner, types, visitor),
		TypeDef::Primitive(inner) => decode_primitive_value(data, inner, visitor),
		TypeDef::Compact(inner) => decode_compact_value(data, inner, types, visitor),
		TypeDef::BitSequence(inner) => decode_bit_sequence_value(data, inner, types, visitor),
	}
}

fn decode_composite_value<'a, V: Visitor>(
	data: &mut &'a [u8],
	ty: &'a TypeDefComposite<PortableForm>,
	types: &'a PortableRegistry,
	visitor: V,
) -> Result<V::Value, V::Error> {
	let mut items = Composite::new(data, ty.fields(), types);
	let res = visitor.visit_composite(&mut items);

	// Skip over any bytes that the visitor chose not to decode:
	items.skip_rest()?;
	*data = items.bytes();

	res
}

fn decode_variant_value<'a, V: Visitor>(
	data: &mut &'a [u8],
	ty: &'a TypeDefVariant<PortableForm>,
	types: &'a PortableRegistry,
	visitor: V,
) -> Result<V::Value, V::Error> {
	let index = *data.get(0).ok_or(DecodeError::Eof)?;
	*data = &data[1..];

	// Does a variant exist with the index we're looking for?
	let variant = ty
		.variants()
		.iter()
		.find(|v| v.index() == index)
		.ok_or_else(|| DecodeError::VariantNotFound(index, ty.clone()))?;

	let composite = Composite::new(data, variant.fields(), types);
	let mut variant = Variant::new(variant, composite);
	let res = visitor.visit_variant(&mut variant);

	// Skip over any bytes that the visitor chose not to decode:
	variant.skip_rest()?;
	*data = variant.bytes();

	res
}

fn decode_sequence_value<'a, V: Visitor>(
	data: &mut &'a [u8],
	ty: &'a TypeDefSequence<PortableForm>,
	types: &'a PortableRegistry,
	visitor: V,
) -> Result<V::Value, V::Error> {
	// We assume that the sequence is preceeded by a compact encoded length, so that
	// we know how many values to try pulling out of the data.
	let len = Compact::<u64>::decode(data).map_err(|e| e.into())?;
	let mut items = Sequence::new(data, ty.type_param().id(), len.0 as usize, types);
	let res = visitor.visit_sequence(&mut items);

	// Skip over any bytes that the visitor chose not to decode:
	items.skip_rest()?;
	*data = items.bytes();

	res
}

fn decode_array_value<'a, V: Visitor>(
	data: &mut &'a [u8],
	ty: &'a TypeDefArray<PortableForm>,
	types: &'a PortableRegistry,
	visitor: V,
) -> Result<V::Value, V::Error> {
	let len = ty.len() as usize;
	let seq = Sequence::new(data, ty.type_param().id(), len, types);
	let mut arr = Array::new(seq);
	let res = visitor.visit_array(&mut arr);

	// Skip over any bytes that the visitor chose not to decode:
	arr.skip_rest()?;
	*data = arr.bytes();

	res
}

fn decode_tuple_value<'a, V: Visitor>(
	data: &mut &'a [u8],
	ty: &'a TypeDefTuple<PortableForm>,
	types: &'a PortableRegistry,
	visitor: V,
) -> Result<V::Value, V::Error> {
	let mut items = Tuple::new(data, ty.fields(), types);
	let res = visitor.visit_tuple(&mut items);

	// Skip over any bytes that the visitor chose not to decode:
	items.skip_rest()?;
	*data = items.bytes();

	res
}

fn decode_primitive_value<V: Visitor>(
	data: &mut &[u8],
	ty: &TypeDefPrimitive,
	visitor: V,
) -> Result<V::Value, V::Error> {
	match ty {
		TypeDefPrimitive::Bool => {
			let b = bool::decode(data).map_err(|e| e.into())?;
			visitor.visit_bool(b)
		}
		TypeDefPrimitive::Char => {
			// Treat chars as u32's
			let val = u32::decode(data).map_err(|e| e.into())?;
			let c = char::from_u32(val).ok_or(DecodeError::InvalidChar(val))?;
			visitor.visit_char(c)
		}
		TypeDefPrimitive::Str => {
			// Avoid allocating; don't decode into a String. instead, pull the bytes
			// and let the visitor decide whether to use them or not.
			let s = Str::new_from(data)?;
			visitor.visit_str(&s)
		}
		TypeDefPrimitive::U8 => {
			let n = u8::decode(data).map_err(|e| e.into())?;
			visitor.visit_u8(n)
		}
		TypeDefPrimitive::U16 => {
			let n = u16::decode(data).map_err(|e| e.into())?;
			visitor.visit_u16(n)
		}
		TypeDefPrimitive::U32 => {
			let n = u32::decode(data).map_err(|e| e.into())?;
			visitor.visit_u32(n)
		}
		TypeDefPrimitive::U64 => {
			let n = u64::decode(data).map_err(|e| e.into())?;
			visitor.visit_u64(n)
		}
		TypeDefPrimitive::U128 => {
			let n = u128::decode(data).map_err(|e| e.into())?;
			visitor.visit_u128(n)
		}
		TypeDefPrimitive::U256 => {
			// Note; pass a reference to the visitor because this can be optimised to
			// take a slice of the input bytes instead of decoding to a stack value.
			let n = <[u8; 32]>::decode(data).map_err(|e| e.into())?;
			visitor.visit_u256(&n)
		}
		TypeDefPrimitive::I8 => {
			let n = i8::decode(data).map_err(|e| e.into())?;
			visitor.visit_i8(n)
		}
		TypeDefPrimitive::I16 => {
			let n = i16::decode(data).map_err(|e| e.into())?;
			visitor.visit_i16(n)
		}
		TypeDefPrimitive::I32 => {
			let n = i32::decode(data).map_err(|e| e.into())?;
			visitor.visit_i32(n)
		}
		TypeDefPrimitive::I64 => {
			let n = i64::decode(data).map_err(|e| e.into())?;
			visitor.visit_i64(n)
		}
		TypeDefPrimitive::I128 => {
			let n = i128::decode(data).map_err(|e| e.into())?;
			visitor.visit_i128(n)
		}
		TypeDefPrimitive::I256 => {
			// Note; pass a reference to the visitor because this can be optimised to
			// take a slice of the input bytes instead of decoding to a stack value.
			let n = <[u8; 32]>::decode(data).map_err(|e| e.into())?;
			visitor.visit_i256(&n)
		}
	}
}

fn decode_compact_value<V: Visitor>(
	data: &mut &[u8],
	ty: &TypeDefCompact<PortableForm>,
	types: &PortableRegistry,
	visitor: V,
) -> Result<V::Value, V::Error> {
	fn decode_compact<V: Visitor>(
		data: &mut &[u8],
		inner: &scale_info::Type<PortableForm>,
		types: &PortableRegistry,
		visitor: V,
	) -> Result<V::Value, V::Error> {
		use TypeDefPrimitive::*;
		match inner.type_def() {
			// It's obvious how to decode basic primitive unsigned types, since we have impls for them.
			TypeDef::Primitive(U8) => {
				let n = Compact::<u8>::decode(data).map_err(|e| e.into())?.0;
				visitor.visit_compact_u8(n)
			}
			TypeDef::Primitive(U16) => {
				let n = Compact::<u16>::decode(data).map_err(|e| e.into())?.0;
				visitor.visit_compact_u16(n)
			}
			TypeDef::Primitive(U32) => {
				let n = Compact::<u32>::decode(data).map_err(|e| e.into())?.0;
				visitor.visit_compact_u32(n)
			}
			TypeDef::Primitive(U64) => {
				let n = Compact::<u64>::decode(data).map_err(|e| e.into())?.0;
				visitor.visit_compact_u64(n)
			}
			TypeDef::Primitive(U128) => {
				let n = Compact::<u128>::decode(data).map_err(|e| e.into())?.0;
				visitor.visit_compact_u128(n)
			}
			// A struct with exactly 1 field containing one of the above types can be sensibly compact encoded/decoded.
			TypeDef::Composite(composite) => {
				if composite.fields().len() != 1 {
					return Err(DecodeError::CannotDecodeCompactIntoType(inner.clone()).into());
				}

				// What type is the 1 field that we are able to decode?
				let field = &composite.fields()[0];
				let field_type_id = field.ty().id();
				let inner_ty = types
					.resolve(field_type_id)
					.ok_or(DecodeError::TypeIdNotFound(field_type_id))?;

				// Decode this inner type via compact decoding. This can recurse, in case
				// the inner type is also a 1-field composite type.
				decode_compact(data, inner_ty, types, visitor)
			}
			// For now, we give up if we have been asked for any other type:
			_cannot_decode_from => {
				Err(DecodeError::CannotDecodeCompactIntoType(inner.clone()).into())
			}
		}
	}

	// Pluck the inner type out and run it through our compact decoding logic.
	let inner = types
		.resolve(ty.type_param().id())
		.ok_or_else(|| DecodeError::TypeIdNotFound(ty.type_param().id()))?;
	decode_compact(data, inner, types, visitor)
}

fn decode_bit_sequence_value<V: Visitor>(
	data: &mut &[u8],
	ty: &TypeDefBitSequence<PortableForm>,
	types: &PortableRegistry,
	visitor: V,
) -> Result<V::Value, V::Error> {
	let (store, order) =
		get_bitsequence_details(ty, types).map_err(DecodeError::BitSequenceError)?;

	let mut bitseq = BitSequence::new(store, order, data);
	let res = visitor.visit_bitsequence(&mut bitseq);

	// Decode and skip over the bytes regardless of whether the visitor chooses to or not.
	bitseq.skip_if_not_decoded()?;
	*data = bitseq.bytes();

	res
}
