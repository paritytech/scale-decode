// Copyright (C) 2023 Parity Technologies (UK) Ltd. (admin@parity.io)
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

//! An error that is emitted whenever some decoding fails.
mod context;
mod linkedlist;

use std::borrow::Cow;
use std::fmt::Display;

pub use context::{Context, Location};

/// An error produced while attempting to decode some type.
#[derive(Debug, thiserror::Error)]
pub struct Error {
    context: Context,
    kind: ErrorKind,
}

impl Error {
    /// Construct a new error given an error kind.
    pub fn new(kind: ErrorKind) -> Error {
        Error { context: Context::new(), kind }
    }
    /// Construct a new, custom error.
    pub fn custom(error: impl Into<CustomError>) -> Error {
        Error::new(ErrorKind::Custom(error.into()))
    }
    /// Retrieve more information about what went wrong.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
    /// Retrieve details about where the error occurred.
    pub fn context(&self) -> &Context {
        &self.context
    }
    /// Give some context to the error.
    pub fn at(self, loc: Location) -> Self {
        Error { context: self.context.at(loc), kind: self.kind }
    }
    /// Note which sequence index the error occurred in.
    pub fn at_idx(self, idx: usize) -> Self {
        Error { context: self.context.at(Location::idx(idx)), kind: self.kind }
    }
    /// Note which field the error occurred in.
    pub fn at_field(self, field: impl Into<Cow<'static, str>>) -> Self {
        Error { context: self.context.at(Location::field(field)), kind: self.kind }
    }
    /// Note which variant the error occurred in.
    pub fn at_variant(self, variant: impl Into<Cow<'static, str>>) -> Self {
        Error { context: self.context.at(Location::variant(variant)), kind: self.kind }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = self.context.path();
        let kind = &self.kind;
        write!(f, "Error at {path}: {kind}")
    }
}

impl From<crate::visitor::DecodeError> for Error {
    fn from(err: crate::visitor::DecodeError) -> Error {
        Error::new(err.into())
    }
}

/// The underlying nature of the error.
#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    /// Something went wrong decoding the bytes based on the type
    /// and type registry provided.
    #[error("Error decoding bytes given the type ID and registry provided: {0}")]
    VisitorDecodeError(#[from] crate::visitor::DecodeError),
    /// We cannot decode the number seen into the target type; it's out of range.
    #[error("Number {value} is out of range")]
    NumberOutOfRange {
        /// A string representation of the numeric value that was out of range.
        value: String,
    },
    /// We cannot find the variant we're trying to decode from in the target type.
    #[error("Cannot find variant {got}; expects one of {expected:?}")]
    CannotFindVariant {
        /// The variant that we are given back from the encoded bytes.
        got: String,
        /// The possible variants that we can decode into.
        expected: Vec<&'static str>,
    },
    /// The types line up, but the expected length of the target type is different from the length of the input value.
    #[error("Cannot decode from type; expected length {expected_len} but got length {actual_len}")]
    WrongLength {
        /// Length of the type we are trying to decode from
        actual_len: usize,
        /// Length fo the type we're trying to decode into
        expected_len: usize,
    },
    /// Cannot find a field that we need to decode to our target type
    #[error("Field {name} does not exist in our encoded data")]
    CannotFindField {
        /// Name of the field which was not provided.
        name: String,
    },
    /// A custom error.
    #[error("Custom error: {0}")]
    Custom(CustomError),
}

type CustomError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[cfg(test)]
mod test {
    use super::*;

    #[derive(thiserror::Error, Debug)]
    enum MyError {
        #[error("Foo!")]
        Foo,
    }

    #[test]
    fn custom_error() {
        // Just a compile-time check that we can ergonomically provide an arbitrary custom error:
        Error::custom(MyError::Foo);
    }
}
