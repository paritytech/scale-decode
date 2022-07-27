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
pub struct Composite<'a> {
	bytes: &'a [u8],
	fields: &'a [Field<PortableForm>],
	types: &'a PortableRegistry,
	len: usize,
}

impl<'a> Composite<'a> {
	pub(crate) fn new(
		bytes: &'a [u8],
		fields: &'a [Field<PortableForm>],
		types: &'a PortableRegistry,
	) -> Composite<'a> {
		Composite { len: fields.len(), bytes, fields, types }
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
	/// The number of items in this composite type.
	pub fn len(&self) -> usize {
		self.len
	}
	/// The number of un-decoded items remaining in this composite type.
	pub fn remaining(&self) -> usize {
		self.fields.len()
	}
	/// Decode the next field in the composite type by providing a visitor to handle it.
	pub fn decode_item<V: Visitor>(
		&mut self,
		visitor: V,
	) -> Result<Option<(Option<&'a str>, V::Value)>, V::Error> {
		if self.fields.is_empty() {
			return Ok(None);
		}

		let field = &self.fields[0];
		let field_name = self.fields.get(0).and_then(|f| f.name().map(|n| &**n));
		let b = &mut self.bytes;

		// Don't return here; decrement bytes properly first and then return, so that
		// calling decode_item again works as expected.
		let res = crate::decode::decode(b, field.ty().id(), self.types, visitor);

		self.bytes = *b;
		self.fields = &self.fields[1..];

		res.map(|val| Some((field_name, val)))
	}
}
