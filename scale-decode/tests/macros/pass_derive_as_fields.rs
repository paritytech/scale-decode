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

use scale_decode::{ DecodeAsType, DecodeAsFields };

// Single field named struct
#[derive(DecodeAsType)]
struct Foo {
    some_field: u8,
    value: u16,
    type_id: bool,
}

// Single field unnamed struct
#[derive(DecodeAsType)]
struct Foo2(String);

// Multi field unnamed struct
#[derive(DecodeAsType)]
struct Foo3(String, bool, u8, u8);

// This should be auto implemented and cause no compile issues:
fn can_decode_as_fields<T: DecodeAsFields>() {}

fn main() {
    // assert that the trait is implemented:
    can_decode_as_fields::<Foo>();
    can_decode_as_fields::<Foo2>();
    can_decode_as_fields::<Foo3>();
}