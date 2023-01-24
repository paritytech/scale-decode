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

/// This represents a tuple of values.
pub struct Tuple<'scale, 'info> {
    bytes: &'scale [u8],
    item_bytes: &'scale [u8],
    fields: &'info [scale_info::interner::UntrackedSymbol<std::any::TypeId>],
    types: &'info PortableRegistry,
}

impl<'scale, 'info> Tuple<'scale, 'info> {
    pub(crate) fn new(
        bytes: &'scale [u8],
        fields: &'info [scale_info::interner::UntrackedSymbol<std::any::TypeId>],
        types: &'info PortableRegistry,
    ) -> Tuple<'scale, 'info> {
        Tuple { bytes, item_bytes: bytes, fields, types }
    }
    /// Skip over all bytes associated with this tuple. After calling this,
    /// [`Self::remaining_bytes()`] will represent the bytes after this tuple.
    pub fn skip(&mut self) -> Result<(), DecodeError> {
        while !self.fields.is_empty() {
            self.decode_item(IgnoreVisitor).transpose()?;
        }
        Ok(())
    }
    /// The bytes representing this tuple and anything following it.
    pub fn bytes(&self) -> &'scale [u8] {
        self.bytes
    }
    /// The bytes that have not yet been decoded in this tuple.
    pub fn remaining_bytes(&self) -> &'scale [u8] {
        self.item_bytes
    }
    /// The number of un-decoded items remaining in the tuple.
    pub fn remaining(&self) -> usize {
        self.fields.len()
    }
    /// Decode the next item from the tuple by providing a visitor to handle it.
    pub fn decode_item<V: Visitor>(
        &mut self,
        visitor: V,
    ) -> Option<Result<V::Value<'scale>, V::Error>> {
        if self.fields.is_empty() {
            return None;
        }

        let field = &self.fields[0];
        let b = &mut self.item_bytes;

        // Don't return here; decrement bytes properly first and then return, so that
        // calling decode_item again works as expected.
        let res = crate::visitor::decode_with_visitor(b, field.id(), self.types, visitor);

        self.fields = &self.fields[1..];
        self.item_bytes = *b;

        Some(res)
    }
}

impl<'scale, 'info> crate::visitor::DecodeItemIterator<'scale> for Tuple<'scale, 'info> {
    fn decode_item<'a, V: Visitor>(
        &mut self,
        visitor: V,
    ) -> Option<Result<V::Value<'scale>, V::Error>> {
        self.decode_item(visitor)
    }
}
