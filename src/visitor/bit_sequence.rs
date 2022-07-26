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
            // items of whatever the store type that are to follow. So, pluck
            // the length out and then skip over a number of bytes corresponding
            // to that number of store type items.
            let data = &mut self.bytes;
            let items_len = <Compact<u64>>::decode(data)?.0 as usize;
            let byte_len = match self.store {
                BitStoreTy::U8 => items_len,
                BitStoreTy::U16 => items_len * 2,
                BitStoreTy::U32 => items_len * 4,
                BitStoreTy::U64 => items_len * 8,
            };

            if byte_len > data.len() {
                return Err(DecodeError::Eof)
            }

            // We only modify the bytes here when we're sure nothing will go wrong.
            self.bytes = *data;
            self.bytes = &self.bytes[byte_len..];
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

/// A decoded BitVec.
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