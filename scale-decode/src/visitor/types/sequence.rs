// Copyright (C) 2023 Parity Technologies (UK) Ltd. (admin@parity.io)
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

use crate::visitor::{DecodeError, IgnoreVisitor, Visitor};
use codec::{Compact, Decode};
use scale_info::PortableRegistry;

/// This enables a visitor to decode items from a sequence type.
pub struct Sequence<'scale, 'info> {
    bytes: &'scale [u8],
    item_bytes: &'scale [u8],
    type_id: u32,
    types: &'info PortableRegistry,
    remaining: usize,
}

impl<'scale, 'info> Sequence<'scale, 'info> {
    pub(crate) fn new(
        bytes: &'scale [u8],
        type_id: u32,
        types: &'info PortableRegistry,
    ) -> Result<Sequence<'scale, 'info>, DecodeError> {
        // Sequences are prefixed with their length in bytes. Make a note of this,
        // as well as the number of bytes
        let item_bytes = &mut &*bytes;
        let len = <Compact<u64>>::decode(item_bytes)?.0 as usize;

        Ok(Sequence { bytes, item_bytes, type_id, types, remaining: len })
    }
    /// Skip over all bytes associated with this sequence. After calling this,
    /// [`Self::remaining_bytes()`] will represent the bytes after this sequence.
    pub fn skip(&mut self) -> Result<(), DecodeError> {
        while self.remaining > 0 {
            self.decode_item(IgnoreVisitor).transpose()?;
        }
        Ok(())
    }
    /// The bytes representing this sequence and anything following it.
    pub fn bytes(&self) -> &'scale [u8] {
        self.bytes
    }
    /// The bytes that have not yet been decoded in this sequence (this never includes the
    /// compact length preceeding the sequence items).
    pub fn remaining_bytes(&self) -> &'scale [u8] {
        self.item_bytes
    }
    /// The number of un-decoded items remaining in this sequence.
    pub fn remaining(&self) -> usize {
        self.remaining
    }
    /// Decode an item from the sequence by providing a visitor to handle it.
    pub fn decode_item<V: Visitor>(
        &mut self,
        visitor: V,
    ) -> Option<Result<V::Value<'scale>, V::Error>> {
        if self.remaining == 0 {
            return None;
        }

        let b = &mut self.item_bytes;
        // Don't return here; decrement bytes and remaining properly first and then return, so that
        // calling decode_item again works as expected.
        let res = crate::visitor::decode_with_visitor(b, self.type_id, self.types, visitor);
        self.item_bytes = *b;
        self.remaining -= 1;
        Some(res)
    }
}

impl<'scale, 'info> crate::visitor::DecodeItemIterator<'scale> for Sequence<'scale, 'info> {
    fn decode_item<'a, V: Visitor>(
        &mut self,
        visitor: V,
    ) -> Option<Result<V::Value<'scale>, V::Error>> {
        self.decode_item(visitor)
    }
}
