// Copyright (C) 2023 Parity Technologies (UK) Ltd. (admin@parity.io)
// This file is a part of the scale-decode crate.
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

use crate::{
    visitor::{DecodeError, IgnoreVisitor, Visitor},
    DecodeAsType, FieldIter,
};
use scale_info::PortableRegistry;

/// This represents a tuple of values.
pub struct Tuple<'scale, 'info, I> {
    bytes: &'scale [u8],
    item_bytes: &'scale [u8],
    fields: I,
    types: &'info PortableRegistry,
}

impl<'scale, 'info, I> Tuple<'scale, 'info, I>
where
    I: FieldIter<'info>,
{
    pub(crate) fn new(
        bytes: &'scale [u8],
        fields: I,
        types: &'info PortableRegistry,
    ) -> Tuple<'scale, 'info, I> {
        Tuple { bytes, item_bytes: bytes, fields, types }
    }
    /// Skip over all bytes associated with this tuple. After calling this,
    /// [`Self::bytes_from_undecoded()`] will represent the bytes after this tuple.
    pub fn skip_decoding(&mut self) -> Result<(), DecodeError> {
        while let Some(field) = self.fields.next() {
            self.decode_item_with_id(field.id(), IgnoreVisitor).transpose()?;
        }
        Ok(())
    }
    /// The bytes representing this tuple and anything following it.
    pub fn bytes_from_start(&self) -> &'scale [u8] {
        self.bytes
    }
    /// The bytes that have not yet been decoded in this tuple, and anything
    /// following it.
    pub fn bytes_from_undecoded(&self) -> &'scale [u8] {
        self.item_bytes
    }
    /// The number of un-decoded items remaining in the tuple.
    pub fn remaining(&self) -> usize {
        self.fields.clone().count()
    }
    /// Decode the next item from the tuple by providing a visitor to handle it.
    pub fn decode_item<V: Visitor>(
        &mut self,
        visitor: V,
    ) -> Option<Result<V::Value<'scale, 'info>, V::Error>> {
        let field = self.fields.next()?;
        self.decode_item_with_id(field.id(), visitor)
    }

    fn decode_item_with_id<V: Visitor>(
        &mut self,
        id: u32,
        visitor: V,
    ) -> Option<Result<V::Value<'scale, 'info>, V::Error>> {
        let b = &mut &*self.item_bytes;

        // Decode the bytes:
        let res = crate::visitor::decode_with_visitor(b, id, self.types, visitor);

        // Move our bytes cursor forwards:
        self.item_bytes = *b;

        Some(res)
    }
}

// Iterating returns a representation of each field in the tuple type.
impl<'scale, 'info, I> Iterator for Tuple<'scale, 'info, I>
where
    I: FieldIter<'info>,
{
    type Item = Result<TupleField<'scale, 'info>, DecodeError>;
    fn next(&mut self) -> Option<Self::Item> {
        let field = self.fields.next()?;

        // Record details we need before we decode and skip over the thing:
        let num_bytes_before = self.item_bytes.len();
        let item_bytes = self.item_bytes;

        // Now, skip over the item we're going to hand back:
        if let Err(e) = self.decode_item_with_id(field.id(), IgnoreVisitor)? {
            return Some(Err(e));
        };

        // How many bytes did we skip over? What bytes represent the thing we decoded?
        let num_bytes_after = self.item_bytes.len();
        let res_bytes = &item_bytes[..num_bytes_before - num_bytes_after];

        Some(Ok(TupleField { bytes: res_bytes, type_id: field.id(), types: self.types }))
    }
}

/// A single field in the tuple type.
#[derive(Copy, Clone)]
pub struct TupleField<'scale, 'info> {
    bytes: &'scale [u8],
    type_id: u32,
    types: &'info PortableRegistry,
}

impl<'scale, 'info> TupleField<'scale, 'info> {
    /// The bytes associated with this field.
    pub fn bytes(&self) -> &'scale [u8] {
        self.bytes
    }
    /// The type ID associated with this field.
    pub fn type_id(&self) -> u32 {
        self.type_id
    }
    /// Decode this field using a visitor.
    pub fn decode_with_visitor<V: Visitor>(
        &self,
        visitor: V,
    ) -> Result<V::Value<'scale, 'info>, V::Error> {
        crate::visitor::decode_with_visitor(&mut &*self.bytes, self.type_id, self.types, visitor)
    }
    /// Decode this field into a specific type via [`DecodeAsType`].
    pub fn decode_as_type<T: DecodeAsType>(&self) -> Result<T, crate::Error> {
        T::decode_as_type(&mut &*self.bytes, self.type_id, self.types)
    }
}

impl<'scale, 'info, I> crate::visitor::DecodeItemIterator<'scale, 'info> for Tuple<'scale, 'info, I>
where
    I: FieldIter<'info>,
{
    fn decode_item<'a, V: Visitor>(
        &mut self,
        visitor: V,
    ) -> Option<Result<V::Value<'scale, 'info>, V::Error>> {
        self.decode_item(visitor)
    }
}
