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

#[derive(DecodeAsType)]
struct Foo {
    some_field: u8,
    value: u16,
    #[decode_as_type(skip)]
    some_field_to_skip: bool,
}

#[derive(DecodeAsType)]
struct Foo2(String, #[decode_as_type(skip)] usize, bool);

#[derive(DecodeAsType)]
enum Foo3 {
    NamedField {
        some_field: u8,
        value: u16,
        #[decode_as_type(skip)]
        some_field_to_skip: bool,
    },
    UnnamedField (
        String,
        // The codec attr will work too, for compat:
        #[codec(skip)] usize,
        bool
    )
}

fn can_decode_as_type<T: DecodeAsType>() {}

fn main() {
    // assert that the trait is implemented:
    can_decode_as_type::<Foo>();
    can_decode_as_type::<Foo2>();
    can_decode_as_type::<Foo3>();
}