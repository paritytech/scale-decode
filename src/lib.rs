// Copyright (C) 2022 Parity Technologies (UK) Ltd. (admin@parity.io)
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

// #![deny(missing_docs)]

// BitVec only supports u64 BitStore if `target_pointer_width = "64"`.
// Turn this into a feature so it can be tested, and use to avoid using
// this store type on 32bit architectures.
#![cfg_attr(
    not(target_pointer_width = "64"),
    feature(32bit_target)
)]

mod bit_sequence;
mod decode;

pub mod visitor;
pub use decode::decode;