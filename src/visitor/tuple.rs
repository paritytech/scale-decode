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
use scale_info::PortableRegistry;

/// This represents a tuple of values.
pub struct Tuple<'a, 'b> {
	bytes: &'a [u8],
	fields: &'b [scale_info::interner::UntrackedSymbol<std::any::TypeId>],
	types: &'b PortableRegistry,
}

impl<'a, 'b> Tuple<'a, 'b> {
	pub(crate) fn new(
		bytes: &'a [u8],
		fields: &'b [scale_info::interner::UntrackedSymbol<std::any::TypeId>],
		types: &'b PortableRegistry,
	) -> Tuple<'a, 'b> {
		Tuple { bytes, fields, types }
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
	/// The number of un-decoded items remaining in the tuple.
	pub fn len(&self) -> usize {
		self.fields.len()
	}
	/// Are there any un-decoded items remaining in the tuple.
	pub fn is_empty(&self) -> bool {
		self.fields.is_empty()
	}
	/// Decode the next item from the tuple by providing a visitor to handle it.
	pub fn decode_item<V: Visitor>(&mut self, visitor: V) -> Result<Option<V::Value>, V::Error> {
		if self.fields.is_empty() {
			return Ok(None);
		}

		let field = &self.fields[0];
		let b = &mut self.bytes;

		// Don't return here; decrement bytes properly first and then return, so that
		// calling decode_item again works as expected.
		let res = crate::decode::decode(b, field.id(), self.types, visitor);

		self.fields = &self.fields[1..];
		self.bytes = *b;

		res.map(Some)
	}
}
