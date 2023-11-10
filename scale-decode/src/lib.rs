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

#![cfg_attr(not(feature = "std"), no_std)]

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

extern crate alloc;

mod impls;

pub mod error;
pub mod visitor;

pub use crate::error::Error;
pub use visitor::Visitor;

// Used in trait definitions.
use scale_info::form::PortableForm;
pub use scale_info::PortableRegistry;

// This is exported for generated derive code to use, to be compatible with std or no-std as needed.
#[doc(hidden)]
pub use alloc::{collections::BTreeMap, string::ToString, vec};

/// Re-exports of external crates.
pub mod ext {
    #[cfg(feature = "primitive-types")]
    pub use primitive_types;
}

use alloc::vec::Vec;

/// This trait is implemented for any type `T` where `T` implements [`IntoVisitor`] and the errors returned
/// from this [`Visitor`] can be converted into [`Error`]. It's essentially a convenience wrapper around
/// [`visitor::decode_with_visitor`] that mirrors `scale-encode`'s `EncodeAsType`.
pub trait DecodeAsType: Sized + IntoVisitor {
    /// Given some input bytes, a `type_id`, and type registry, attempt to decode said bytes into
    /// `Self`. Implementations should modify the `&mut` reference to the bytes such that any bytes
    /// not used in the course of decoding are still pointed to after decoding is complete.
    fn decode_as_type(
        input: &mut &[u8],
        type_id: u32,
        types: &PortableRegistry,
    ) -> Result<Self, Error> {
        Self::decode_as_type_maybe_compact(input, type_id, types, false)
    }

    /// Given some input bytes, a `type_id`, and type registry, attempt to decode said bytes into
    /// `Self`. Implementations should modify the `&mut` reference to the bytes such that any bytes
    /// not used in the course of decoding are still pointed to after decoding is complete.
    ///
    /// If is_compact=true, it is assumed the value is compact encoded (only works for some types).
    #[doc(hidden)]
    fn decode_as_type_maybe_compact(
        input: &mut &[u8],
        type_id: u32,
        types: &PortableRegistry,
        is_compact: bool,
    ) -> Result<Self, Error>;
}

impl<T: Sized + IntoVisitor> DecodeAsType for T {
    fn decode_as_type_maybe_compact(
        input: &mut &[u8],
        type_id: u32,
        types: &scale_info::PortableRegistry,
        is_compact: bool,
    ) -> Result<Self, Error> {
        let res = visitor::decode_with_visitor_maybe_compact(
            input,
            type_id,
            types,
            T::into_visitor(),
            is_compact,
        )?;
        Ok(res)
    }
}

/// This is similar to [`DecodeAsType`], except that it's instead implemented for types that can be given a list of
/// fields denoting the type being decoded from and attempt to do this decoding. This is generally implemented just
/// for tuple and struct types, and is automatically implemented via the [`macro@DecodeAsType`] macro.
pub trait DecodeAsFields: Sized {
    /// Given some bytes and some fields denoting their structure, attempt to decode.
    fn decode_as_fields<'info>(
        input: &mut &[u8],
        fields: &mut dyn FieldIter<'info>,
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

impl<'a> From<&'a scale_info::Field<PortableForm>> for Field<'a> {
    fn from(value: &'a scale_info::Field<PortableForm>) -> Self {
        Field { name: value.name.as_deref(), id: value.ty.id }
    }
}

/// An iterator over a set of fields.
pub trait FieldIter<'a>: Iterator<Item = Field<'a>> {}
impl<'a, T> FieldIter<'a> for T where T: Iterator<Item = Field<'a>> {}

/// This trait can be implemented on any type that has an associated [`Visitor`] responsible for decoding
/// SCALE encoded bytes to it whose error type is [`Error`]. Anything that implements this trait gets a
/// [`DecodeAsType`] implementation for free.
// Dev note: This used to allow for any Error type that could be converted into `scale_decode::Error`.
// The problem with this is that the `DecodeAsType` trait became tricky to use in some contexts, because it
// didn't automatically imply so much. Realistically, being stricter here shouldn't matter too much; derive
// impls all use `scale_decode::Error` anyway, and manual impls can just manually convert into the error
// rather than rely on auto conversion, if they care about also being able to impl `DecodeAsType`.
pub trait IntoVisitor {
    /// The visitor type used to decode SCALE encoded bytes to `Self`.
    type Visitor: for<'scale, 'info> visitor::Visitor<Value<'scale, 'info> = Self, Error = Error>;
    /// A means of obtaining this visitor.
    fn into_visitor() -> Self::Visitor;
}

// In a few places, we need an empty path with a lifetime that outlives 'info,
// so here's one that lives forever that we can use.
#[doc(hidden)]
pub static EMPTY_SCALE_INFO_PATH: &scale_info::Path<scale_info::form::PortableForm> =
    &scale_info::Path { segments: Vec::new() };

/// The `DecodeAsType` derive macro can be used to implement `DecodeAsType` on structs and enums whose
/// fields all implement `DecodeAsType`. Under the hood, the macro generates `scale_decode::visitor::Visitor`
/// and `scale_decode::IntoVisitor` implementations for each type (as well as an associated `Visitor` struct),
/// which in turn means that the type will automatically implement `scale_decode::DecodeAsType`.
///
/// # Examples
///
/// This can be applied to structs and enums:
///
/// ```rust
/// use scale_decode::DecodeAsType;
///
/// #[derive(DecodeAsType)]
/// struct Foo(String);
///
/// #[derive(DecodeAsType)]
/// struct Bar {
///     a: u64,
///     b: bool
/// }
///
/// #[derive(DecodeAsType)]
/// enum Wibble<T> {
///     A(usize, bool, T),
///     B { value: String },
///     C
/// }
/// ```
///
/// If you aren't directly depending on `scale_decode`, you must tell the macro what the path
/// to it is so that it knows how to generate the relevant impls:
///
/// ```rust
/// # use scale_decode as alt_path;
/// use alt_path::DecodeAsType;
///
/// #[derive(DecodeAsType)]
/// #[decode_as_type(crate_path = "alt_path")]
/// struct Foo<T> {
///    a: u64,
///    b: T
/// }
/// ```
///
/// If you use generics, the macro will assume that each of them also implements `EncodeAsType`.
/// This can be overridden when it's not the case (the compiler will ensure that you can't go wrong here):
///
/// ```rust
/// use scale_decode::DecodeAsType;
///
/// #[derive(DecodeAsType)]
/// #[decode_as_type(trait_bounds = "")]
/// struct Foo<T> {
///    a: u64,
///    b: bool,
///    #[decode_as_type(skip)]
///    c: std::marker::PhantomData<T>
/// }
/// ```
///
/// You'll note that we can also opt to skip fields that we don't want to decode into; such fields will receive
/// their default value and no attempt to decode SCALE bytes into them will occur.
///
/// # Attributes
///
/// - `#[decode_as_type(crate_path = "::path::to::scale_decode")]`:
///   By default, the macro expects `scale_decode` to be a top level dependency,
///   available as `::scale_decode`. If this is not the case, you can provide the
///   crate path here.
/// - `#[decode_as_type(trait_bounds = "T: Foo, U::Input: DecodeAsType")]`:
///   By default, for each generate type parameter, the macro will add trait bounds such
///   that these type parameters must implement `DecodeAsType` too. You can override this
///   behaviour and provide your own trait bounds instead using this option.
/// - `#[decode_as_type(skip)]` (or `#[codec(skip)]`):
///   Any fields annotated with this will be skipped when attempting to decode into the
///   type, and instead will be populated with their default value (and therefore must
///   implement [`std::default::Default`]).
#[cfg(feature = "derive")]
pub use scale_decode_derive::DecodeAsType;
