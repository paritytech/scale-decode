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

use crate::{
    visitor::{DecodeError, IgnoreVisitor, Visitor},
    DecodeAsType
};
use scale_info::PortableRegistry;

/// This represents a tuple of values.
pub struct Tuple<'scale, 'info> {
    bytes: &'scale [u8],
    item_bytes: &'scale [u8],
    fields: TupleFieldIds<'info>,
    types: &'info PortableRegistry,
}

impl<'scale, 'info> Tuple<'scale, 'info> {
    pub(crate) fn new(
        bytes: &'scale [u8],
        fields: impl Into<TupleFieldIds<'info>>,
        types: &'info PortableRegistry,
    ) -> Tuple<'scale, 'info> {
        Tuple { bytes, item_bytes: bytes, fields: fields.into(), types }
    }
    /// Skip over all bytes associated with this tuple. After calling this,
    /// [`Self::remaining_bytes()`] will represent the bytes after this tuple.
    pub fn skip_decoding(&mut self) -> Result<(), DecodeError> {
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
        self.next()
            .map(|res| res.map_err(|e| e.into()).and_then(|item| item.decode_with_visitor(visitor)))
    }
}

// Iterating returns a representation of each field in the tuple type.
impl<'scale, 'info> Iterator for Tuple<'scale, 'info> {
    type Item = Result<TupleField<'scale, 'info>, DecodeError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.fields.is_empty() {
            return None;
        }

        let type_id = self.fields.first().expect("not empty; checked above");
        let b = &mut &*self.item_bytes;

        // Skip over the bytes, ignoring them:
        let res =
            crate::visitor::decode_with_visitor(b, type_id, self.types, IgnoreVisitor);

        // Pull out the bytes representing the thing we just skipped over:
        let num_bytes_before = self.item_bytes.len();
        let num_bytes_after = b.len();
        let res_bytes = &self.item_bytes[..num_bytes_before - num_bytes_after];

        // Update self to point to the next item, now:
        self.fields.pop_front_unwrap();
        self.item_bytes = *b;

        Some(res.map(|()| TupleField {
            bytes: res_bytes,
            type_id,
            types: self.types
        }))
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
    /// Decode this field using a visitor.
    pub fn decode_with_visitor<V: Visitor>(
        &self,
        visitor: V,
    ) -> Result<V::Value<'scale>, V::Error> {
        crate::visitor::decode_with_visitor(
            &mut &*self.bytes,
            self.type_id,
            self.types,
            visitor,
        )
    }
    /// Decode this field into a specific type via [`DecodeAsType`].
    pub fn decode_as_type<T: DecodeAsType>(&self) -> Result<T, crate::Error> {
        T::decode_as_type(&mut &*self.bytes, self.type_id, self.types)
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

pub enum TupleFieldIds<'info> {
    Ids(&'info [scale_info::interner::UntrackedSymbol<std::any::TypeId>]),
    Fields(&'info [scale_info::Field<scale_info::form::PortableForm>])
}

impl <'info> TupleFieldIds<'info> {
    fn len(&self) -> usize {
        match self {
            TupleFieldIds::Ids(fs) => fs.len(),
            TupleFieldIds::Fields(fs) => fs.len(),
        }
    }
    fn is_empty(&self) -> bool {
        match self {
            TupleFieldIds::Ids(fs) => fs.is_empty(),
            TupleFieldIds::Fields(fs) => fs.is_empty(),
        }
    }
    fn first(&self) -> Option<u32> {
        match self {
            TupleFieldIds::Ids(fs) => fs.get(0).map(|f| f.id()),
            TupleFieldIds::Fields(fs) => fs.get(0).map(|f| f.ty().id()),
        }
    }
    fn pop_front_unwrap(&mut self) {
        match self {
            TupleFieldIds::Ids(fs) => { *fs = &fs[1..]; },
            TupleFieldIds::Fields(fs) => { *fs = &fs[1..]; },
        }
    }
}

impl <'info> From<&'info [scale_info::interner::UntrackedSymbol<std::any::TypeId>]> for TupleFieldIds<'info> {
    fn from(fields: &'info [scale_info::interner::UntrackedSymbol<std::any::TypeId>]) -> Self {
        TupleFieldIds::Ids(fields)
    }
}
impl <'info> From<&'info [scale_info::Field<scale_info::form::PortableForm>]> for TupleFieldIds<'info> {
    fn from(fields: &'info [scale_info::Field<scale_info::form::PortableForm>]) -> Self {
        TupleFieldIds::Fields(fields)
    }
}