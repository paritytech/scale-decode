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

//! This module provides a [`Context`] type that must be provided with every
//! attempt to encode some type. Internally, the [`Context`] tracks the path
//! that we're attempting to encode to aid in error reporting.

use crate::utils::linkedlist::LinkedList;
use std::borrow::Cow;

/// A cheaply clonable opaque context which allows us to track the current
/// location into a type that we're trying to encode, to aid in
/// error reporting.
#[derive(Clone, Default, Debug)]
pub struct Context {
    path: LinkedList<Location>,
}

impl Context {
    /// Construct a new, empty context.
    pub fn new() -> Context {
        Default::default()
    }
    /// Return a new context with the given location appended.
    pub fn at(&self, loc: Location) -> Context {
        let path = self.path.clone().push(loc);
        Context { path }
    }
    /// Return a new context with a field location appended.
    pub fn at_field(&self, field: impl Into<Cow<'static, str>>) -> Context {
        self.at(Location::field(field))
    }
    /// Return a new context with a variant location appended.
    pub fn at_variant(&self, name: impl Into<Cow<'static, str>>) -> Context {
        self.at(Location::variant(name))
    }
    /// Return a new context with an index location appended.
    pub fn at_idx(&self, i: usize) -> Context {
        self.at(Location::idx(i))
    }
    /// Return the current path.
    pub fn path(&self) -> Path<'_> {
        Path(Cow::Borrowed(&self.path))
    }
    /// Return true if the context is empty.
    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }
}

/// The current path that we're trying to encode.
pub struct Path<'a>(Cow<'a, LinkedList<Location>>);

impl<'a> Path<'a> {
    /// Cheaply convert the path to an owned version.
    pub fn to_owned(self) -> Path<'static> {
        Path(Cow::Owned(self.0.into_owned()))
    }
    /// Return each location visited, most recent first.
    pub fn locations(&self) -> impl Iterator<Item = &Location> {
        self.0.iter_back()
    }
}

impl<'a> std::fmt::Display for Path<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut items = Vec::with_capacity(self.0.len());
        for item in self.0.iter_back() {
            items.push(item);
        }

        for (idx, loc) in items.iter().rev().enumerate() {
            if idx != 0 {
                f.write_str(".")?;
            }
            match &loc.inner {
                Loc::Field(name) => f.write_str(&*name)?,
                Loc::Index(i) => write!(f, "[{i}]")?,
                Loc::Variant(name) => write!(f, "({name})")?,
            }
        }
        Ok(())
    }
}

/// Some location, like a field, variant or index in an array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    inner: Loc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Loc {
    Field(Cow<'static, str>),
    Index(usize),
    Variant(Cow<'static, str>),
}

impl Location {
    /// This represents some struct field.
    pub fn field(name: impl Into<Cow<'static, str>>) -> Self {
        Location { inner: Loc::Field(name.into()) }
    }
    /// This represents some variant name.
    pub fn variant(name: impl Into<Cow<'static, str>>) -> Self {
        Location { inner: Loc::Variant(name.into()) }
    }
    /// This represents a tuple or array index.
    pub fn idx(i: usize) -> Self {
        Location { inner: Loc::Index(i) }
    }
}
