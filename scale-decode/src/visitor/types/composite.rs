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
    DecodeAsType, Field, FieldIter,
};
use scale_info::{form::PortableForm, Path, PortableRegistry};

/// This represents a composite type.
pub struct Composite<'scale, 'info, I> {
    bytes: &'scale [u8],
    item_bytes: &'scale [u8],
    path: &'info Path<PortableForm>,
    fields: I,
    types: &'info PortableRegistry,
}

impl<'scale, 'info, I> Composite<'scale, 'info, I>
where
    I: FieldIter<'info>,
{
    // Used in macros, but not really expected to be used elsewhere.
    #[doc(hidden)]
    pub fn new(
        bytes: &'scale [u8],
        path: &'info Path<PortableForm>,
        fields: I,
        types: &'info PortableRegistry,
    ) -> Composite<'scale, 'info, I> {
        Composite { bytes, path, item_bytes: bytes, fields, types }
    }
    /// Skip over all bytes associated with this composite type. After calling this,
    /// [`Self::bytes_from_undecoded()`] will represent the bytes after this composite type.
    pub fn skip_decoding(&mut self) -> Result<(), DecodeError> {
        while let Some(field) = self.fields.next() {
            self.decode_item_with_id(field.id(), IgnoreVisitor).transpose()?;
        }
        Ok(())
    }
    /// The bytes representing this composite type and anything following it.
    pub fn bytes_from_start(&self) -> &'scale [u8] {
        self.bytes
    }
    /// The bytes that have not yet been decoded in this composite type and anything
    /// following it.
    pub fn bytes_from_undecoded(&self) -> &'scale [u8] {
        self.item_bytes
    }
    /// The number of un-decoded items remaining in this composite type.
    pub fn remaining(&self) -> usize {
        self.fields.clone().count()
    }
    /// Path to this type.
    pub fn path(&self) -> &'info Path<PortableForm> {
        self.path
    }
    /// The yet-to-be-decoded fields still present in this composite type.
    pub fn fields(&self) -> I {
        self.fields.clone()
    }
    /// Return whether any of the fields are unnamed.
    pub fn has_unnamed_fields(&self) -> bool {
        self.fields.clone().any(|f| f.name().is_none())
    }
    /// Convert the remaining fields in this Composite type into a [`super::Tuple`]. This allows them to
    /// be parsed in the same way as a tuple type, discarding name information.
    pub fn as_tuple(&self) -> super::Tuple<'scale, 'info, I> {
        super::Tuple::new(self.item_bytes, self.fields.clone(), self.types)
    }
    /// Return the name of the next field to be decoded; `None` if either the field has no name,
    /// or there are no fields remaining.
    pub fn peek_name(&self) -> Option<&'info str> {
        self.fields.clone().next().and_then(|f| f.name())
    }
    /// Decode the next field in the composite type by providing a visitor to handle it. This is more
    /// efficient than iterating over the key/value pairs if you already know how you want to decode the
    /// values.
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

// Iterating returns a representation of each field in the composite type.
impl<'scale, 'info, I> Iterator for Composite<'scale, 'info, I>
where
    I: FieldIter<'info>,
{
    type Item = Result<CompositeField<'scale, 'info>, DecodeError>;
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

        Some(Ok(CompositeField { bytes: res_bytes, field, types: self.types }))
    }
}

/// A single field in the composite type.
#[derive(Copy, Clone)]
pub struct CompositeField<'scale, 'info> {
    bytes: &'scale [u8],
    field: Field<'info>,
    types: &'info PortableRegistry,
}

impl<'scale, 'info> CompositeField<'scale, 'info> {
    /// The field name.
    pub fn name(&self) -> Option<&'info str> {
        self.field.name()
    }
    /// The bytes associated with this field.
    pub fn bytes(&self) -> &'scale [u8] {
        self.bytes
    }
    /// The type ID associated with this field.
    pub fn type_id(&self) -> u32 {
        self.field.id()
    }
    /// Decode this field using a visitor.
    pub fn decode_with_visitor<V: Visitor>(
        &self,
        visitor: V,
    ) -> Result<V::Value<'scale, 'info>, V::Error> {
        crate::visitor::decode_with_visitor(&mut &*self.bytes, self.field.id(), self.types, visitor)
    }
    /// Decode this field into a specific type via [`DecodeAsType`].
    pub fn decode_as_type<T: DecodeAsType>(&self) -> Result<T, crate::Error> {
        T::decode_as_type(&mut &*self.bytes, self.field.id(), self.types)
    }
}

impl<'scale, 'info, I> crate::visitor::DecodeItemIterator<'scale, 'info>
    for Composite<'scale, 'info, I>
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
