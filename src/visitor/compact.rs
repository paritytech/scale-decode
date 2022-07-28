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

use super::TypeId;

/// This represents a compact encoded type.
pub struct Compact<'c, T> {
	val: T,
	inner_type_ids: &'c [TypeId],
}

impl<'c, T: Copy> Compact<'c, T> {
	pub(crate) fn new(val: T, inner_type_ids: &'c [TypeId]) -> Compact<'c, T> {
		Compact { val, inner_type_ids }
	}
	/// Return the value that was compact-encoded
	pub fn value(&self) -> T {
		self.val
	}
	/// The type ID returned in the compact visitor functions is always the
	/// one corresponding to the outermost `Compact` value. This returns all
	/// of the other Type IDs encountered down to, and including the one for the
	/// value itself (which should be an ID pointing to some primitive type).
	///
	/// It will always have a length of at least 1, for this reason.
	pub fn type_ids(&self) -> &'c [TypeId] {
		self.inner_type_ids
	}
}
