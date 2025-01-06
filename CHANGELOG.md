# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## 0.16.0 - 2024-11-15

This release updates scale-bits to 0.7.0 which is exposed in the public API of scale-decode.

## 0.15.0 - 2024-11-08

This release makes scale-decode entirely `no_std` which is now using `core::error::Error` instead of `std::error::Error` as it was using before behind
the `std feature`. Because of that the `std feature` is now removed and the MSRV is bumped to 1.81.0.

### Changed
- chore(deps): use core::error::Error and make no_std ([#67](https://github.com/paritytech/scale-decode/pull/67))

## 0.14.0 - 2024-10-21

### Changed

- chore(deps): bump syn from 1.0 to 2.0 ([#60](https://github.com/paritytech/scale-decode/pull/60))
- chore(deps): bump primitives types from 0.12.0 to 0.13.1 ([#61](https://github.com/paritytech/scale-decode/pull/61))
- chore(deps): chore(deps): bump derive_more from 0.99 to 1.0.0 ([#62](https://github.com/paritytech/scale-decode/pull/62))

## 0.13.2 - 2024-10-21 [YANKED]

Yanked because `primitives-types` is re-exported from this crate and thus is a breaking change.

## 0.13.1 - 2024-06-07

Useful visitor errors can be hidden in some cases when a `skip_decoding` error is also emitted. This patch release fixes that.

## Fixed

- Don't hide visitor error when a skip_decode error is also emitted ([#58](https://github.com/paritytech/scale-decode/pull/58))

## 0.13.0 - 2024-05-15

This minor release avoids a couple of panics in the case of invalid bytes being interpreted as BitVecs or strings.

A small API change was made to accommodate this, such that the `Str::bytes_after` now returns a result rather than just the bytes.

### Fixed

- Avoid panic on invalid bitvec input bytes ([#53](https://github.com/paritytech/scale-decode/pull/53)
- Avoid panic on invalid str input bytes ([#54](https://github.com/paritytech/scale-decode/pull/54))

## 0.12.0 - 2024-04-29

Update the `scale-type-resolver` dependency to 0.2.0 (and bump `scale-bits` for the same reason).

The main changes here are:
- Type IDs are now passed by value rather than reference.
- The `Composite` type handed back in the visitor's `visit_composite()` method now exposes the name and path of the composite type being decoded, if one was provided.

## 0.11.1 - 2024-02-16

- `scale-info` was still being pulled in via `scale-type-resolver`; this has now been fixed

## 0.11.0 - 2024-02-09

Up until now, this crate depended heavily on `scale_info` to provide the type information that we used to drive our decoding of types. This release removes the explicit dependency on `scale-info`, and instead depends on `scale-type-resolver`, which offers a generic `TypeResolver` trait whose implementations are able to provide the information needed to decode types (see [this PR](https://github.com/paritytech/scale-decode/pull/45) for more details). So now, the traits and types in `scale-decode` have been made generic over which `TypeResolver` is used to help decode things. `scale-info::PortableRegistry` is one such implementation of `TypeResolver`, and so can continue to be used in a similar way to before.

The main breaking changes are:

### Visitor trait

The `scale_decode::Visitor` trait now has an additional associated type, `TypeResolver`.

The simplest change to recover the previous behaviour is to just continue to use `scale_info::PortableRegistry` for this task, as was the case implicitly before:

```rust
struct MyVisitor;

impl Visitor for MyVisitor {
  // ...
  type TypeResolver = scale_info::PortableRegistry;

  // ...
}
```

A better change is to make your `Visitor` impls generic over what is used to resolve types unless you need a specific resolver:

```rust
struct MyVisitor<R>(PhantomData<R>);

impl <R> MyVisitor<R> {
  pub fn new() -> Self {
    Self(PhantomData)
  }
}

impl <R: TypeResolver> Visitor for MyVisitor<R> {
  // ...
  type TypeResolver = R;

  // ...
}
```

### IntoVisitor trait

`scale_decode::IntoVisitor` is implemented on all types that have an associated `Visitor` that we can use to decode it. It used to look like this:

```rust
pub trait IntoVisitor {
    type Visitor: for<'scale, 'resolver> visitor::Visitor<
        Value<'scale, 'resolver> = Self,
        Error = Error,
    >;
    fn into_visitor() -> Self::Visitor;
}
```

Now it looks like this:

```rust
pub trait IntoVisitor {
    type AnyVisitor<R: TypeResolver>: for<'scale, 'resolver> visitor::Visitor<
        Value<'scale, 'resolver> = Self,
        Error = Error,
        TypeResolver = R,
    >;
    fn into_visitor<R: TypeResolver>() -> Self::AnyVisitor<R>;
}
```

What this means is that if you want to implement `IntoVisitor` for some type, then your `Visitor` must be able to accept any valid `TypeResolver`. This allows `DecodeAsType` to also be generic over which `TypeResolver` is provided.

### DecodeAsType

This trait previously was specific to `scale_info::PortableRegistry` and looked something like this:

```rust
pub trait DecodeAsType: Sized + IntoVisitor {
    fn decode_as_type<R: TypeResolver>(
        input: &mut &[u8],
        type_id: u32,
        types: &scale_info::PortableRegistry,
    ) -> Result<Self, Error> {
        // ...
    }
```

Now, it is generic over which type resolver to use (and as a side effect, requires a reference to the `type_id` now, because it may not be `Copy` any more):

```rust
pub trait DecodeAsType: Sized + IntoVisitor {
    fn decode_as_type<R: TypeResolver>(
        input: &mut &[u8],
        type_id: &R::TypeId,
        types: &R,
    ) -> Result<Self, Error> {
        // ...
    }
```

This is automatically implemented for all types which implement `IntoVisitor`, as before.

### Composite and Variant paths

Mostly, the changes just exist to make things generic over the `TypeResolver`. One consequence of this is that Composite and Variant types no longer know their _path_, because currently `TypeResolver` doesn't provide that information (since it's not necessary to the actual encoding/decoding of types). If there is demand for this, we can consider allowing `TypeResolver` impls to optionally provide these extra details.

## 0.10.0 - 2023-11-10

This release changes `IntoVisitor` to require that the corresponding `Visitor` implements `scale_decode::Error`, rather than allowing the error to be anything that can be converted into a `scale_decode::Error`. This has the following advantages:

- It makes `DecodeAsType` a proper super-trait of `IntoVisitor`, meaning it can be used anywhere `IntoVisitor` is. Previously, using `DecodeAsType` in some places also required that you add a bound like `B: IntoVisitor, <B::Visitor as Visitor>::Error: Into<scale_decode::Error`, ie the `Error` type needed to explicitly be declared as convertible in the way that `DecodeAsType` requires.
- It simplifies the code.
- It makes it a bit easier to understand how to correctly make a type implement `DecodeAsType`.

The main drawback is that if your `Visitor` implementation doesn't have `Error = scale_decode::Error`, then it can no longer be used with `IntoVisitor`. To work around this, a new adapter type, `scale_decode::visitor::VisitorWithCrateError(your_visitor)` has been added; any visitor wrapped in this type whose error implements `Into<scale_decode::Error>` will now implement `Visitor` with `Error = scale_decode::Error`.

## 0.9.0 - 2023-08-02

- Change how compact encoding is handled: `visit_compact_*` functions are removed from the `Visitor` trait, and
  compact encoding is now handled recursively and should now work in more cases (such as nested structs with compact
  encoded values inside) ([#32](https://github.com/paritytech/scale-decode/pull/32)).
- Improve custom error handling: custom errors now require `Debug + Display` on `no_std` or `Error` on `std`.
  `Error::custom()` now accepts anything implementing these traits rather than depending on `Into<Error>`
  ([#31](https://github.com/paritytech/scale-decode/pull/31)).

## 0.8.0

- Add support for `no_std` (+alloc) builds ([#26](https://github.com/paritytech/scale-decode/pull/26)). Thankyou @haerdib!

## 0.7.0

- Change `DecodeAsFields` again; remove the generic iterator parameter and use `&mut dyn FieldIter` instead. This
  Simplifies the call signatures in a bunch of places and is consistent with how `scale-encode` works.
- Use `smallvec` instead of our own stack allocated vec.

## 0.6.0

- Change `DecodeAsFields` to accept an iterator of fields to decode into. This makes it more flexible in what it
  is able to decode. This bleeds into various other types, which inherit a generic parameter to represent this
  iterator that is used to drive decoding.

## 0.5.0

This release shifts `scale-decode` to being a mirror of a new `scale-encode` crate, and:
- Adds a new `IntoVisitor` trait that types can implement if there is a `Visitor` which can be used to decode
  into them.
- Adds a new `DecodeAsType` trait to mirror the `EncodeAsType` trait there; any type that implements `IntoVisitor`
  automatically implements `DecodeAsType`.
- Adds a new `DecodeAsFields` trait to mirror `EncodeAsFields`, implemented for tuple and struct types.
- Moves the `Visitor` trait into a sub module and re-works the interface to allow zero copy decoding, allow more
  concise implementations of it, and provide a fallback escape hatch to allow for more arbitrary `DecodeAsType`
  implementations from it.
- Implements `DecodeAsType` (via `Visitor` and `IntoVisitor` impls) and `DecodeAsFields` on common types.
- Adds a `DecodeAsType` derive macro to auto-generate impls on custom struct and enum types.

Any `Visitor` impls will need to be updated to use the refined `Visitor` trait; this should be fairly mechanical
(check out the examples and follow the compiler guidance to do this). Otherwise, the rest of the changes are
additive and just make it easier to implement this trait and obtain a `DecodeAsType` implementation, if desired.

### Changed

- Add DecodeAsType backed by Visitor impls for standard types. ([#11](https://github.com/paritytech/scale-decode/pull/11))

## 0.4.0

This release removes `bitvec` and the 32bit feature flag needed to play nicely with it and leans on `scale-bits` instead
to decode bit sequences. We add a CI check to ensure that it can be compiled to WASM.

### Changed

- Use scale-bits to handle bit sequences ([#5](https://github.com/paritytech/scale-decode/pull/5)

## 0.3.0

This release makes the following changes:
- All `visit_` functions are now handed a `TypeId` too, which is just a wrapper around a `u32` corresponding
  to the type being decoded in the given PortableRegistry. Useful if you'd like to look up more details about
  the type in the registry or just store the ID somewhere.
- `visit_compact_` functions are now handed a `visitor::Compact` struct which one can obtain the compact encoded
  value from, or view the path to the compact encoded value via a `.locations()` method, in case the compact value
  is actually nested within named/unnamed structs. The TypeId provided is always the outermost type of the Compact
  value, and so one can also discover this information by manual traversal (but since we have to traverse anyway..).
- The `Variant` type has been simplified and largely just allows access to the underlying `Composite` type to decode.
- The `Composite` type now provides direct access to the (yet-to-be-decoded) `fields`, and offers separate
  `decode_item` and `decode_item_with_name` methods to make decoding into named or unnamed shapes a little easier.
- `Visitor` related types are now exported directly from the `visitor` module rather than a `visitor::types` module.
- Lifetimes have been made more precise to avoid unnecessary lifetime related errors.

### Changed

- TypeIds, more info for compact encoded values and tidy up ([#1](https://github.com/paritytech/scale-decode/pull/1))

## 0.2.0

- Remove `remaining()` functions from visitor structs; the `len()` calls now return the
items left to decode.
- Fix clippy and doc links.

## 0.1.0

Initial release containging a `decode` function, `Visitor` trait to implement, and an
`IgnoreVisitor` impl to skip over SCALE bytes instead of decode them into some type.
