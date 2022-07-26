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
    PortableRegistry,
};
use super::{
    DecodeError,
    Visitor,
    IgnoreVisitor,
};

// This enables a visitor to decode information out of composite or variant fields.
pub struct Tuple<'a> {
    bytes: &'a [u8],
    fields: &'a [scale_info::interner::UntrackedSymbol<std::any::TypeId>],
    types: &'a PortableRegistry,
    len: usize,
}

impl <'a> Tuple<'a> {
    pub (crate) fn new(
        bytes: &'a [u8],
        fields: &'a [scale_info::interner::UntrackedSymbol<std::any::TypeId>],
        types: &'a PortableRegistry,
    ) -> Tuple<'a> {
        Tuple { len: fields.len(), bytes, fields, types }
    }
    pub (crate) fn bytes(&self) -> &'a [u8] {
        self.bytes
    }
    pub (crate) fn skip_rest(&mut self) -> Result<(), DecodeError> {
        while !self.fields.is_empty() {
            self.decode_item(IgnoreVisitor)?;
        }
        Ok(())
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn remaining(&self) -> usize {
        self.fields.len()
    }
    pub fn decode_item<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, V::Error> {
        if self.fields.is_empty() {
            return Err(DecodeError::NothingLeftToDecode.into())
        }

        let field = &self.fields[0];
        self.fields = &self.fields[1..];

        let b = &mut self.bytes;
        // Don't return here; decrement bytes properly first and then return, so that
        // calling decode_item again works as expected.
        let res = crate::decode::decode(b, field.id(), self.types, visitor);
        self.bytes = *b;
        res
    }
}