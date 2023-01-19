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

//! This crate is concerned with decoding arbitrary values from some
//! SCALE encoded bytes, given a type ID and type registry that defines
//! the expected shape that the bytes should be decoded into.
//!
//! In order to allow the user to decode bytes into any shape they like,
//! you must implement a [`visitor::Visitor`] trait, which is handed
//! values back and has the opportunity to transform them into some
//! output representation of your choice (or fail with an error of your
//! choice). This Visitor is passed to the [`decode()`] method, whose job it
//! is to look at the type information provided and pass values of those
//! types to the Visitor, or fail if the bytes do not match the expected
//! shape.

#![deny(missing_docs)]

mod utils;

pub mod visitor;
