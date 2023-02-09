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

use crate::visitor::types::*;
use crate::visitor::{DecodeAsTypeResult, DecodeError, TypeId, Visitor};

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
        match self.visitor.unchecked_decode_as_type(input, type_id, types) {
            DecodeAsTypeResult::Decoded(r) => DecodeAsTypeResult::Decoded(self.mapper.call(r)),
            DecodeAsTypeResult::Skipped(v) => {
                DecodeAsTypeResult::Skipped(AndThen { visitor: v, mapper: self.mapper })
            }
        }
    }
    fn visit_bool<'scale, 'info>(
        self,
        value: bool,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_bool(value, type_id))
    }
    fn visit_char<'scale, 'info>(
        self,
        value: char,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_char(value, type_id))
    }
    fn visit_u8<'scale, 'info>(
        self,
        value: u8,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_u8(value, type_id))
    }
    fn visit_u16<'scale, 'info>(
        self,
        value: u16,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_u16(value, type_id))
    }
    fn visit_u32<'scale, 'info>(
        self,
        value: u32,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_u32(value, type_id))
    }
    fn visit_u64<'scale, 'info>(
        self,
        value: u64,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_u64(value, type_id))
    }
    fn visit_u128<'scale, 'info>(
        self,
        value: u128,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_u128(value, type_id))
    }
    fn visit_u256<'info>(
        self,
        value: &'_ [u8; 32],
        type_id: TypeId,
    ) -> Result<Self::Value<'_, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_u256(value, type_id))
    }
    fn visit_i8<'scale, 'info>(
        self,
        value: i8,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_i8(value, type_id))
    }
    fn visit_i16<'scale, 'info>(
        self,
        value: i16,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_i16(value, type_id))
    }
    fn visit_i32<'scale, 'info>(
        self,
        value: i32,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_i32(value, type_id))
    }
    fn visit_i64<'scale, 'info>(
        self,
        value: i64,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_i64(value, type_id))
    }
    fn visit_i128<'scale, 'info>(
        self,
        value: i128,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_i128(value, type_id))
    }
    fn visit_i256<'info>(
        self,
        value: &'_ [u8; 32],
        type_id: TypeId,
    ) -> Result<Self::Value<'_, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_i256(value, type_id))
    }
    fn visit_sequence<'scale, 'info>(
        self,
        value: &mut Sequence<'scale, 'info>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_sequence(value, type_id))
    }
    fn visit_composite<'scale, 'info>(
        self,
        value: &mut Composite<'scale, 'info>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_composite(value, type_id))
    }
    fn visit_tuple<'scale, 'info>(
        self,
        value: &mut Tuple<'scale, 'info>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_tuple(value, type_id))
    }
    fn visit_str<'scale, 'info>(
        self,
        value: &mut Str<'scale>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_str(value, type_id))
    }
    fn visit_variant<'scale, 'info>(
        self,
        value: &mut Variant<'scale, 'info>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_variant(value, type_id))
    }
    fn visit_array<'scale, 'info>(
        self,
        value: &mut Array<'scale, 'info>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_array(value, type_id))
    }
    fn visit_bitsequence<'scale, 'info>(
        self,
        value: &mut BitSequence<'scale>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_bitsequence(value, type_id))
    }
    fn visit_compact_u8<'scale, 'info>(
        self,
        value: Compact<u8>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_compact_u8(value, type_id))
    }
    fn visit_compact_u16<'scale, 'info>(
        self,
        value: Compact<u16>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_compact_u16(value, type_id))
    }
    fn visit_compact_u32<'scale, 'info>(
        self,
        value: Compact<u32>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_compact_u32(value, type_id))
    }
    fn visit_compact_u64<'scale, 'info>(
        self,
        value: Compact<u64>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_compact_u64(value, type_id))
    }
    fn visit_compact_u128<'scale, 'info>(
        self,
        value: Compact<u128>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
        self.mapper.call(self.visitor.visit_compact_u128(value, type_id))
    }
}
