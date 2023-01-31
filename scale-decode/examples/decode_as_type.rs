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

use codec::Encode;
use scale_decode::{error::ErrorKind, DecodeAsType, Error, IntoVisitor, Visitor};
use std::collections::HashMap;

// We have some struct Foo that we'll be encoding. The aim of this example is
// to write a decoder for it
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
    type Value<'scale> = Foo;
    type Error = Error;

    fn visit_composite<'scale>(
        self,
        value: &mut scale_decode::visitor::types::Composite<'scale, '_>,
        _type_id: scale_decode::visitor::TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        let vals: HashMap<Option<&str>, _> =
            value.map(|res| res.map(|item| (item.name(), item))).collect::<Result<_, _>>()?;

        let bar = *vals
            .get(&Some("bar"))
            .ok_or_else(|| Error::new(ErrorKind::CannotFindField { name: "bar".to_owned() }))?;
        let wibble = *vals
            .get(&Some("wibble"))
            .ok_or_else(|| Error::new(ErrorKind::CannotFindField { name: "wibble".to_owned() }))?;

        Ok(Foo { bar: bar.decode_as_type()?, wibble: wibble.decode_as_type()? })
    }
}

fn main() {
    let foo = Foo { bar: true, wibble: 12345 };

    // First, the setup. We encode Foo to bytes, and use `TypeInfo` to give us
    // a type registry and ID describing the shape of it. Substrate metadata contains
    // all of this type information already.
    let foo_bytes = foo.encode();
    let (type_id, types) = make_type::<Foo>();

    // We can decode via `DecodeAsType`, which is automatically implemented:
    let foo_via_decode_as_type = Foo::decode_as_type(&mut &*foo_bytes, type_id, &types).unwrap();
    // We can also attempt to decode it into any other type; we'll get an error if this fails:
    let foo_via_decode_as_type_arc = <std::sync::Arc<Foo>>::decode_as_type(&mut &*foo_bytes, type_id, &types).unwrap();
    // Or we can also manually use our `Visitor` impl:
    let foo_via_visitor =
        scale_decode::visitor::decode_with_visitor(&mut &*foo_bytes, type_id, &types, FooVisitor)
            .unwrap();

    assert_eq!(foo, foo_via_decode_as_type);
    assert_eq!(&foo, &*foo_via_decode_as_type_arc);
    assert_eq!(foo, foo_via_visitor);
}

// Normally we'd have a type registry to hand already, but if not, we can build our own:
fn make_type<T: scale_info::TypeInfo + 'static>() -> (u32, scale_info::PortableRegistry) {
    let m = scale_info::MetaType::new::<T>();
    let mut types = scale_info::Registry::new();
    let id = types.register_type(&m);
    let portable_registry: scale_info::PortableRegistry = types.into();

    (id.id(), portable_registry)
}
