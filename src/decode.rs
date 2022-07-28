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

use crate::utils::{bit_sequence::get_bitsequence_details, stack_vec::StackVec};
use crate::visitor::{
	Array, BitSequence, Compact, CompactLocation, Composite, DecodeError, Sequence, Str, Tuple,
	TypeId, Variant, Visitor,
};
use codec::{self, Decode};
use scale_info::{
	form::PortableForm, PortableRegistry, TypeDef, TypeDefArray, TypeDefBitSequence,
	TypeDefCompact, TypeDefComposite, TypeDefPrimitive, TypeDefSequence, TypeDefTuple,
	TypeDefVariant,
};

/// Decode data according to the type ID and [`PortableRegistry`] provided.
/// The provided pointer to the data slice will be moved forwards as needed
/// depending on what was decoded, and a method on the provided [`Visitor`]
/// will be called depending on the type that needs to be decoded.
pub fn decode<V: Visitor>(
	data: &mut &[u8],
	ty_id: u32,
	types: &PortableRegistry,
	visitor: V,
) -> Result<V::Value, V::Error> {
	let ty = types.resolve(ty_id).ok_or(DecodeError::TypeIdNotFound(ty_id))?;
	let ty_id = TypeId(ty_id);

	match ty.type_def() {
		TypeDef::Composite(inner) => decode_composite_value(data, ty_id, inner, types, visitor),
		TypeDef::Sequence(inner) => decode_sequence_value(data, ty_id, inner, types, visitor),
		TypeDef::Variant(inner) => decode_variant_value(data, ty_id, inner, types, visitor),
		TypeDef::Array(inner) => decode_array_value(data, ty_id, inner, types, visitor),
		TypeDef::Tuple(inner) => decode_tuple_value(data, ty_id, inner, types, visitor),
		TypeDef::Primitive(inner) => decode_primitive_value(data, ty_id, inner, visitor),
		TypeDef::Compact(inner) => decode_compact_value(data, ty_id, inner, types, visitor),
		TypeDef::BitSequence(inner) => {
			decode_bit_sequence_value(data, ty_id, inner, types, visitor)
		}
	}
}

fn decode_composite_value<'b, V: Visitor>(
	data: &mut &[u8],
	ty_id: TypeId,
	ty: &'b TypeDefComposite<PortableForm>,
	types: &'b PortableRegistry,
	visitor: V,
) -> Result<V::Value, V::Error> {
	let mut items = Composite::new(data, ty.fields(), types);
	let res = visitor.visit_composite(&mut items, ty_id);

	// Skip over any bytes that the visitor chose not to decode:
	items.skip_rest()?;
	*data = items.bytes();

	res
}

fn decode_variant_value<'b, V: Visitor>(
	data: &mut &[u8],
	ty_id: TypeId,
	ty: &'b TypeDefVariant<PortableForm>,
	types: &'b PortableRegistry,
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
	let res = visitor.visit_variant(&mut variant, ty_id);

	// Skip over any bytes that the visitor chose not to decode:
	variant.skip_rest()?;
	*data = variant.bytes();

	res
}

fn decode_sequence_value<'b, V: Visitor>(
	data: &mut &[u8],
	ty_id: TypeId,
	ty: &'b TypeDefSequence<PortableForm>,
	types: &'b PortableRegistry,
	visitor: V,
) -> Result<V::Value, V::Error> {
	// We assume that the sequence is preceeded by a compact encoded length, so that
	// we know how many values to try pulling out of the data.
	let len = codec::Compact::<u64>::decode(data).map_err(|e| e.into())?;
	let mut items = Sequence::new(data, ty.type_param().id(), len.0 as usize, types);
	let res = visitor.visit_sequence(&mut items, ty_id);

	// Skip over any bytes that the visitor chose not to decode:
	items.skip_rest()?;
	*data = items.bytes();

	res
}

fn decode_array_value<'b, V: Visitor>(
	data: &mut &[u8],
	ty_id: TypeId,
	ty: &'b TypeDefArray<PortableForm>,
	types: &'b PortableRegistry,
	visitor: V,
) -> Result<V::Value, V::Error> {
	let len = ty.len() as usize;
	let seq = Sequence::new(data, ty.type_param().id(), len, types);
	let mut arr = Array::new(seq);
	let res = visitor.visit_array(&mut arr, ty_id);

	// Skip over any bytes that the visitor chose not to decode:
	arr.skip_rest()?;
	*data = arr.bytes();

	res
}

fn decode_tuple_value<'b, V: Visitor>(
	data: &mut &[u8],
	ty_id: TypeId,
	ty: &'b TypeDefTuple<PortableForm>,
	types: &'b PortableRegistry,
	visitor: V,
) -> Result<V::Value, V::Error> {
	let mut items = Tuple::new(*data, ty.fields(), types);
	let res = visitor.visit_tuple(&mut items, ty_id);

	// Skip over any bytes that the visitor chose not to decode:
	items.skip_rest()?;
	*data = items.bytes();

	res
}

fn decode_primitive_value<V: Visitor>(
	data: &mut &[u8],
	ty_id: TypeId,
	ty: &TypeDefPrimitive,
	visitor: V,
) -> Result<V::Value, V::Error> {
	match ty {
		TypeDefPrimitive::Bool => {
			let b = bool::decode(data).map_err(|e| e.into())?;
			visitor.visit_bool(b, ty_id)
		}
		TypeDefPrimitive::Char => {
			// Treat chars as u32's
			let val = u32::decode(data).map_err(|e| e.into())?;
			let c = char::from_u32(val).ok_or(DecodeError::InvalidChar(val))?;
			visitor.visit_char(c, ty_id)
		}
		TypeDefPrimitive::Str => {
			// Avoid allocating; don't decode into a String. instead, pull the bytes
			// and let the visitor decide whether to use them or not.
			let s = Str::new_from(data)?;
			visitor.visit_str(s, ty_id)
		}
		TypeDefPrimitive::U8 => {
			let n = u8::decode(data).map_err(|e| e.into())?;
			visitor.visit_u8(n, ty_id)
		}
		TypeDefPrimitive::U16 => {
			let n = u16::decode(data).map_err(|e| e.into())?;
			visitor.visit_u16(n, ty_id)
		}
		TypeDefPrimitive::U32 => {
			let n = u32::decode(data).map_err(|e| e.into())?;
			visitor.visit_u32(n, ty_id)
		}
		TypeDefPrimitive::U64 => {
			let n = u64::decode(data).map_err(|e| e.into())?;
			visitor.visit_u64(n, ty_id)
		}
		TypeDefPrimitive::U128 => {
			let n = u128::decode(data).map_err(|e| e.into())?;
			visitor.visit_u128(n, ty_id)
		}
		TypeDefPrimitive::U256 => {
			// Note; pass a reference to the visitor because this can be optimised to
			// take a slice of the input bytes instead of decoding to a stack value.
			let n = <[u8; 32]>::decode(data).map_err(|e| e.into())?;
			visitor.visit_u256(&n, ty_id)
		}
		TypeDefPrimitive::I8 => {
			let n = i8::decode(data).map_err(|e| e.into())?;
			visitor.visit_i8(n, ty_id)
		}
		TypeDefPrimitive::I16 => {
			let n = i16::decode(data).map_err(|e| e.into())?;
			visitor.visit_i16(n, ty_id)
		}
		TypeDefPrimitive::I32 => {
			let n = i32::decode(data).map_err(|e| e.into())?;
			visitor.visit_i32(n, ty_id)
		}
		TypeDefPrimitive::I64 => {
			let n = i64::decode(data).map_err(|e| e.into())?;
			visitor.visit_i64(n, ty_id)
		}
		TypeDefPrimitive::I128 => {
			let n = i128::decode(data).map_err(|e| e.into())?;
			visitor.visit_i128(n, ty_id)
		}
		TypeDefPrimitive::I256 => {
			// Note; pass a reference to the visitor because this can be optimised to
			// take a slice of the input bytes instead of decoding to a stack value.
			let n = <[u8; 32]>::decode(data).map_err(|e| e.into())?;
			visitor.visit_i256(&n, ty_id)
		}
	}
}

fn decode_compact_value<'b, V: Visitor>(
	data: &mut &[u8],
	ty_id: TypeId,
	ty: &TypeDefCompact<PortableForm>,
	types: &'b PortableRegistry,
	visitor: V,
) -> Result<V::Value, V::Error> {
	#[allow(clippy::too_many_arguments)]
	fn decode_compact<'b, V: Visitor>(
		data: &mut &[u8],
		outermost_ty_id: TypeId,
		current_type_id: TypeId,
		mut locations: StackVec<CompactLocation<'b>, 8>,
		inner: &'b scale_info::Type<PortableForm>,
		types: &'b PortableRegistry,
		visitor: V,
		depth: usize,
	) -> Result<V::Value, V::Error> {
		use TypeDefPrimitive::*;
		match inner.type_def() {
			// It's obvious how to decode basic primitive unsigned types, since we have impls for them.
			TypeDef::Primitive(U8) => {
				locations.push(CompactLocation::Primitive(current_type_id));
				let n = codec::Compact::<u8>::decode(data).map_err(|e| e.into())?.0;
				let c = Compact::new(n, locations.as_slice());
				visitor.visit_compact_u8(c, outermost_ty_id)
			}
			TypeDef::Primitive(U16) => {
				locations.push(CompactLocation::Primitive(current_type_id));
				let n = codec::Compact::<u16>::decode(data).map_err(|e| e.into())?.0;
				let c = Compact::new(n, locations.as_slice());
				visitor.visit_compact_u16(c, outermost_ty_id)
			}
			TypeDef::Primitive(U32) => {
				locations.push(CompactLocation::Primitive(current_type_id));
				let n = codec::Compact::<u32>::decode(data).map_err(|e| e.into())?.0;
				let c = Compact::new(n, locations.as_slice());
				visitor.visit_compact_u32(c, outermost_ty_id)
			}
			TypeDef::Primitive(U64) => {
				locations.push(CompactLocation::Primitive(current_type_id));
				let n = codec::Compact::<u64>::decode(data).map_err(|e| e.into())?.0;
				let c = Compact::new(n, locations.as_slice());
				visitor.visit_compact_u64(c, outermost_ty_id)
			}
			TypeDef::Primitive(U128) => {
				locations.push(CompactLocation::Primitive(current_type_id));
				let n = codec::Compact::<u128>::decode(data).map_err(|e| e.into())?.0;
				let c = Compact::new(n, locations.as_slice());
				visitor.visit_compact_u128(c, outermost_ty_id)
			}
			// A struct with exactly 1 field containing one of the above types can be sensibly compact encoded/decoded.
			TypeDef::Composite(composite) => {
				if composite.fields().len() != 1 {
					return Err(DecodeError::CannotDecodeCompactIntoType(inner.clone()).into());
				}

				// What type is the 1 field that we are able to decode?
				let field = &composite.fields()[0];

				// Record this composite location.
				match field.name() {
					Some(name) => {
						locations.push(CompactLocation::NamedComposite(current_type_id, &**name))
					}
					None => locations.push(CompactLocation::UnnamedComposite(current_type_id)),
				}

				let field_type_id = field.ty().id();
				let inner_ty = types
					.resolve(field_type_id)
					.ok_or(DecodeError::TypeIdNotFound(field_type_id))?;

				// Decode this inner type via compact decoding. This can recurse, in case
				// the inner type is also a 1-field composite type.
				decode_compact(
					data,
					outermost_ty_id,
					TypeId(field_type_id),
					locations,
					inner_ty,
					types,
					visitor,
					depth + 1,
				)
			}
			// For now, we give up if we have been asked for any other type:
			_cannot_decode_from => {
				Err(DecodeError::CannotDecodeCompactIntoType(inner.clone()).into())
			}
		}
	}

	// The type ID of the thing encoded into a Compact type.
	let inner_ty_id = ty.type_param().id();

	// Attempt to compact-decode this inner type.
	let inner = types.resolve(inner_ty_id).ok_or(DecodeError::TypeIdNotFound(inner_ty_id))?;

	// Track any inner type IDs we encounter.
	let locations = StackVec::<CompactLocation, 8>::new();

	decode_compact(data, ty_id, TypeId(inner_ty_id), locations, inner, types, visitor, 0)
}

fn decode_bit_sequence_value<V: Visitor>(
	data: &mut &[u8],
	ty_id: TypeId,
	ty: &TypeDefBitSequence<PortableForm>,
	types: &PortableRegistry,
	visitor: V,
) -> Result<V::Value, V::Error> {
	let (store, order) =
		get_bitsequence_details(ty, types).map_err(DecodeError::BitSequenceError)?;

	let mut bitseq = BitSequence::new(store, order, data);
	let res = visitor.visit_bitsequence(&mut bitseq, ty_id);

	// Decode and skip over the bytes regardless of whether the visitor chooses to or not.
	bitseq.skip_if_not_decoded()?;
	*data = bitseq.bytes();

	res
}
