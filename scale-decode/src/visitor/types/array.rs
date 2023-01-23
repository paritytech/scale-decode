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
use scale_info::PortableRegistry;

/// This enables a visitor to decode items from an array type.
pub struct Array<'scale, 'info> {
    bytes: &'scale [u8],
    item_bytes: &'scale [u8],
    type_id: u32,
    types: &'info PortableRegistry,
    remaining: usize,
}

impl<'scale, 'info> Array<'scale, 'info> {
    pub(crate) fn new(
        bytes: &'scale [u8],
        type_id: u32,
        len: usize,
        types: &'info PortableRegistry,
    ) -> Array<'scale, 'info> {
        Array { bytes, item_bytes: bytes, type_id, types, remaining: len }
    }
    /// Skip over all bytes associated with this array. After calling this,
    /// [`Self::remaining_bytes()`] will represent the bytes after this array.
    pub fn skip(&mut self) -> Result<(), DecodeError> {
        while self.remaining > 0 {
            self.decode_item(IgnoreVisitor).transpose()?;
        }
        Ok(())
    }
    /// The bytes representing this array and anything following it.
    pub fn bytes(&self) -> &'scale [u8] {
        self.bytes
    }
    /// The bytes that have not yet been decoded in this array.
    pub fn remaining_bytes(&self) -> &'scale [u8] {
        self.item_bytes
    }
    /// The number of un-decoded items remaining in this array.
    pub fn remaining(&self) -> usize {
        self.remaining
    }
    /// Are there any un-decoded items remaining in this array.
    pub fn is_empty(&self) -> bool {
        self.remaining == 0
    }
    /// Decode an item from the array by providing a visitor to handle it.
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

impl <'scale, 'info> crate::visitor::DecodeItemIterator<'scale> for Array<'scale, 'info> {
    fn decode_item<'a, V: Visitor>(&mut self, visitor: V) -> Option<Result<V::Value<'scale>, V::Error>> {
        self.decode_item(visitor)
    }
}