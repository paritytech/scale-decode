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

// impl Drop

// #[cfg(test)]
// mod test {
// 	use super::*;
// 	use bitvec::bitvec;
// 	use codec::Encode;

// 	fn encode_decode_bitseq<Bits: Encode>(bits: Bits, store: BitStoreTy, order: BitOrderTy) {
// 		let bytes = bits.encode();

// 		// Test skipping works:
// 		let mut seq = BitSequence::new(store, order, &bytes);
// 		seq.skip_if_not_decoded().expect("can skip bitseq without error");
// 		assert_eq!(seq.bytes().len(), 0, "No bytes should remain after skipping over");

// 		// Test actual decoding works:
// 		let mut seq = BitSequence::new(store, order, &bytes);
// 		let new_bytes = match seq.decode_bitsequence().expect("can decode bytes") {
// 			BitSequenceValue::U8Lsb0(new_bits)
// 				if store == BitStoreTy::U8 && order == BitOrderTy::Lsb0 =>
// 			{
// 				new_bits.encode()
// 			}
// 			BitSequenceValue::U8Msb0(new_bits)
// 				if store == BitStoreTy::U8 && order == BitOrderTy::Msb0 =>
// 			{
// 				new_bits.encode()
// 			}
// 			BitSequenceValue::U16Lsb0(new_bits)
// 				if store == BitStoreTy::U16 && order == BitOrderTy::Lsb0 =>
// 			{
// 				new_bits.encode()
// 			}
// 			BitSequenceValue::U16Msb0(new_bits)
// 				if store == BitStoreTy::U16 && order == BitOrderTy::Msb0 =>
// 			{
// 				new_bits.encode()
// 			}
// 			BitSequenceValue::U32Lsb0(new_bits)
// 				if store == BitStoreTy::U32 && order == BitOrderTy::Lsb0 =>
// 			{
// 				new_bits.encode()
// 			}
// 			BitSequenceValue::U32Msb0(new_bits)
// 				if store == BitStoreTy::U32 && order == BitOrderTy::Msb0 =>
// 			{
// 				new_bits.encode()
// 			}
// 			BitSequenceValue::U64Lsb0(new_bits)
// 				if store == BitStoreTy::U64 && order == BitOrderTy::Lsb0 =>
// 			{
// 				new_bits.encode()
// 			}
// 			BitSequenceValue::U64Msb0(new_bits)
// 				if store == BitStoreTy::U64 && order == BitOrderTy::Msb0 =>
// 			{
// 				new_bits.encode()
// 			}
// 			v => panic!("Value {v:?} was unexpected given store {store:?} and order {order:?}"),
// 		};

// 		assert_eq!(bytes, new_bytes, "Original encoded bytes don't line up to decoded bytes");
// 	}

// 	#[test]
// 	fn can_decode() {
// 		encode_decode_bitseq(bitvec![u8, Lsb0; 0, 0, 1, 1, 0, 1], BitStoreTy::U8, BitOrderTy::Lsb0);
// 		encode_decode_bitseq(
// 			bitvec![u16, Lsb0; 0, 0, 1, 1, 0, 1],
// 			BitStoreTy::U16,
// 			BitOrderTy::Lsb0,
// 		);
// 		encode_decode_bitseq(
// 			bitvec![u32, Lsb0; 0, 0, 1, 1, 0, 1],
// 			BitStoreTy::U32,
// 			BitOrderTy::Lsb0,
// 		);
// 		encode_decode_bitseq(
// 			bitvec![u64, Lsb0; 0, 0, 1, 1, 0, 1],
// 			BitStoreTy::U64,
// 			BitOrderTy::Lsb0,
// 		);

// 		encode_decode_bitseq(bitvec![u8, Msb0; 0, 0, 1, 1, 0, 1], BitStoreTy::U8, BitOrderTy::Msb0);
// 		encode_decode_bitseq(
// 			bitvec![u16, Msb0; 0, 0, 1, 1, 0, 1],
// 			BitStoreTy::U16,
// 			BitOrderTy::Msb0,
// 		);
// 		encode_decode_bitseq(
// 			bitvec![u32, Msb0; 0, 0, 1, 1, 0, 1],
// 			BitStoreTy::U32,
// 			BitOrderTy::Msb0,
// 		);
// 		encode_decode_bitseq(
// 			bitvec![u64, Msb0; 0, 0, 1, 1, 0, 1],
// 			BitStoreTy::U64,
// 			BitOrderTy::Msb0,
// 		);
// 	}

// 	#[test]
// 	fn number_of_bits() {
// 		let tests = vec![
// 			// u8
// 			(0, BitStoreTy::U8, 0),
// 			(1, BitStoreTy::U8, 1),
// 			(7, BitStoreTy::U8, 1),
// 			(8, BitStoreTy::U8, 1),
// 			(9, BitStoreTy::U8, 2),
// 			(16, BitStoreTy::U8, 2),
// 			(17, BitStoreTy::U8, 3),
// 			// u16
// 			(0, BitStoreTy::U16, 0),
// 			(1, BitStoreTy::U16, 2),
// 			(15, BitStoreTy::U16, 2),
// 			(16, BitStoreTy::U16, 2),
// 			(17, BitStoreTy::U16, 4),
// 			(32, BitStoreTy::U16, 4),
// 			(33, BitStoreTy::U16, 6),
// 			// u32
// 			(0, BitStoreTy::U32, 0),
// 			(1, BitStoreTy::U32, 4),
// 			(31, BitStoreTy::U32, 4),
// 			(32, BitStoreTy::U32, 4),
// 			(33, BitStoreTy::U32, 8),
// 			(64, BitStoreTy::U32, 8),
// 			(65, BitStoreTy::U32, 12),
// 			// u64
// 			(0, BitStoreTy::U64, 0),
// 			(1, BitStoreTy::U64, 8),
// 			(64, BitStoreTy::U64, 8),
// 			(65, BitStoreTy::U64, 16),
// 			(128, BitStoreTy::U64, 16),
// 			(129, BitStoreTy::U64, 24),
// 		];
// 		for (bits, store, expected) in tests {
// 			assert_eq!(
// 				number_of_bytes_needed(bits, store),
// 				expected,
// 				"bits: {bits}, store: {store:?}, expected: {expected}"
// 			);
// 		}
// 	}
// }
