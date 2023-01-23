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
use scale_info::{form::PortableForm, Field, PortableRegistry};

/// This represents a composite type.
pub struct Composite<'scale, 'info> {
    bytes: &'scale [u8],
    item_bytes: &'scale [u8],
    fields: &'info [Field<PortableForm>],
    types: &'info PortableRegistry,
}

impl<'scale, 'info> Composite<'scale, 'info> {
    pub(crate) fn new(
        bytes: &'scale [u8],
        fields: &'info [Field<PortableForm>],
        types: &'info PortableRegistry,
    ) -> Composite<'scale, 'info> {
        Composite { bytes, item_bytes: bytes, fields, types }
    }
    /// Skip over all bytes associated with this composite type. After calling this,
    /// [`Self::remaining_bytes()`] will represent the bytes after this composite type.
    pub fn skip(&mut self) -> Result<(), DecodeError> {
        while !self.fields.is_empty() {
            self.decode_item(IgnoreVisitor).transpose()?;
        }
        Ok(())
    }
    /// The bytes representing this composite type and anything following it.
    pub fn bytes(&self) -> &'scale [u8] {
        self.bytes
    }
    /// The bytes that have not yet been decoded in this composite type.
    pub fn remaining_bytes(&self) -> &'scale [u8] {
        self.item_bytes
    }
    /// The number of un-decoded items remaining in this composite type.
    pub fn remaining(&self) -> usize {
        self.fields.len()
    }
    /// The yet-to-be-decoded fields still present in this composite type.
    pub fn fields(&self) -> &'info [Field<PortableForm>] {
        self.fields
    }
    /// Decode the next field in the composite type by providing a visitor to handle it.
    pub fn decode_item<V: Visitor>(
        &mut self,
        visitor: V,
    ) -> Option<Result<V::Value<'scale>, V::Error>> {
        self.decode_item_with_name(visitor).map(|o| o.map(|(_n, v)| v))
    }
    /// Decode the next field in the composite type by providing a visitor to handle it.
    /// The name of the field will be returned too, or an empty string if it doesn't exist.
    pub fn decode_item_with_name<V: Visitor>(
        &mut self,
        visitor: V,
    ) -> Option<Result<(&'info str, V::Value<'scale>), V::Error>> {
        if self.fields.is_empty() {
            return None;
        }

        let field = &self.fields[0];
        let field_name = self.fields.get(0).and_then(|f| f.name().map(|n| &**n)).unwrap_or("");
        let b = &mut self.item_bytes;

        // Don't return here; decrement bytes properly first and then return, so that
        // calling decode_item again works as expected.
        let res = crate::visitor::decode_with_visitor(b, field.ty().id(), self.types, visitor);

        self.item_bytes = *b;
        self.fields = &self.fields[1..];

        res.map(|val| Some((field_name, val))).transpose()
    }
}

impl <'scale, 'info> crate::visitor::DecodeItemIterator<'scale> for Composite<'scale, 'info> {
    fn decode_item<'a, V: Visitor>(&mut self, visitor: V) -> Option<Result<V::Value<'scale>, V::Error>> {
        self.decode_item(visitor)
    }
}