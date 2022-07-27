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

use codec::{
    Decode,
    Compact,
};
use crate::bit_sequence::{
    BitOrderTy,
    BitStoreTy,
};
use bitvec::{
    order::{Lsb0, Msb0},
    vec::BitVec,
};
use super::{
    DecodeError,
};

/// This represents a BitSequence, deferring decoding until the implementation wants to.
pub struct BitSequence<'a> {
    store: BitStoreTy,
    order: BitOrderTy,
    bytes: &'a [u8],
    decoded: bool
}

impl <'a> BitSequence<'a> {
    pub (crate) fn new(
        store: BitStoreTy,
        order: BitOrderTy,
        bytes: &'a [u8]
    ) -> Self {
        BitSequence {
            store,
            order,
            bytes,
            decoded: false
        }
    }
    pub (crate) fn skip_if_not_decoded(&mut self) -> Result<(), DecodeError> {
        if !self.decoded {
            // A bitvec is a compact encoded length, which is the number of
            // bits in the bitvec. Based on the store type we turn that into a number
            // of bytes we need to skip over.
            let data = &mut self.bytes;

            // Number of bits stored.
            let number_of_bits = <Compact<u64>>::decode(data)?.0 as usize;

            // How many bytes needed to store those bits?
            let number_of_bytes = number_of_bytes_needed(number_of_bits, self.store);

println!("ITEMS {number_of_bits}, bytes {number_of_bytes}");
            if number_of_bytes > data.len() {
                return Err(DecodeError::Eof)
            }

            // We only modify the bytes here when we're sure nothing will go wrong.
            self.bytes = *data;
            self.bytes = &self.bytes[number_of_bytes..];
        }
        Ok(())
    }
    pub (crate) fn bytes(&self) -> &'a [u8] {
        self.bytes
    }
    /// Decode the bit sequence, returning an enum that represents the type of bit sequence that
    /// was actually seen in the input.
    pub fn decode_bitsequence(&mut self) -> Result<BitSequenceValue, DecodeError> {
        if self.decoded {
            return Err(DecodeError::NothingLeftToDecode);
        }
        let data = &mut self.bytes;
        let bits = match (self.store, self.order) {
            (BitStoreTy::U8, BitOrderTy::Lsb0) => BitSequenceValue::U8Lsb0(BitVec::decode(data)?),
            (BitStoreTy::U8, BitOrderTy::Msb0) => BitSequenceValue::U8Msb0(BitVec::decode(data)?),
            (BitStoreTy::U16, BitOrderTy::Lsb0) => BitSequenceValue::U16Lsb0(BitVec::decode(data)?),
            (BitStoreTy::U16, BitOrderTy::Msb0) => BitSequenceValue::U16Msb0(BitVec::decode(data)?),
            (BitStoreTy::U32, BitOrderTy::Lsb0) => BitSequenceValue::U32Lsb0(BitVec::decode(data)?),
            (BitStoreTy::U32, BitOrderTy::Msb0) => BitSequenceValue::U32Msb0(BitVec::decode(data)?),
            // BitVec doesn't impl BitStore on u64 if pointer width isn't 64 bit, avoid using this store type here
            // in that case to avoid compile errors (see https://docs.rs/bitvec/1.0.0/src/bitvec/store.rs.html#184)
            #[cfg(not(feature = "32bit_target"))]
            (BitStoreTy::U64, BitOrderTy::Lsb0) => BitSequenceValue::U64Lsb0(BitVec::decode(data)?),
            #[cfg(not(feature = "32bit_target"))]
            (BitStoreTy::U64, BitOrderTy::Msb0) => BitSequenceValue::U64Msb0(BitVec::decode(data)?),
            #[cfg(feature = "32bit_target")]
            (BitStoreTy::U64, _) => {
                return Err(DecodeError::BitSequenceError(BitSequenceError::StoreTypeNotSupported(
                    "u64 (pointer-width on this compile target is not 64)".into(),
                )))
            }
        };
        self.bytes = *data;
        self.decoded = true;
        Ok(bits)
    }
}

fn number_of_bytes_needed(number_of_bits: usize, store: BitStoreTy) -> usize {
    let store_width_bits: usize = match store {
        BitStoreTy::U8 => 8,
        BitStoreTy::U16 => 16,
        BitStoreTy::U32 => 32,
        BitStoreTy::U64 => 64,
    };

    // This nifty code works out the number of bytes needed to
    // store the number of bits given.
    let number_of_bits_rounded_up = number_of_bits + (store_width_bits-1) & !(store_width_bits-1);

    // Round to the number of bytes needed.
    number_of_bits_rounded_up / 8
}

/// A decoded BitVec.
#[derive(Clone, PartialEq, Debug)]
pub enum BitSequenceValue {
    /// A bit sequence with a store type of `u8` and an order of `Lsb0`
    U8Lsb0(BitVec::<u8, Lsb0>),
    /// A bit sequence with a store type of `u8` and an order of `Msb0`
    U8Msb0(BitVec::<u8, Msb0>),
    /// A bit sequence with a store type of `u16` and an order of `Lsb0`
    U16Lsb0(BitVec::<u16, Lsb0>),
    /// A bit sequence with a store type of `u16` and an order of `Msb0`
    U16Msb0(BitVec::<u16, Msb0>),
    /// A bit sequence with a store type of `u32` and an order of `Lsb0`
    U32Lsb0(BitVec::<u32, Lsb0>),
    /// A bit sequence with a store type of `u32` and an order of `Msb0`
    U32Msb0(BitVec::<u32, Msb0>),
    /// A bit sequence with a store type of `u64` and an order of `Lsb0`
    U64Lsb0(BitVec::<u64, Lsb0>),
    /// A bit sequence with a store type of `u64` and an order of `Msb0`
    U64Msb0(BitVec::<u64, Msb0>),
}

#[cfg(test)]
mod test {
    use super::*;
    use bitvec::bitvec;
    use codec::Encode;

    fn encode_decode_bitseq<Bits: Encode>(bits: Bits, store: BitStoreTy, order: BitOrderTy) {
        let bytes = bits.encode();

        // Test skipping works:
        let mut seq = BitSequence::new(store, order, &bytes);
        seq.skip_if_not_decoded().expect("can skip bitseq without error");
        assert_eq!(seq.bytes().len(), 0,  "No bytes should remain after skipping over");

        // Test actual decoding works:
        let mut seq = BitSequence::new(store, order, &bytes);
        let new_bytes = match seq.decode_bitsequence().expect("can decode bytes") {
            BitSequenceValue::U8Lsb0(new_bits) if store == BitStoreTy::U8 && order == BitOrderTy::Lsb0 => new_bits.encode(),
            BitSequenceValue::U8Msb0(new_bits) if store == BitStoreTy::U8 && order == BitOrderTy::Msb0 => new_bits.encode(),
            BitSequenceValue::U16Lsb0(new_bits) if store == BitStoreTy::U16 && order == BitOrderTy::Lsb0 => new_bits.encode(),
            BitSequenceValue::U16Msb0(new_bits) if store == BitStoreTy::U16 && order == BitOrderTy::Msb0 => new_bits.encode(),
            BitSequenceValue::U32Lsb0(new_bits) if store == BitStoreTy::U32 && order == BitOrderTy::Lsb0 => new_bits.encode(),
            BitSequenceValue::U32Msb0(new_bits) if store == BitStoreTy::U32 && order == BitOrderTy::Msb0 => new_bits.encode(),
            BitSequenceValue::U64Lsb0(new_bits) if store == BitStoreTy::U64 && order == BitOrderTy::Lsb0 => new_bits.encode(),
            BitSequenceValue::U64Msb0(new_bits) if store == BitStoreTy::U64 && order == BitOrderTy::Msb0 => new_bits.encode(),
            v => panic!("Value {v:?} was unexpected given store {store:?} and order {order:?}")
        };

        assert_eq!(bytes, new_bytes, "Original encoded bytes don't line up to decoded bytes");
    }

    #[test]
    fn can_decode() {
        encode_decode_bitseq(bitvec![u8, Lsb0; 0, 0, 1, 1, 0, 1], BitStoreTy::U8, BitOrderTy::Lsb0);
        encode_decode_bitseq(bitvec![u16, Lsb0; 0, 0, 1, 1, 0, 1], BitStoreTy::U16, BitOrderTy::Lsb0);
        encode_decode_bitseq(bitvec![u32, Lsb0; 0, 0, 1, 1, 0, 1], BitStoreTy::U32, BitOrderTy::Lsb0);
        encode_decode_bitseq(bitvec![u64, Lsb0; 0, 0, 1, 1, 0, 1], BitStoreTy::U64, BitOrderTy::Lsb0);

        encode_decode_bitseq(bitvec![u8, Msb0; 0, 0, 1, 1, 0, 1], BitStoreTy::U8, BitOrderTy::Msb0);
        encode_decode_bitseq(bitvec![u16, Msb0; 0, 0, 1, 1, 0, 1], BitStoreTy::U16, BitOrderTy::Msb0);
        encode_decode_bitseq(bitvec![u32, Msb0; 0, 0, 1, 1, 0, 1], BitStoreTy::U32, BitOrderTy::Msb0);
        encode_decode_bitseq(bitvec![u64, Msb0; 0, 0, 1, 1, 0, 1], BitStoreTy::U64, BitOrderTy::Msb0);
    }

    #[test]
    fn number_of_bits() {
        let tests = vec![
            // u8
            (0, BitStoreTy::U8, 0),
            (1, BitStoreTy::U8, 1),
            (7, BitStoreTy::U8, 1),
            (8, BitStoreTy::U8, 1),
            (9, BitStoreTy::U8, 2),
            (16, BitStoreTy::U8, 2),
            (17, BitStoreTy::U8, 3),
            // u16
            (0, BitStoreTy::U16, 0),
            (1, BitStoreTy::U16, 2),
            (15, BitStoreTy::U16, 2),
            (16, BitStoreTy::U16, 2),
            (17, BitStoreTy::U16, 4),
            (32, BitStoreTy::U16, 4),
            (33, BitStoreTy::U16, 6),
            // u32
            (0, BitStoreTy::U32, 0),
            (1, BitStoreTy::U32, 4),
            (31, BitStoreTy::U32, 4),
            (32, BitStoreTy::U32, 4),
            (33, BitStoreTy::U32, 8),
            (64, BitStoreTy::U32, 8),
            (65, BitStoreTy::U32, 12),
            // u64
            (0, BitStoreTy::U64, 0),
            (1, BitStoreTy::U64, 8),
            (64, BitStoreTy::U64, 8),
            (65, BitStoreTy::U64, 16),
            (128, BitStoreTy::U64, 16),
            (129, BitStoreTy::U64, 24),
        ];
        for (bits, store, expected) in tests {
            assert_eq!(number_of_bytes_needed(bits, store), expected, "bits: {bits}, store: {store:?}, expected: {expected}");
        }
    }

}