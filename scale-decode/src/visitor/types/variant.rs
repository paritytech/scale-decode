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

use crate::visitor::{Composite, DecodeError};
use scale_type_resolver::{FieldIter, TypeResolver, VariantIter};

/// A representation of the a variant type.
pub struct Variant<'scale, 'info, R: TypeResolver> {
    bytes: &'scale [u8],
    variant_name: &'info str,
    variant_index: u8,
    fields: Composite<'scale, 'info, R>,
}

impl<'scale, 'info, R: TypeResolver> Variant<'scale, 'info, R> {
    pub(crate) fn new<
        Fields: FieldIter<'info, R::TypeId> + 'info,
        Variants: VariantIter<'info, Fields>,
    >(
        bytes: &'scale [u8],
        mut variants: Variants,
        types: &'info R,
    ) -> Result<Variant<'scale, 'info, R>, DecodeError> {
        let index = *bytes.first().ok_or(DecodeError::NotEnoughInput)?;
        let item_bytes = &bytes[1..];

        // Does a variant exist with the index we're looking for?
        let mut variant = variants
            .find(|v| v.index == index)
            .ok_or_else(|| DecodeError::VariantNotFound(index))?;

        // Allow decoding of the fields:
        // let mut fields_iter = variant.fields.map(|f| Field::new(f.id, f.name.as_deref()));
        let fields = Composite::new(item_bytes, &mut variant.fields, types, false);

        Ok(Variant { bytes, variant_index: index, variant_name: &variant.name, fields })
    }
}

impl<'scale, 'info, R: TypeResolver> Variant<'scale, 'info, R> {
    /// Skip over all bytes associated with this variant. After calling this,
    /// [`Self::bytes_from_undecoded()`] will represent the bytes after this variant.
    pub fn skip_decoding(&mut self) -> Result<(), DecodeError> {
        self.fields.skip_decoding()
    }
    /// The bytes representing this sequence and anything following it.
    pub fn bytes_from_start(&self) -> &'scale [u8] {
        self.bytes
    }
    /// The bytes that have not yet been decoded in this variant (this never includes the
    /// variant index at the front) and anything following it.
    pub fn bytes_from_undecoded(&self) -> &'scale [u8] {
        self.fields.bytes_from_undecoded()
    }
    /// The name of the variant.
    pub fn name(&self) -> &'info str {
        &self.variant_name
    }
    /// The index of the variant.
    pub fn index(&self) -> u8 {
        self.variant_index
    }
    /// Access the variant fields.
    pub fn fields(&mut self) -> &mut Composite<'scale, 'info, R> {
        &mut self.fields
    }
}
