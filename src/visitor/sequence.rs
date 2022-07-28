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

/// This enables a visitor to decode items from a sequence type.
pub struct Sequence<'a, 'b> {
	bytes: &'a [u8],
	type_id: u32,
	types: &'b PortableRegistry,
	remaining: usize,
}

impl<'a, 'b> Sequence<'a, 'b> {
	pub(crate) fn new(
		bytes: &'a [u8],
		type_id: u32,
		len: usize,
		types: &'b PortableRegistry,
	) -> Sequence<'a, 'b> {
		Sequence { bytes, type_id, types, remaining: len }
	}
	pub(crate) fn bytes(&self) -> &'a [u8] {
		self.bytes
	}
	pub(crate) fn skip_rest(&mut self) -> Result<(), DecodeError> {
		while self.remaining > 0 {
			self.decode_item(IgnoreVisitor)?;
		}
		Ok(())
	}
	/// The number of un-decoded items remaining in this sequence.
	pub fn len(&self) -> usize {
		self.remaining
	}
	/// Are there any un-decoded items remaining in this sequence.
	pub fn is_empty(&self) -> bool {
		self.remaining == 0
	}
	/// Decode an item from the sequence by providing a visitor to handle it.
	pub fn decode_item<V: Visitor>(&mut self, visitor: V) -> Result<Option<V::Value>, V::Error> {
		if self.remaining == 0 {
			return Ok(None);
		}

		let b = &mut self.bytes;
		// Don't return here; decrement bytes and remaining properly first and then return, so that
		// calling decode_item again works as expected.
		let res = crate::decode::decode(b, self.type_id, self.types, visitor);
		self.bytes = *b;
		self.remaining -= 1;
		res.map(Some)
	}
}
