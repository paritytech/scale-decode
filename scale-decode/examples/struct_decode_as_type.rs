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
use scale_decode::{error::ErrorKind, DecodeAsType, Error, IntoVisitor, Visitor};
use std::collections::HashMap;

// We have some struct Foo that we'll encode to bytes. The aim of this example is
// to manually write a decoder for it.
//
// Note: we can derive this automatically via the `DecodeAsType` derive macro.
#[derive(scale_info::TypeInfo, codec::Encode, PartialEq, Debug)]
struct Foo {
    bar: bool,
    wibble: u32,
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

    // Support decoding from composite types. We support decoding from either named or
    // unnamed fields (matching by field index if unnamed) and add context to errors via
    // `.map_err(|e| e.at_x(..))` calls to give back more precise information about where
    // decoding failed, if it does.
    fn visit_composite<'scale, 'info>(
        self,
        value: &mut scale_decode::visitor::types::Composite<'scale, 'info>,
        type_id: scale_decode::visitor::TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        if value.has_unnamed_fields() {
            // handle it like a tuple if there are unnamed fields in it:
            return self.visit_tuple(&mut value.as_tuple(), type_id);
        }

        let vals: HashMap<Option<&str>, _> =
            value.map(|res| res.map(|item| (item.name(), item))).collect::<Result<_, _>>()?;

        let bar = *vals
            .get(&Some("bar"))
            .ok_or_else(|| Error::new(ErrorKind::CannotFindField { name: "bar".to_owned() }))?;
        let wibble = *vals
            .get(&Some("wibble"))
            .ok_or_else(|| Error::new(ErrorKind::CannotFindField { name: "wibble".to_owned() }))?;

        Ok(Foo {
            bar: bar.decode_as_type().map_err(|e| e.at_field("bar"))?,
            wibble: wibble.decode_as_type().map_err(|e| e.at_field("wibble"))?,
        })
    }

    // If we like, we can also support decoding from tuples of matching lengths:
    fn visit_tuple<'scale, 'info>(
        self,
        value: &mut scale_decode::visitor::types::Tuple<'scale, 'info>,
        _type_id: scale_decode::visitor::TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        if value.remaining() != 2 {
            return Err(Error::new(ErrorKind::WrongLength {
                actual_len: value.remaining(),
                expected_len: 2,
            }));
        }

        let bar = value.next().unwrap()?;
        let wibble = value.next().unwrap()?;

        Ok(Foo {
            bar: bar.decode_as_type().map_err(|e| e.at_field("bar"))?,
            wibble: wibble.decode_as_type().map_err(|e| e.at_field("wibble"))?,
        })
    }
}

fn main() {
    let foo_original = Foo { bar: true, wibble: 12345 };

    // First, the setup. We encode Foo to bytes, and use `TypeInfo` to give us
    // a type registry and ID describing the shape of it. Substrate metadata contains
    // all of this type information already.
    let foo_bytes = foo_original.encode();
    let (type_id, types) = make_type::<Foo>();

    // We can decode via `DecodeAsType`, which is automatically implemented:
    let foo_via_decode_as_type = Foo::decode_as_type(&mut &*foo_bytes, type_id, &types).unwrap();
    // We can also attempt to decode it into any other type; we'll get an error if this fails:
    let foo_via_decode_as_type_arc =
        <std::sync::Arc<Foo>>::decode_as_type(&mut &*foo_bytes, type_id, &types).unwrap();
    // Or we can also manually use our `Visitor` impl:
    let foo_via_visitor = scale_decode::visitor::decode_with_visitor(
        &mut &*foo_bytes,
        type_id,
        &types,
        FooVisitor,
        false,
    )
    .unwrap();

    assert_eq!(foo_original, foo_via_decode_as_type);
    assert_eq!(&foo_original, &*foo_via_decode_as_type_arc);
    assert_eq!(foo_original, foo_via_visitor);
}

// Normally we'd have a type registry to hand already, but if not, we can build our own:
fn make_type<T: scale_info::TypeInfo + 'static>() -> (u32, scale_info::PortableRegistry) {
    let m = scale_info::MetaType::new::<T>();
    let mut types = scale_info::Registry::new();
    let id = types.register_type(&m);
    let portable_registry: scale_info::PortableRegistry = types.into();

    (id.id, portable_registry)
}
