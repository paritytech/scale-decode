# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

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

Any `Visitor` impls wil need to be updated to use the refined `Visitor` trait; this should be fairly mechanical
(check out the examples and follow the compiler guidance to do this). Otherwise, the rest of the changes are
additive and just make it easier to implement this trait and obtain a `DecodeAsType` implementation, if desired.

## Changed

- Add DecodeAsType backed by Visitor impls for standard types. ([#11](https://github.com/paritytech/scale-decode/pull/11))

## 0.4.0

This release removes `bitvec` and the 32bit feature flag needed to play nicely with it and leans on `scale-bits` instead to decode bit sequences. We add a CI check to ensure that it can be compiled to WASM.

## Changed

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

## Changed

- TypeIds, more info for compact encoded values and tidy up ([#1](https://github.com/paritytech/scale-decode/pull/1))

## 0.2.0

- Remove `remaining()` functions from visitor structs; the `len()` calls now return the
items left to decode.
- Fix clippy and doc links.

## 0.1.0

Initial release containging a `decode` function, `Visitor` trait to implement, and an
`IgnoreVisitor` impl to skip over SCALE bytes instead of decode them into some type.