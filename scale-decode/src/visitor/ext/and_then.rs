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

use crate::visitor::{decode_with_visitor, DecodeAsTypeResult, DecodeError, TypeId, Visitor};

/// Transform the result from a visitor. This type also implements [`Visitor`].
pub struct AndThen<V, F> {
    visitor: V,
    mapper: F,
}

impl<V, F> AndThen<V, F> {
    /// Transform the result obtained from a visitor given the provided function.
    pub fn new(visitor: V, f: F) -> AndThen<V, F> {
        AndThen { visitor, mapper: f }
    }
}

/// This trait is implemented for all valid function types that can be passed to [`AndThen`].
/// This exists just to remove a couple of generic parameters from [`AndThen`] which would
/// otherwise need repeating from the function return type, simplifying implementations a little.
pub trait AndThenFn<V>
where
    V: Visitor,
{
    /// The value returned on success.
    type Value;
    /// The error returned on failure.
    type Error: From<DecodeError>;
    /// Call the provided function.
    fn call(self, val: Result<V::Value<'_, '_>, V::Error>) -> Result<Self::Value, Self::Error>;
}
impl<F, V, O, E> AndThenFn<V> for F
where
    V: Visitor,
    F: for<'scale, 'info> FnOnce(Result<V::Value<'scale, 'info>, V::Error>) -> Result<O, E>,
    E: From<DecodeError>,
{
    type Value = O;
    type Error = E;
    fn call(self, val: Result<V::Value<'_, '_>, V::Error>) -> Result<O, E> {
        (self)(val)
    }
}

// Implemenet Visitor on AndThen as long as V is a Visitor and F a valid transform function.
impl<V, F> Visitor for AndThen<V, F>
where
    V: Visitor,
    F: AndThenFn<V>,
{
    type Value<'scale, 'info> = F::Value;
    type Error = F::Error;

    fn unchecked_decode_as_type<'scale, 'info>(
        self,
        input: &mut &'scale [u8],
        type_id: TypeId,
        types: &'info scale_info::PortableRegistry,
    ) -> DecodeAsTypeResult<Self, Result<Self::Value<'scale, 'info>, Self::Error>> {
        // Use the original visitor to decode into some type:
        let inner_res = decode_with_visitor(input, type_id.0, types, self.visitor);
        // map this type into our desired output and return it:
        let res = self.mapper.call(inner_res);
        DecodeAsTypeResult::Decoded(res)
    }
}
