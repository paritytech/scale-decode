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

use codec::Encode;
use scale_decode::{error::ErrorKind, DecodeAsType, Error, FieldIter, IntoVisitor, Visitor};
use std::collections::HashMap;

// We have some enum Foo that we'll encode to bytes. The aim of this example is
// to manually write a decoder for it.
//
// Note: we can derive this automatically via the `DecodeAsType` derive macro.
#[derive(scale_info::TypeInfo, codec::Encode, PartialEq, Debug)]
enum Foo {
    Bar { bar: bool },
    Wibble(u32),
    Empty,
}

// Define a struct that will be our `Visitor` capable of decoding to a `Foo`.
struct FooVisitor;

// Describe how to turn `Foo` into this visitor struct.
impl IntoVisitor for Foo {
    type Visitor = FooVisitor;
    fn into_visitor() -> Self::Visitor {
        FooVisitor
    }
}

// Next, implement `Visitor` on this struct. if we set `Error` to be `scale_decode::Error` (or
// any error type that can be converted into a `scale_decode::Error`), we'll also get a `DecodeAsType`
// implementation for free (and it will compose nicely with other types that implement `DecodeAsType`).
// We can opt not to do this if we prefer.
impl Visitor for FooVisitor {
    type Value<'scale, 'info> = Foo;
    type Error = Error;

    // We have opted here to be quite flexible in what we support; we'll happily ignore fields in the input that we
    // don't care about and support unnamed to named fields. You could choose to be more strict if you prefer. We also
    // add context to any errors coming from decoding sub-types via `.map_err(|e| e.at_x(..))` calls.
    fn visit_variant<'scale, 'info, I: FieldIter<'info>>(
        self,
        value: &mut scale_decode::visitor::types::Variant<'scale, 'info, I>,
        _type_id: scale_decode::visitor::TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        if value.name() == "Bar" {
            // Here we choose to support decoding named or unnamed fields into our Bar variant.
            let fields = value.fields();
            if fields.has_unnamed_fields() {
                if fields.remaining() != 1 {
                    return Err(Error::new(ErrorKind::WrongLength {
                        actual_len: fields.remaining(),
                        expected_len: 1,
                    }));
                }
                let bar = fields.next().unwrap()?;
                Ok(Foo::Bar { bar: bar.decode_as_type().map_err(|e| e.at_field("bar"))? })
            } else {
                let vals: HashMap<Option<&str>, _> = fields
                    .map(|res| res.map(|item| (item.name(), item)))
                    .collect::<Result<_, _>>()?;
                let bar = *vals.get(&Some("bar")).ok_or_else(|| {
                    Error::new(ErrorKind::CannotFindField { name: "bar".to_owned() })
                })?;
                Ok(Foo::Bar { bar: bar.decode_as_type().map_err(|e| e.at_field("bar"))? })
            }
        } else if value.name() == "Wibble" {
            // If we have the same number of fields (named or unnamed) we'll try to line them up by index.
            let fields = value.fields();
            if fields.remaining() != 1 {
                return Err(Error::new(ErrorKind::WrongLength {
                    actual_len: fields.remaining(),
                    expected_len: 1,
                }));
            }
            let val = fields.next().unwrap()?;
            Ok(Foo::Wibble(val.decode_as_type().map_err(|e| e.at_idx(0))?))
        } else if value.name() == "Empty" {
            // If we want no fields, we ignore any fields we're given and emit the variant name.
            Ok(Foo::Empty)
        } else {
            // The variant name doesn't match; we can't decode!
            Err(Error::new(ErrorKind::CannotFindVariant {
                got: value.name().to_string(),
                expected: vec!["Bar", "Wibble", "Empty"],
            }))
        }
    }
}

fn main() {
    let bar = Foo::Bar { bar: true };
    let wibble = Foo::Wibble(12345);
    let empty = Foo::Empty;

    // First, the setup. We encode our variants to bytes, and use `TypeInfo` to give us
    // a type registry and ID describing the shape of the enum. Substrate metadata contains
    // all of this type information already.
    let bar_bytes = bar.encode();
    let wibble_bytes = wibble.encode();
    let empty_bytes = empty.encode();

    let (type_id, types) = make_type::<Foo>();

    // We can decode via `DecodeAsType`, which is automatically implemented:
    let bar_via_decode_as_type = Foo::decode_as_type(&mut &*bar_bytes, type_id, &types).unwrap();
    let wibble_via_decode_as_type =
        Foo::decode_as_type(&mut &*wibble_bytes, type_id, &types).unwrap();
    let empty_via_decode_as_type =
        Foo::decode_as_type(&mut &*empty_bytes, type_id, &types).unwrap();

    // Or we can also manually use our `Visitor` impl:
    let bar_via_visitor =
        scale_decode::visitor::decode_with_visitor(&mut &*bar_bytes, type_id, &types, FooVisitor)
            .unwrap();
    let wibble_via_visitor = scale_decode::visitor::decode_with_visitor(
        &mut &*wibble_bytes,
        type_id,
        &types,
        FooVisitor,
    )
    .unwrap();
    let empty_via_visitor =
        scale_decode::visitor::decode_with_visitor(&mut &*empty_bytes, type_id, &types, FooVisitor)
            .unwrap();

    assert_eq!(bar, bar_via_decode_as_type);
    assert_eq!(bar, bar_via_visitor);
    assert_eq!(wibble, wibble_via_decode_as_type);
    assert_eq!(wibble, wibble_via_visitor);
    assert_eq!(empty, empty_via_decode_as_type);
    assert_eq!(empty, empty_via_visitor);
}

// Normally we'd have a type registry to hand already, but if not, we can build our own:
fn make_type<T: scale_info::TypeInfo + 'static>() -> (u32, scale_info::PortableRegistry) {
    let m = scale_info::MetaType::new::<T>();
    let mut types = scale_info::Registry::new();
    let id = types.register_type(&m);
    let portable_registry: scale_info::PortableRegistry = types.into();

    (id.id, portable_registry)
}
