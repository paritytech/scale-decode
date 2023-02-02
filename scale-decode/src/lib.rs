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
//! The standard approach is to use the [`macro@DecodeAsType`] macro to auto-implement
//! [`IntoVisitor`] and ultimately [`trait@DecodeAsType`] on your custom struct or enum.
//! If you'd like to do mroe custom decoding, you can instead implement [`Visitor`] directly
//! in order to have full control over how to decode some bytes into your custom type..

#![deny(missing_docs)]

mod impls;
mod utils;

pub mod error;
pub mod visitor;

use scale_info::PortableRegistry;

pub use crate::error::Error;
pub use visitor::{Visitor, VisitorExt};

#[cfg(feature = "derive")]
pub use scale_decode_derive::DecodeAsType;

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
/// SCALE encoded bytes to it. If you implement this on some type and the [`Visitor`] that you return has
/// an error type that converts into [`Error`], then you'll also get a [`DecodeAsType`] implementation for free.
pub trait IntoVisitor {
    /// The visitor type used to decode SCALE encoded bytes to `Self`.
    type Visitor: for<'b> visitor::Visitor<Value<'b> = Self>;
    /// A means of obtaining this visitor.
    fn into_visitor() -> Self::Visitor;
}
