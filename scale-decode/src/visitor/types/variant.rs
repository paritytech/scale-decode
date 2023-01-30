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

use crate::visitor::{Composite, DecodeError};
use scale_info::form::PortableForm;
use scale_info::{PortableRegistry, TypeDefVariant};

/// A representation of the a variant type.
pub struct Variant<'scale, 'info> {
    bytes: &'scale [u8],
    variant: &'info scale_info::Variant<PortableForm>,
    fields: Composite<'scale, 'info>,
}

impl<'scale, 'info> Variant<'scale, 'info> {
    pub(crate) fn new(
        bytes: &'scale [u8],
        ty: &'info TypeDefVariant<PortableForm>,
        types: &'info PortableRegistry,
    ) -> Result<Variant<'scale, 'info>, DecodeError> {
        let index = *bytes.first().ok_or(DecodeError::NotEnoughInput)?;
        let item_bytes = &bytes[1..];

        // Does a variant exist with the index we're looking for?
        let variant = ty
            .variants()
            .iter()
            .find(|v| v.index() == index)
            .ok_or_else(|| DecodeError::VariantNotFound(index, ty.clone()))?;

        // Allow decoding of the fields:
        let fields = Composite::new(item_bytes, variant.fields(), types);

        Ok(Variant { bytes, variant, fields })
    }
    /// Skip over all bytes associated with this variant. After calling this,
    /// [`Self::remaining_bytes()`] will represent the bytes after this variant.
    pub fn skip(&mut self) -> Result<(), DecodeError> {
        self.fields.skip()
    }
    /// The bytes representing this sequence and anything following it.
    pub fn bytes(&self) -> &'scale [u8] {
        self.bytes
    }
    /// The bytes that have not yet been decoded in this variant (this never includes the
    /// variant index at the front).
    pub fn remaining_bytes(&self) -> &'scale [u8] {
        self.fields.remaining_bytes()
    }
    /// The name of the variant.
    pub fn name(&self) -> &str {
        self.variant.name()
    }
    /// The index of the variant.
    pub fn index(&self) -> u8 {
        self.variant.index()
    }
    /// Access the variant fields.
    pub fn fields(&mut self) -> &mut Composite<'scale, 'info> {
        &mut self.fields
    }
}