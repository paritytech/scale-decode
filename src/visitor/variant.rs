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

use super::{composite::Composite, DecodeError};
use scale_info::form::PortableForm;

/// A representation of the a variant type.
pub struct Variant<'a, 'b> {
	variant: &'b scale_info::Variant<PortableForm>,
	fields: Composite<'a, 'b>,
}

impl<'a, 'b> Variant<'a, 'b> {
	pub(crate) fn new(
		variant: &'b scale_info::Variant<PortableForm>,
		fields: Composite<'a, 'b>,
	) -> Variant<'a, 'b> {
		Variant { variant, fields }
	}
	pub(crate) fn bytes(&self) -> &'a [u8] {
		self.fields.bytes()
	}
	pub(crate) fn skip_rest(&mut self) -> Result<(), DecodeError> {
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
	/// Access the variant fields.
	pub fn fields(&mut self) -> &mut Composite<'a, 'b> {
		&mut self.fields
	}
}
