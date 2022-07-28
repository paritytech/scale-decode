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
pub struct Compact<'b, 'c, T> {
	val: T,
	locations: &'c [CompactLocation<'b>],
}

impl<'b, 'c, T: Copy> Compact<'b, 'c, T> {
	pub(crate) fn new(val: T, locations: &'c [CompactLocation<'b>]) -> Compact<'b, 'c, T> {
		Compact { val, locations }
	}
	/// Return the value that was compact-encoded
	pub fn value(&self) -> T {
		self.val
	}
	/// Compact values can be nested inside named or unnamed fields in structs.
	/// This provides back a slice of
	pub fn locations(&self) -> &'c [CompactLocation<'b>] {
		self.locations
	}
}

/// A pointer to what the compact value is contained within.
#[derive(Clone, Copy, Debug)]
pub enum CompactLocation<'b> {
	/// We're in an unnamed composite (struct) with the type ID given.
	UnnamedComposite(TypeId),
	/// We're in a named composite (struct) with the type ID given, and the compact
	/// value lives inside the field with the given name.
	NamedComposite(TypeId, &'b str),
	/// We're at a primitive type with the type ID given; the compact value itself.
	Primitive(TypeId),
}

// Default values for locations are never handed back, but they are
// stored on the StackArray in the "unused" positions. We could avoid needing
// this with some unsafe code.
impl<'a> Default for CompactLocation<'a> {
	fn default() -> Self {
		CompactLocation::Primitive(TypeId::default())
	}
}
