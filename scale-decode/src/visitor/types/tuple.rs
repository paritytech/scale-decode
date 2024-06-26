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
use scale_type_resolver::{Field, TypeResolver};

/// This represents a tuple of values.
pub struct Tuple<'scale, 'resolver, R: TypeResolver> {
    bytes: &'scale [u8],
    item_bytes: &'scale [u8],
    fields: smallvec::SmallVec<[Field<'resolver, R::TypeId>; 16]>,
    next_field_idx: usize,
    types: &'resolver R,
    is_compact: bool,
}

impl<'scale, 'resolver, R: TypeResolver> Tuple<'scale, 'resolver, R> {
    pub(crate) fn new(
        bytes: &'scale [u8],
        fields: &mut dyn FieldIter<'resolver, R::TypeId>,
        types: &'resolver R,
        is_compact: bool,
    ) -> Tuple<'scale, 'resolver, R> {
        let fields = smallvec::SmallVec::from_iter(fields);
        Tuple { bytes, item_bytes: bytes, fields, types, next_field_idx: 0, is_compact }
    }
    /// Skip over all bytes associated with this tuple. After calling this,
    /// [`Self::bytes_from_undecoded()`] will represent the bytes after this tuple.
    pub fn skip_decoding(&mut self) -> Result<(), DecodeError> {
        while let Some(res) = self.decode_item(IgnoreVisitor::<R>::new()) {
            res?;
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
        self.fields.len()
    }
    /// Decode the next item from the tuple by providing a visitor to handle it.
    pub fn decode_item<V: Visitor<TypeResolver = R>>(
        &mut self,
        visitor: V,
    ) -> Option<Result<V::Value<'scale, 'resolver>, V::Error>> {
        let field = self.fields.get(self.next_field_idx)?;
        let b = &mut &*self.item_bytes;
        // Decode the bytes:
        let res = crate::visitor::decode_with_visitor_maybe_compact(
            b,
            field.id.clone(),
            self.types,
            visitor,
            self.is_compact,
        );

        if res.is_ok() {
            // Move our cursors forwards only if decode was OK:
            self.item_bytes = *b;
            self.next_field_idx += 1;
        } else {
            // Otherwise, skip to end to prevent any future iterations:
            self.next_field_idx = self.fields.len()
        }

        Some(res)
    }
}

// Iterating returns a representation of each field in the tuple type.
impl<'scale, 'resolver, R: TypeResolver> Iterator for Tuple<'scale, 'resolver, R> {
    type Item = Result<TupleField<'scale, 'resolver, R>, DecodeError>;
    fn next(&mut self) -> Option<Self::Item> {
        // Record details we need before we decode and skip over the thing:
        let field = self.fields.get(self.next_field_idx)?.clone();
        let num_bytes_before = self.item_bytes.len();
        let item_bytes = self.item_bytes;

        // Now, decode and skip over the item we're going to hand back:
        if let Err(e) = self.decode_item(IgnoreVisitor::<R>::new())? {
            return Some(Err(e));
        };

        // How many bytes did we skip over? What bytes represent the thing we decoded?
        let num_bytes_after = self.item_bytes.len();
        let res_bytes = &item_bytes[..num_bytes_before - num_bytes_after];

        Some(Ok(TupleField {
            bytes: res_bytes,
            type_id: field.id,
            types: self.types,
            is_compact: self.is_compact,
        }))
    }
}

/// A single field in the tuple type.
#[derive(Copy, Clone, Debug)]
pub struct TupleField<'scale, 'resolver, R: TypeResolver> {
    bytes: &'scale [u8],
    type_id: R::TypeId,
    types: &'resolver R,
    is_compact: bool,
}

impl<'scale, 'resolver, R: TypeResolver> TupleField<'scale, 'resolver, R> {
    /// The bytes associated with this field.
    pub fn bytes(&self) -> &'scale [u8] {
        self.bytes
    }
    /// The type ID associated with this field.
    pub fn type_id(&self) -> &R::TypeId {
        &self.type_id
    }
    /// If the field is compact encoded
    pub fn is_compact(&self) -> bool {
        self.is_compact
    }
    /// Decode this field using a visitor.
    pub fn decode_with_visitor<V: Visitor<TypeResolver = R>>(
        &self,
        visitor: V,
    ) -> Result<V::Value<'scale, 'resolver>, V::Error> {
        crate::visitor::decode_with_visitor(
            &mut &*self.bytes,
            self.type_id.clone(),
            self.types,
            visitor,
        )
    }
    /// Decode this field into a specific type via [`DecodeAsType`].
    pub fn decode_as_type<T: DecodeAsType>(&self) -> Result<T, crate::Error> {
        T::decode_as_type_maybe_compact(
            &mut &*self.bytes,
            self.type_id.clone(),
            self.types,
            self.is_compact,
        )
    }
}

impl<'scale, 'resolver, R: TypeResolver> crate::visitor::DecodeItemIterator<'scale, 'resolver, R>
    for Tuple<'scale, 'resolver, R>
{
    fn decode_item<'a, V: Visitor<TypeResolver = R>>(
        &mut self,
        visitor: V,
    ) -> Option<Result<V::Value<'scale, 'resolver>, V::Error>> {
        self.decode_item(visitor)
    }
}
