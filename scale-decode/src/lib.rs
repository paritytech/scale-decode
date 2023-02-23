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

//! This crate makes it easy to decode SCALE encoded bytes into a custom data structure with the help of [`scale_info`] types.
//! By using this type information to guide decoding (instead of just trying to decode bytes based on the shape of the target type),
//! it's possible to be much more flexible in how data is decoded and mapped to some target type.
//!
//! The main trait used to decode types is a [`Visitor`] trait (example below). By implementing this trait, you can describe how to
//! take SCALE decoded values and map them to some custom type of your choosing (whether it is a dynamically shaped type or some
//! static type you'd like to decode into). Implementations of this [`Visitor`] trait exist for many existing Rust types in the standard
//! library.
//!
//! There also exists an [`IntoVisitor`] trait, which is implemented on many existing rust types and maps a given type to some visitor
//! implementation capable of decoding into it.
//!
//! Finally, a wrapper trait, [`DecodeAsType`], is auto-implemented for all types that have an [`IntoVisitor`] implementation,
//! and whose visitor errors can be turned into a standard [`crate::Error`].
//!
//! For custom structs and enums, one can use the [`macro@DecodeAsType`] derive macro to have a [`DecodeAsType`] implementation automatically
//! generated.

#![deny(missing_docs)]

mod impls;
mod utils;

pub mod error;
pub mod visitor;

pub use crate::error::Error;
pub use visitor::Visitor;

#[cfg(feature = "derive")]
pub use scale_decode_derive::DecodeAsType;

// Used in trait definitions.
pub use scale_info::PortableRegistry;
/// A description of a single field in a tuple or struct type.
pub type PortableField = scale_info::Field<scale_info::form::PortableForm>;
/// A type ID used to represent tuple fields.
pub type PortableFieldId = scale_info::interner::UntrackedSymbol<std::any::TypeId>;

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

/// This is similar to [`DecodeAsType`], except that it's instead implemented for types that can be given a list of
/// fields denoting the type being decoded from and attempt to do this decoding. This is generally implemented just
/// for tuple and struct types, and is automatically implemented via the [`macro@DecodeAsType`] macro.
pub trait DecodeAsFields: Sized {
    /// Given some bytes and some fields denoting their structure, attempt to decode.
    fn decode_as_fields(
        input: &mut &[u8],
        fields: &[PortableField],
        types: &PortableRegistry,
    ) -> Result<Self, Error>;

    /// Given some bytes and some field IDs denoting their structure, attempt to decode.
    fn decode_as_field_ids(
        input: &mut &[u8],
        field_ids: &[PortableFieldId],
        types: &PortableRegistry,
    ) -> Result<Self, Error> {
        // [TODO jsdw]: It would be good to use a more efficient data structure
        // here to avoid allocating with smaller numbers of fields.
        let fields: Vec<PortableField> =
            field_ids.iter().map(|f| PortableField::new(None, *f, None, Vec::new())).collect();
        Self::decode_as_fields(input, &fields, types)
    }
}

/// This trait can be implemented on any type that has an associated [`Visitor`] responsible for decoding
/// SCALE encoded bytes to it. If you implement this on some type and the [`Visitor`] that you return has
/// an error type that converts into [`Error`], then you'll also get a [`DecodeAsType`] implementation for free.
pub trait IntoVisitor {
    /// The visitor type used to decode SCALE encoded bytes to `Self`.
    type Visitor: for<'scale, 'info> visitor::Visitor<Value<'scale, 'info> = Self>;
    /// A means of obtaining this visitor.
    fn into_visitor() -> Self::Visitor;
}
