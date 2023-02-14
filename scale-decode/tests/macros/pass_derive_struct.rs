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

use scale_decode::DecodeAsType;

// Single field named struct
#[derive(DecodeAsType)]
// this should lead to no issues:
#[decode_as_type(crate_path = "::scale_decode")]
struct Foo {
    some_field: u8,
    // fields with same name as internal macro variable; make sure
    // no name conflicts can happen:
    value: u16,
    type_id: bool,
}

// Single field unnamed struct
#[derive(DecodeAsType)]
struct Foo2(String);

// Multi field unnamed struct
#[derive(DecodeAsType)]
struct Foo3(String, bool, u8, u8);

// Multi field named struct (using names commonly
// used i nthe trait definition):
#[derive(DecodeAsType)]
struct Foo4 {
    ty: u8,
    out: bool,
    context: String,
    types: u64
}

fn can_decode_as_type<T: DecodeAsType>() {}

fn main() {
    // assert that the trait is implemented:
    can_decode_as_type::<Foo>();
    can_decode_as_type::<Foo2>();
    can_decode_as_type::<Foo3>();
    can_decode_as_type::<Foo4>();
}