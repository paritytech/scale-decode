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

use super::DecodeError;
use scale_bits::{
	scale::{decode_using_format_from, Decoder},
	Format,
};

/// This represents a BitSequence, deferring decoding until the implementation wants to.
pub struct BitSequence<'a> {
	format: Format,
	bytes: &'a [u8],
}

impl<'a> BitSequence<'a> {
	pub(crate) fn new(format: Format, bytes: &'a [u8]) -> Self {
		BitSequence { format, bytes }
	}

	/// The bytes after this bit sequence.
	pub(crate) fn remaining_bytes(&mut self) -> Result<&'a [u8], DecodeError> {
		let decoder = decode_using_format_from(self.bytes, self.format)?;
		let num_bytes = decoder.encoded_size();
		Ok(&self.bytes[num_bytes..])
	}

	/// Return a decoder to decode the bits in this bit sequence.
	pub fn decode(&mut self) -> Result<Decoder<'_>, DecodeError> {
		let decoder = decode_using_format_from(self.bytes, self.format)?;
		Ok(decoder)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use bitvec::{
		order::{Lsb0, Msb0},
		vec::BitVec,
	};
	use codec::Encode;
	use scale_bits::{
		bits,
		scale::format::{OrderFormat, StoreFormat},
		Bits,
	};

	fn assert_remaining_bytes_works<Input: Encode>(
		bits: Input,
		store: StoreFormat,
		order: OrderFormat,
	) {
		let bytes = bits.encode();
		let format = Format::new(store, order);

		// Test skipping works:
		let mut seq = BitSequence::new(format, &bytes);
		let leftover = seq.remaining_bytes().expect("can skip bitseq without error");
		assert_eq!(leftover.len(), 0, "No bytes should remain after skipping over");
	}

	fn assert_remaining_bytes_works_all(bits: Bits) {
		let b: BitVec<u8, Lsb0> = bits.iter().collect();
		assert_remaining_bytes_works(b, StoreFormat::U8, OrderFormat::Lsb0);
		let b: BitVec<u16, Lsb0> = bits.iter().collect();
		assert_remaining_bytes_works(b, StoreFormat::U16, OrderFormat::Lsb0);
		let b: BitVec<u32, Lsb0> = bits.iter().collect();
		assert_remaining_bytes_works(b, StoreFormat::U32, OrderFormat::Lsb0);
		let b: BitVec<u64, Lsb0> = bits.iter().collect();
		assert_remaining_bytes_works(b, StoreFormat::U64, OrderFormat::Lsb0);
		let b: BitVec<u8, Msb0> = bits.iter().collect();
		assert_remaining_bytes_works(b, StoreFormat::U8, OrderFormat::Msb0);
		let b: BitVec<u16, Msb0> = bits.iter().collect();
		assert_remaining_bytes_works(b, StoreFormat::U16, OrderFormat::Msb0);
		let b: BitVec<u32, Msb0> = bits.iter().collect();
		assert_remaining_bytes_works(b, StoreFormat::U32, OrderFormat::Msb0);
		let b: BitVec<u64, Msb0> = bits.iter().collect();
		assert_remaining_bytes_works(b, StoreFormat::U64, OrderFormat::Msb0);
	}

	#[test]
	fn skipping_remaining_bytes_works() {
		assert_remaining_bytes_works_all(bits![]);
		assert_remaining_bytes_works_all(bits![0]);
		assert_remaining_bytes_works_all(bits![0, 1]);
		assert_remaining_bytes_works_all(bits![1, 0, 1, 1, 0, 1, 1, 0, 1]);
		assert_remaining_bytes_works_all(bits![
			1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 1, 0, 1, 1, 0, 0, 1, 1, 1
		]);
	}
}
