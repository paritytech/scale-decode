mod visitor;
mod bit_sequence;

use bit_sequence::{get_bitsequence_details, BitOrderTy, BitSequenceError, BitStoreTy};
use bitvec::{
	order::{BitOrder, Lsb0, Msb0},
	store::BitStore,
	vec::BitVec,
};
use codec::{Compact, Decode};
use scale_info::{
	form::PortableForm, Field, PortableRegistry, TypeDefArray, TypeDefBitSequence, TypeDefCompact,
	TypeDefComposite, TypeDefPrimitive, TypeDefSequence, TypeDefTuple, TypeDefVariant,
};

pub use visitor::{
    Visitor,
    Fields,
    Sequence,
    Tuple,
    IgnoreVisitor,
    DecodeError,
};

/// The portable version of [`scale_info::Type`]
type Type = scale_info::Type<scale_info::form::PortableForm>;

/// The portable version of a [`scale_info`] type ID.
type TypeId = scale_info::interner::UntrackedSymbol<std::any::TypeId>; // equivalent to: <scale_info::form::PortableForm as scale_info::form::Form>::Type;

/// The portable version of [`scale_info::TypeDef`]
type TypeDef = scale_info::TypeDef<scale_info::form::PortableForm>;


/// Decode data according to the [`TypeId`] provided.
/// The provided pointer to the data slice will be moved forwards as needed
/// depending on what was decoded.
pub fn decode<'a, V: Visitor>(
	data: &mut &'a [u8],
	ty_id: u32,
	types: &'a PortableRegistry,
    visitor: V,
) -> Result<V::Value, V::Error> {
	let ty = types.resolve(ty_id).ok_or_else(|| DecodeError::TypeIdNotFound(ty_id))?;

	match ty.type_def() {
		TypeDef::Composite(inner) => {
			decode_composite_value(data, inner, types, visitor)
		},
		TypeDef::Sequence(inner) => {
            decode_sequence_value(data, inner, types, visitor)
		}
        TypeDef::Variant(inner) => {
            decode_variant_value(data, inner, types, visitor)
        },
        TypeDef::Array(inner) => {
            decode_array_value(data, inner, types, visitor)
        },
        TypeDef::Tuple(inner) => {
            decode_tuple_value(data, inner, types, visitor)
        },
        TypeDef::Primitive(inner) => {
            decode_primitive_value(data, inner, visitor)
        },
		// TypeDef::Compact(inner) => decode_compact_value(data, inner, types, visitor),
		// TypeDef::BitSequence(inner) => {
		// 	decode_bit_sequence_value(data, inner, types, visitor)
		// },
        _ => todo!()
	}
}

fn decode_composite_value<'a, V: Visitor>(
	data: &mut &'a [u8],
	ty: &'a TypeDefComposite<PortableForm>,
	types: &'a PortableRegistry,
    visitor: V,
) -> Result<V::Value, V::Error> {
    decode_fields(data, ty.fields(), types, visitor)
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

	decode_fields(data, variant.fields(), types, visitor)
}

/// Variant and Composite types both have fields; this will decode them into values.
fn decode_fields<'a, V: Visitor>(
	data: &mut &'a [u8],
	fields: &'a [Field<PortableForm>],
	types: &'a PortableRegistry,
    visitor: V,
) -> Result<V::Value, V::Error> {
    let mut items = Fields::new(data, fields, types);
    let res = visitor.visit_composite(&mut items);

    // Skip over any bytes that the visitor chose not to decode:
    items.skip_rest()?;
    *data = items.bytes();

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
    let mut items = Sequence::new(data, ty.type_param().id(), len, types);
    let res = visitor.visit_sequence(&mut items);

    // Skip over any bytes that the visitor chose not to decode:
    items.skip_rest()?;
    *data = items.bytes();

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
	let val = match ty {
		TypeDefPrimitive::Bool => {
            let b = bool::decode(data).map_err(|e| e.into())?;
            visitor.visit_bool(b)
        },
		TypeDefPrimitive::Char => {
			// Treat chars as u32's
			let val = u32::decode(data).map_err(|e| e.into())?;
            let c = char::from_u32(val).ok_or(DecodeError::InvalidChar(val))?;
            visitor.visit_char(c)
		}
		TypeDefPrimitive::Str => Primitive::String(String::decode(data)?), // todo avoid allocating
		TypeDefPrimitive::U8 => {
            let n = u8::decode(data).map_err(|e| e.into())?;
            visitor.visit_u8(n)
        },
		TypeDefPrimitive::U16 => {
            let n = u16::decode(data).map_err(|e| e.into())?;
            visitor.visit_u16(n)
        },
		TypeDefPrimitive::U32 => {
            let n = u32::decode(data).map_err(|e| e.into())?;
            visitor.visit_u32(n)
        },
		TypeDefPrimitive::U64 => {
            let n = u64::decode(data).map_err(|e| e.into())?;
            visitor.visit_u64(n)
        },
		TypeDefPrimitive::U128 => {
            let n = u128::decode(data).map_err(|e| e.into())?;
            visitor.visit_u128(n)
        },
		TypeDefPrimitive::U256 => {
            // Note; pass a reference to the visitor because this can be optimised to
            // take a slice of the input bytes instead of decoding to a stack value.
            let n = <[u8; 32]>::decode(data).map_err(|e| e.into())?;
            visitor.visit_u256(&n)
        },
		TypeDefPrimitive::I8 => {
            let n = i8::decode(data).map_err(|e| e.into())?;
            visitor.visit_i8(n)
        },
		TypeDefPrimitive::I16 => {
            let n = i16::decode(data).map_err(|e| e.into())?;
            visitor.visit_i16(n)
        },
		TypeDefPrimitive::I32 => {
            let n = i32::decode(data).map_err(|e| e.into())?;
            visitor.visit_i32(n)
        },
		TypeDefPrimitive::I64 => {
            let n = i64::decode(data).map_err(|e| e.into())?;
            visitor.visit_i64(n)
        },
		TypeDefPrimitive::I128 => {
            let n = i128::decode(data).map_err(|e| e.into())?;
            visitor.visit_i128(n)
        },
		TypeDefPrimitive::I256 => {
            // Note; pass a reference to the visitor because this can be optimised to
            // take a slice of the input bytes instead of decoding to a stack value.
            let n = <[u8; 32]>::decode(data).map_err(|e| e.into())?;
            visitor.visit_i256(&n)
        },
	};
	Ok(val)
}

// fn decode_compact_value<V: Visitor>(
// 	data: &mut &[u8],
// 	ty: &TypeDefCompact<PortableForm>,
// 	types: &PortableRegistry,
//     visitor: V,
// ) -> Result<ValueDef<TypeId>, DecodeError> {
// 	fn decode_compact(
// 		data: &mut &[u8],
// 		inner: &Type,
// 		types: &PortableRegistry,
// 	) -> Result<ValueDef<TypeId>, DecodeError> {
// 		use TypeDefPrimitive::*;
// 		let val = match inner.type_def() {
// 			// It's obvious how to decode basic primitive unsigned types, since we have impls for them.
// 			TypeDef::Primitive(U8) => {
// 				ValueDef::Primitive(Primitive::u128(Compact::<u8>::decode(data)?.0 as u128))
// 			}
// 			TypeDef::Primitive(U16) => {
// 				ValueDef::Primitive(Primitive::u128(Compact::<u16>::decode(data)?.0 as u128))
// 			}
// 			TypeDef::Primitive(U32) => {
// 				ValueDef::Primitive(Primitive::u128(Compact::<u32>::decode(data)?.0 as u128))
// 			}
// 			TypeDef::Primitive(U64) => {
// 				ValueDef::Primitive(Primitive::u128(Compact::<u64>::decode(data)?.0 as u128))
// 			}
// 			TypeDef::Primitive(U128) => {
// 				ValueDef::Primitive(Primitive::u128(Compact::<u128>::decode(data)?.0 as u128))
// 			}
// 			// A struct with exactly 1 field containing one of the above types can be sensibly compact encoded/decoded.
// 			TypeDef::Composite(composite) => {
// 				if composite.fields().len() != 1 {
// 					return Err(DecodeError::CannotDecodeCompactIntoType(inner.clone()));
// 				}

// 				// What type is the 1 field that we are able to decode?
// 				let field = &composite.fields()[0];
// 				let field_type_id = field.ty().id();
// 				let inner_ty = types
// 					.resolve(field_type_id)
// 					.ok_or(DecodeError::TypeIdNotFound(field_type_id))?;

// 				// Decode this inner type via compact decoding. This can recurse, in case
// 				// the inner type is also a 1-field composite type.
// 				let inner_value = Value {
// 					value: decode_compact(data, inner_ty, types)?,
// 					context: field.ty().into(),
// 				};

// 				// Wrap the inner type in a representation of this outer composite type.
// 				let composite = match field.name() {
// 					Some(name) => Composite::Named(vec![(name.clone(), inner_value)]),
// 					None => Composite::Unnamed(vec![inner_value]),
// 				};

// 				ValueDef::Composite(composite)
// 			}
// 			// For now, we give up if we have been asked for any other type:
// 			_cannot_decode_from => {
// 				return Err(DecodeError::CannotDecodeCompactIntoType(inner.clone()))
// 			}
// 		};

// 		Ok(val)
// 	}

// 	// Pluck the inner type out and run it through our compact decoding logic.
// 	let inner = types
// 		.resolve(ty.type_param().id())
// 		.ok_or_else(|| DecodeError::TypeIdNotFound(ty.type_param().id()))?;
// 	decode_compact(data, inner, types)
// }

// fn decode_bit_sequence_value<V: Visitor>(
// 	data: &mut &[u8],
// 	ty: &TypeDefBitSequence<PortableForm>,
// 	types: &PortableRegistry,
//     visitor: V,
// ) -> Result<BitSequence, DecodeError> {
// 	let details = get_bitsequence_details(ty, types).map_err(DecodeError::BitSequenceError)?;

// 	fn to_bit_sequence<S: BitStore, O: BitOrder>(bits: BitVec<S, O>) -> BitSequence {
// 		bits.iter().by_vals().collect()
// 	}

// 	// Decode the native BitSequence type easily, or else convert to it from the type given.
// 	let bits = match details {
// 		(BitStoreTy::U8, BitOrderTy::Lsb0) => BitVec::<u8, Lsb0>::decode(data)?,
// 		(BitStoreTy::U8, BitOrderTy::Msb0) => to_bit_sequence(BitVec::<u8, Msb0>::decode(data)?),
// 		(BitStoreTy::U16, BitOrderTy::Lsb0) => to_bit_sequence(BitVec::<u16, Lsb0>::decode(data)?),
// 		(BitStoreTy::U16, BitOrderTy::Msb0) => to_bit_sequence(BitVec::<u16, Msb0>::decode(data)?),
// 		(BitStoreTy::U32, BitOrderTy::Lsb0) => to_bit_sequence(BitVec::<u32, Lsb0>::decode(data)?),
// 		(BitStoreTy::U32, BitOrderTy::Msb0) => to_bit_sequence(BitVec::<u32, Msb0>::decode(data)?),
// 		// BitVec doesn't impl BitStore on u64 if pointer width isn't 64 bit, avoid using this store type here
// 		// in that case to avoid compile errors (see https://docs.rs/bitvec/1.0.0/src/bitvec/store.rs.html#184)
// 		#[cfg(not(feature = "32bit_target"))]
// 		(BitStoreTy::U64, BitOrderTy::Lsb0) => to_bit_sequence(BitVec::<u64, Lsb0>::decode(data)?),
// 		#[cfg(not(feature = "32bit_target"))]
// 		(BitStoreTy::U64, BitOrderTy::Msb0) => to_bit_sequence(BitVec::<u64, Msb0>::decode(data)?),
// 		#[cfg(feature = "32bit_target")]
// 		(BitStoreTy::U64, _) => {
// 			return Err(DecodeError::BitSequenceError(BitSequenceError::StoreTypeNotSupported(
// 				"u64 (pointer-width on this compile target is not 64)".into(),
// 			)))
// 		}
// 	};

// 	Ok(bits)
// }