// Copyright (C) 2022 Parity Technologies (UK) Ltd. (admin@parity.io)
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

use scale_info::{
    form::PortableForm,
};
use super::{
    composite::Composite,
    DecodeError,
    Visitor,
};

/// A representation of the a variant type.
pub struct Variant<'a> {
    variant: &'a scale_info::Variant<PortableForm>,
    fields: Composite<'a>
}

impl <'a> Variant<'a> {
    pub (crate) fn new(
        variant: &'a scale_info::Variant<PortableForm>,
        fields: Composite<'a>,
    ) -> Variant<'a> {
        Variant { variant, fields }
    }
    pub (crate) fn bytes(&self) -> &'a [u8] {
        self.fields.bytes()
    }
    pub (crate) fn skip_rest(&mut self) -> Result<(), DecodeError> {
        self.fields.skip_rest()
    }
    /// The name of the variant.
    pub fn name(&self) -> &str {
        self.variant.name()
    }
    /// The index of the variant.
    pub fn index(&self) -> u8 {
        self.variant.index()
    }
    /// The number of fields in the variant.
    pub fn len(&self) -> usize {
        self.fields.len()
    }
    /// The number of un-decoded fields remaining in the variant.
    pub fn remaining(&self) -> usize {
        self.fields.remaining()
    }
    /// The name of the next field to be decoded, if it has one.
    pub fn next_field_name(&self) -> Option<&str> {
        self.fields.next_field_name()
    }
    /// Decode the next field in the variant by providing a visitor to handle it.
    pub fn decode_item<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, V::Error> {
        self.fields.decode_item(visitor)
    }
}