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

/*!
`parity-scale-codec` provides a `Decode` trait which allows bytes to be scale decoded into types based on the shape of those types.
This crate builds on this, and allows bytes to be decoded into types based on [`scale_info`] type information, rather than the shape
of the target type. At a high level, this crate just aims to do the reverse of the `scale-encode` crate.

This crate exposes four traits:

- A [`visitor::Visitor`] trait which when implemented on some type, can be used in conjunction with [`visitor::decode_with_visitor`]
  to decode SCALE encoded bytes based on some type information into some arbitrary type.
- An [`IntoVisitor`] trait which can be used to obtain the [`visitor::Visitor`] implementation for some type.
- A [`DecodeAsType`] trait which is implemented for types which implement [`IntoVisitor`], and provides a high level interface for
  decoding SCALE encoded bytes into some type with the help of a type ID and [`scale_info::PortableRegistry`].
- A [`DecodeAsFields`] trait which when implemented on some type, describes how SCALE encoded bytes can be decoded
  into it with the help of an iterator of [`Field`]s and a type registry describing the shape of the encoded bytes. This is
  generally only implemented for tuples and structs, since we need a set of fields to map to the provided slices.

Implementations for many built-in types are also provided for each trait, and the [`macro@DecodeAsType`] macro can be used to
generate the relevant impls on new struct and enum types such that they get a [`DecodeAsType`] impl.

The [`DecodeAsType`] and [`DecodeAsFields`] traits are basically the mirror of `scale-encode`'s `EncodeAsType` and `EncodeAsFields`
traits in terms of their interface.

# Motivation

By de-coupling the shape of a type from how bytes are decoded into it, we make it much more likely that the decoding will succeed,
and are no longer reliant on types having a precise layout in order to be decoded into correctly. Some examples of this follow.

```rust
use codec::Encode;
use scale_decode::DecodeAsType;
use scale_info::{PortableRegistry, TypeInfo};
use std::fmt::Debug;

// We normally expect to have type information to hand, but for our examples
// we construct type info from any type that implements `TypeInfo`.
fn get_type_info<T: TypeInfo + 'static>() -> (u32, PortableRegistry) {
    let m = scale_info::MetaType::new::<T>();
    let mut types = scale_info::Registry::new();
    let ty = types.register_type(&m);
    let portable_registry: PortableRegistry = types.into();
    (ty.id(), portable_registry)
}

// Encode the left value statically.
// Decode those bytes into the right type via `DecodeAsType`.
// Assert that the decoded bytes are identical to the right value.
fn assert_decodes_to<A, B>(a: A, b: B)
where
    A: Encode + TypeInfo + 'static,
    B: DecodeAsType + PartialEq + Debug,
{
    let (type_id, types) = get_type_info::<A>();
    let a_bytes = a.encode();
    let new_b = B::decode_as_type(&mut &*a_bytes, type_id, &types).unwrap();
    assert_eq!(b, new_b);
}

// Start simple; a u8 can DecodeAsType into a u64 and vice versa. Numbers will all
// try to convert into the desired output size, failing if this isn't possible:
assert_decodes_to(123u64, 123u8);
assert_decodes_to(123u8, 123u64);

// Compact decoding is also handled "under the hood" by DecodeAsType, so no "compact"
// annotations are needed on values.
assert_decodes_to(codec::Compact(123u64), 123u64);

// Enum variants are lined up by variant name, so no explicit "index" annotation are
// needed either; DecodeAsType will take care of it.
#[derive(Encode, TypeInfo)]
enum Foo {
    #[codec(index = 10)]
    Something(u64),
}
#[derive(DecodeAsType, PartialEq, Debug)]
enum FooTarget {
    Something(u128),
}
assert_decodes_to(Foo::Something(123), FooTarget::Something(123));

// DecodeAsType will skip annotated fields and not look for them in the encoded bytes.
// #[codec(skip)] and #[decode_as_type(skip)] both work.
#[derive(Encode, TypeInfo)]
struct Bar {
    a: bool,
}
#[derive(DecodeAsType, PartialEq, Debug)]
struct BarTarget {
    a: bool,
    #[decode_as_type(skip)]
    b: String,
}
assert_decodes_to(
    Bar { a: true },
    BarTarget { a: true, b: String::new() },
);

// DecodeAsType impls will generally skip through any newtype wrappers.
#[derive(DecodeAsType, Encode, TypeInfo, PartialEq, Debug)]
struct Wrapper {
    value: u64
}
assert_decodes_to(
    (Wrapper { value: 123 },),
    123u64
);
assert_decodes_to(
    123u64,
    (123,)
);

// Things like arrays and sequences are generally interchangeable despite the
// encoding format being slightly different:
assert_decodes_to([1u8,2,3,4,5], vec![1u64,2,3,4,5]);
assert_decodes_to(vec![1u64,2,3,4,5], [1u8,2,3,4,5]);
```

If this high level interface isn't suitable, you can implement your own [`visitor::Visitor`]'s. These can support zero-copy decoding
(unlike the higher level [`DecodeAsType`] interface), and generally the Visitor construction and execution is zero alloc, allowing
for efficient type based decoding.
*/
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

/// This trait is implemented for any type `T` where `T` implements [`IntoVisitor`] and the errors returned
/// from this [`Visitor`] can be converted into [`Error`]. It's essentially a convenience wrapper around
/// [`visitor::decode_with_visitor`] that mirrors `scale-encode`'s `EncodeAsType`.
pub trait DecodeAsType: Sized {
    /// Given some input bytes, a `type_id`, and type registry, attempt to decode said bytes into
    /// `Self`. Implementations should modify the `&mut` reference to the bytes such that any bytes
    /// not used in the course of decoding are still pointed to after decoding is complete.
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
    fn decode_as_fields<'info, I: FieldIter<'info>>(
        input: &mut &[u8],
        fields: I,
        types: &'info PortableRegistry,
    ) -> Result<Self, Error>;
}

/// A representation of a single field to be encoded via [`DecodeAsFields::decode_as_fields`].
#[derive(Debug, Clone, Copy)]
pub struct Field<'a> {
    name: Option<&'a str>,
    id: u32,
}

impl<'a> Field<'a> {
    /// Construct a new field with an ID and optional name.
    pub fn new(id: u32, name: Option<&'a str>) -> Self {
        Field { id, name }
    }
    /// Create a new unnamed field.
    pub fn unnamed(id: u32) -> Self {
        Field { name: None, id }
    }
    /// Create a new named field.
    pub fn named(id: u32, name: &'a str) -> Self {
        Field { name: Some(name), id }
    }
    /// The field name, if any.
    pub fn name(&self) -> Option<&'a str> {
        self.name
    }
    /// The field ID.
    pub fn id(&self) -> u32 {
        self.id
    }
}

/// An iterator over a set of fields.
pub trait FieldIter<'a>: Iterator<Item = Field<'a>> + Clone {}
impl<'a, T> FieldIter<'a> for T where T: Iterator<Item = Field<'a>> + Clone {}

/// This trait can be implemented on any type that has an associated [`Visitor`] responsible for decoding
/// SCALE encoded bytes to it. If you implement this on some type and the [`Visitor`] that you return has
/// an error type that converts into [`Error`], then you'll also get a [`DecodeAsType`] implementation for free.
pub trait IntoVisitor {
    /// The visitor type used to decode SCALE encoded bytes to `Self`.
    type Visitor: for<'scale, 'info> visitor::Visitor<Value<'scale, 'info> = Self>;
    /// A means of obtaining this visitor.
    fn into_visitor() -> Self::Visitor;
}

// In a few places, we need an empty path with a lifetime that outlives 'info,
// so here's one that lives forever that we can use.
#[doc(hidden)]
pub static EMPTY_SCALE_INFO_PATH: &scale_info::Path<scale_info::form::PortableForm> =
    &scale_info::Path { segments: Vec::new() };
