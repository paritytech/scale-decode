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

use super::{
    sequence::Sequence,
    DecodeError,
    Visitor,
};

/// This enables a visitor to decode items from an array type.
pub struct Array<'a> {
    seq: Sequence<'a>
}

impl <'a> Array<'a> {
    pub (crate) fn new(seq: Sequence<'a>) -> Self {
        Array { seq }
    }
    pub (crate) fn bytes(&self) -> &'a [u8] {
        self.seq.bytes()
    }
    pub (crate) fn skip_rest(&mut self) -> Result<(), DecodeError> {
        self.seq.skip_rest()
    }
    pub fn len(&self) -> usize {
        self.seq.len()
    }
    pub fn remaining(&self) -> usize {
        self.seq.remaining()
    }
    pub fn decode_item<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, V::Error> {
        self.seq.decode_item(visitor)
    }
}