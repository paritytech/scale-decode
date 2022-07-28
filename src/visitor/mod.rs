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
mod sequence;
mod str;
mod tuple;
mod variant;

use crate::utils::bit_sequence::BitSequenceError;
use scale_info::form::PortableForm;

pub use self::str::Str;
pub use array::Array;
pub use bit_sequence::{BitSequence, BitSequenceValue};
pub use compact::{Compact, CompactLocation};
pub use composite::Composite;
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
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
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
