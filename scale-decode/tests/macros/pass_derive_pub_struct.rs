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

// struct is public, which means any geenrated visitor which can be returned via
// IntoVisitor needs to also be public.
#[derive(DecodeAsType)]
pub struct Foo(String);


fn can_decode_as_type<T: DecodeAsType>() {}

fn main() {
    // assert that the trait is implemented:
    can_decode_as_type::<Foo>();
}