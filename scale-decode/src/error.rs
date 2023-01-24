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

use crate::context::Context;
use std::fmt::Display;

/// An error produced while attempting to encode some type.
#[derive(Debug, Clone, thiserror::Error)]
pub struct Error {
    context: Context,
    kind: ErrorKind,
}

impl Error {
    /// construct a new error given some context and an error kind.
    pub fn new(context: Context, kind: ErrorKind) -> Error {
        Error { context, kind }
    }
    /// Retrieve more information abotu what went wrong.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
    /// Retrieve details about where the error occurred.
    pub fn context(&self) -> &Context {
        &self.context
    }
    /// If the current error context is empty, replace it with
    /// the one provided.
    pub fn or_context(self, context: Context) -> Self {
        Error {
            kind: self.kind,
            context: if self.context.is_empty() { context } else { self.context },
        }
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
        Error::new(Context::new(), err.into())
    }
}

/// The underlying nature of the error.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum ErrorKind {
    /// Something went wrong decoding the bytes based on the type
    /// and type registry provided.
    #[error("Error decoding bytes given the type ID and registry provided: {0}")]
    VisitorDecodeError(#[from] crate::visitor::DecodeError),
    /// We cannot decode the number seen into the target type; it's out of range.
    #[error("Number {value} is out of range")]
    NumberOutOfRange {
        /// A string represenatation of the numeric value that was out of range.
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
    #[error("Cannot decode from type with ID {actual}; expected length {expected_len} but got length {actual_len}")]
    WrongLength {
        /// ID of the type we're trying to decode from.
        actual: u32,
        /// Length of the type we are trying to decode from
        actual_len: usize,
        /// Length fo the type we're trying to decode into
        expected_len: usize,
    },
}
