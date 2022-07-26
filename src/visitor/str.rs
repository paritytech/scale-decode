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

use codec::{
    Compact,
    Decode,
};
use super::{
    DecodeError,
};

/// This represents a string, but defers proper decoding of it until it's asked for,
/// and avoids allocating.
pub struct Str<'a> {
    len: usize,
    bytes: &'a [u8]
}

impl <'a> Str<'a> {
    pub (crate) fn new_from(bytes: &mut &'a [u8]) -> Result<Self, DecodeError> {
        // Strings are just encoded the same as bytes; a length prefix and then
        // the raw bytes. Pluck these out but don't do any further work.
        let len = <Compact<u32>>::decode(bytes)?.0 as usize;
        let str_bytes = &bytes[..len];
        *bytes = &bytes[len..];
        Ok(Str {
            len: len,
            bytes: str_bytes
        })
    }
    /// The length of the string.
    pub fn len(&self) -> usize {
        self.len
    }
    /// return a string, failing if the bytes could not be properly utf8-decoded.
    pub fn as_str(&self) -> Result<&'a str, DecodeError> {
        std::str::from_utf8(self.bytes).map_err(DecodeError::InvalidStr)
    }
    /// Return the raw bytes representing this string.
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }
}