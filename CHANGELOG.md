# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## 0.2.0

- Remove `remaining()` functions from visitor structs; the `len()` calls now return the
items left to decode.
- Fix clippy and doc links.

## 0.1.0

Initial release containging a `decode` function, `Visitor` trait to implement, and an
`IgnoreVisitor` impl to skip over SCALE bytes instead of decode them into some type.