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

use super::{DecodeError, IgnoreVisitor, Visitor};
use scale_info::{form::PortableForm, Field, PortableRegistry};

/// This represents a composite type.
pub struct Composite<'a, 'b> {
	bytes: &'a [u8],
	fields: &'b [Field<PortableForm>],
	types: &'b PortableRegistry,
}

impl<'a, 'b> Composite<'a, 'b> {
	pub(crate) fn new(
		bytes: &'a [u8],
		fields: &'b [Field<PortableForm>],
		types: &'b PortableRegistry,
	) -> Composite<'a, 'b> {
		Composite { bytes, fields, types }
	}
	pub(crate) fn bytes(&self) -> &'a [u8] {
		self.bytes
	}
	pub(crate) fn skip_rest(&mut self) -> Result<(), DecodeError> {
		while !self.fields.is_empty() {
			self.decode_item(IgnoreVisitor)?;
		}
		Ok(())
	}
	/// Do any of the fields in this composite type have names? Either all of them
	/// should be named, or none of them should be.
	pub fn fields(&self) -> &'b [Field<PortableForm>] {
		self.fields
	}
	/// Decode the next field in the composite type by providing a visitor to handle it.
	pub fn decode_item<V: Visitor>(&mut self, visitor: V) -> Result<Option<V::Value>, V::Error> {
		self.decode_item_with_name(visitor).map(|o| o.map(|(_n, v)| v))
	}
	/// Decode the next field in the composite type by providing a visitor to handle it.
	/// The name of the field will be returned too, or an empty string if it doesn't exist.
	pub fn decode_item_with_name<V: Visitor>(
		&mut self,
		visitor: V,
	) -> Result<Option<(&'b str, V::Value)>, V::Error> {
		if self.fields.is_empty() {
			return Ok(None);
		}

		let field = &self.fields[0];
		let field_name = self.fields.get(0).and_then(|f| f.name().map(|n| &**n)).unwrap_or("");
		let b = &mut self.bytes;

		// Don't return here; decrement bytes properly first and then return, so that
		// calling decode_item again works as expected.
		let res = crate::decode::decode(b, field.ty().id(), self.types, visitor);

		self.bytes = *b;
		self.fields = &self.fields[1..];

		res.map(|val| Some((field_name, val)))
	}
}
