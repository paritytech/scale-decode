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

use super::{sequence::Sequence, DecodeError, Visitor};

/// This represents an array type.
pub struct Array<'a> {
	seq: Sequence<'a>,
}

impl<'a> Array<'a> {
	pub(crate) fn new(seq: Sequence<'a>) -> Self {
		Array { seq }
	}
	pub(crate) fn bytes(&self) -> &'a [u8] {
		self.seq.bytes()
	}
	pub(crate) fn skip_rest(&mut self) -> Result<(), DecodeError> {
		self.seq.skip_rest()
	}
	/// The number of un-decoded items left in the array.
	pub fn len(&self) -> usize {
		self.seq.len()
	}
	/// Are there any un-decoded items remaining in the array.
	pub fn is_empty(&self) -> bool {
		self.seq.is_empty()
	}
	/// Decode the next item from the array by providing a visitor to handle it.
	pub fn decode_item<V: Visitor>(&mut self, visitor: V) -> Result<Option<V::Value>, V::Error> {
		self.seq.decode_item(visitor)
	}
}
