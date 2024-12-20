// Copyright (C) 2023 Parity Technologies (UK) Ltd. (admin@parity.io)
// This file is a part of the scale-decode crate.
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

pub use context::{Context, Location};

use crate::visitor::DecodeError;
use alloc::{borrow::Cow, boxed::Box, string::String, vec::Vec};
use core::fmt::Display;

/// An error produced while attempting to decode some type.
#[derive(Debug)]
pub struct Error {
    context: Context,
    kind: ErrorKind,
}

impl core::error::Error for Error {}

impl Error {
    /// Construct a new error given an error kind.
    pub fn new(kind: ErrorKind) -> Error {
        Error { context: Context::new(), kind }
    }
    /// Construct a new, custom error.
    pub fn custom(error: impl core::error::Error + Send + Sync + 'static) -> Error {
        Error::new(ErrorKind::Custom(Box::new(error)))
    }
    /// Construct a custom error from a static string.
    pub fn custom_str(error: &'static str) -> Error {
        #[derive(Debug, thiserror::Error)]
        #[error("{0}")]
        pub struct StrError(pub &'static str);

        Error::new(ErrorKind::Custom(Box::new(StrError(error))))
    }
    /// Construct a custom error from an owned string.
    pub fn custom_string(error: String) -> Error {
        #[derive(Debug, thiserror::Error)]
        #[error("{0}")]
        pub struct StringError(String);

        Error::new(ErrorKind::Custom(Box::new(StringError(error))))
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
    pub fn at(mut self, loc: Location) -> Self {
        self.context.push(loc);
        Error { context: self.context, kind: self.kind }
    }
    /// Note which sequence index the error occurred in.
    pub fn at_idx(mut self, idx: usize) -> Self {
        self.context.push(Location::idx(idx));
        Error { context: self.context, kind: self.kind }
    }
    /// Note which field the error occurred in.
    pub fn at_field(mut self, field: impl Into<Cow<'static, str>>) -> Self {
        self.context.push(Location::field(field));
        Error { context: self.context, kind: self.kind }
    }
    /// Note which variant the error occurred in.
    pub fn at_variant(mut self, variant: impl Into<Cow<'static, str>>) -> Self {
        self.context.push(Location::variant(variant));
        Error { context: self.context, kind: self.kind }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let path = self.context.path();
        let kind = &self.kind;
        write!(f, "Error at {path}: {kind}")
    }
}

impl From<DecodeError> for Error {
    fn from(err: DecodeError) -> Error {
        Error::new(err.into())
    }
}

impl From<codec::Error> for Error {
    fn from(err: codec::Error) -> Error {
        let err: DecodeError = err.into();
        Error::new(err.into())
    }
}

/// The underlying nature of the error.
#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    /// Something went wrong decoding the bytes based on the type
    /// and type registry provided.
    #[error("Error decoding bytes given the type ID and registry provided: {_0}")]
    VisitorDecodeError(#[from] DecodeError),
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
    Custom(Box<dyn core::error::Error + Send + Sync + 'static>),
}
