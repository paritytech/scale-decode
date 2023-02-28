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
    // We just ignore random "codec" attrs. In part, we allow this because
    // we want to support `#[codec(skip)]` but ignore any other #[codec] attrs
    // that might exist. In part we do it so that when `DecodeAsType` is derived,
    // we can leave #[codec] things on the type in generated code without any issues,
    // so that they can kick in if `#[codec::Decode]` etc is later added.
    #[codec(compact)]
    value: u16,
    #[codec(index = 2)]
    type_id: bool,
}

fn can_decode_as_type<T: DecodeAsType>() {}

fn main() {
    can_decode_as_type::<Foo>();
}