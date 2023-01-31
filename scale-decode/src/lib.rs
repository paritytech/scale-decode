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

//! This crate is concerned with decoding arbitrary values from some
//! SCALE encoded bytes, given a type ID and type registry that defines
//! the expected shape that the bytes should be decoded into.
//!
//! In order to allow the user to decode bytes into any shape they like,
//! you must implement a [`visitor::Visitor`] trait, which is handed
//! values back and has the opportunity to transform them into some
//! output representation of your choice (or fail with an error of your
//! choice). This Visitor is passed to the [`decode()`] method, whose job it
//! is to look at the type information provided and pass values of those
//! types to the Visitor, or fail if the bytes do not match the expected
//! shape.

#![deny(missing_docs)]

mod impls;
mod utils;

pub mod error;
pub mod visitor;

use scale_info::PortableRegistry;

pub use crate::error::Error;
pub use visitor::Visitor;

/// This trait is implemented for any type `T` where `T` implements [`IntoVisitor`] and the errors returned
/// from this [`Visitor`] can be converted into [`Error`]. It's essentially a convenience wrapper around
/// [`visitor::decode_with_visitor`] that mirrors `scale-encode`'s `EncodeAsType`.
pub trait DecodeAsType: Sized {
    /// Given some input bytes, a `type_id`, type registry and context, attempt to decode said bytes into
    /// `Self`. Implementations should modify the `&mut` reference to the bytes such that any bytes not used
    /// in the course of decoding are still pointed to after decoding is complete.
    fn decode_as_type(
        input: &mut &[u8],
        type_id: u32,
        types: &PortableRegistry,
    ) -> Result<Self, Error>;
}

impl<T> DecodeAsType for T
where
    T: IntoVisitor,
    Error: From<<T::Visitor as Visitor>::Error>,
{
    fn decode_as_type(
        input: &mut &[u8],
        type_id: u32,
        types: &scale_info::PortableRegistry,
    ) -> Result<Self, Error> {
        let res = visitor::decode_with_visitor(input, type_id, types, T::into_visitor())?;
        Ok(res)
    }
}

/// This trait can be implemented on any type that has an associated [`Visitor`] responsible for decoding
/// SCALE encoded bytes to it.
pub trait IntoVisitor {
    /// The visitor type used to decode SCALE encoded bytes to `Self`.
    type Visitor: for<'b> visitor::Visitor<Value<'b> = Self>;
    /// A means of obtaining this visitor.
    fn into_visitor() -> Self::Visitor;
}
