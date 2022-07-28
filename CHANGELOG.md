# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

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