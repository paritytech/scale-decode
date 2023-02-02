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
use crate::visitor::{DecodeError, TypeId, Visitor};

/// Transform the result from a visitor. This type also implements [`Visitor`].
pub struct AndThen<V, F, O, E> {
    visitor: V,
    mapper: F,
    _marker: std::marker::PhantomData<(O, E)>,
}

impl<V, F, O, E> AndThen<V, F, O, E> {
    /// Transform the result obtained from a visitor given the provided function.
    pub fn new(visitor: V, f: F) -> AndThen<V, F, O, E> {
        AndThen { visitor, mapper: f, _marker: std::marker::PhantomData }
    }
}

impl<V, F, O, E> Visitor for AndThen<V, F, O, E>
where
    V: Visitor,
    F: for<'b> FnOnce(Result<V::Value<'b>, V::Error>) -> Result<O, E>,
    E: From<DecodeError>,
{
    type Value<'scale> = O;
    type Error = E;

    fn visit_bool<'scale>(
        self,
        value: bool,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_bool(value, type_id))
    }
    fn visit_char<'scale>(
        self,
        value: char,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_char(value, type_id))
    }
    fn visit_u8<'scale>(
        self,
        value: u8,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_u8(value, type_id))
    }
    fn visit_u16<'scale>(
        self,
        value: u16,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_u16(value, type_id))
    }
    fn visit_u32<'scale>(
        self,
        value: u32,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_u32(value, type_id))
    }
    fn visit_u64<'scale>(
        self,
        value: u64,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_u64(value, type_id))
    }
    fn visit_u128<'scale>(
        self,
        value: u128,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_u128(value, type_id))
    }
    fn visit_u256(
        self,
        value: &'_ [u8; 32],
        type_id: TypeId,
    ) -> Result<Self::Value<'_>, Self::Error> {
        (self.mapper)(self.visitor.visit_u256(value, type_id))
    }
    fn visit_i8<'scale>(
        self,
        value: i8,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_i8(value, type_id))
    }
    fn visit_i16<'scale>(
        self,
        value: i16,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_i16(value, type_id))
    }
    fn visit_i32<'scale>(
        self,
        value: i32,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_i32(value, type_id))
    }
    fn visit_i64<'scale>(
        self,
        value: i64,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_i64(value, type_id))
    }
    fn visit_i128<'scale>(
        self,
        value: i128,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_i128(value, type_id))
    }
    fn visit_i256(
        self,
        value: &'_ [u8; 32],
        type_id: TypeId,
    ) -> Result<Self::Value<'_>, Self::Error> {
        (self.mapper)(self.visitor.visit_i256(value, type_id))
    }
    fn visit_sequence<'scale>(
        self,
        value: &mut Sequence<'scale, '_>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_sequence(value, type_id))
    }
    fn visit_composite<'scale>(
        self,
        value: &mut Composite<'scale, '_>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_composite(value, type_id))
    }
    fn visit_tuple<'scale>(
        self,
        value: &mut Tuple<'scale, '_>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_tuple(value, type_id))
    }
    fn visit_str<'scale>(
        self,
        value: &mut Str<'scale>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_str(value, type_id))
    }
    fn visit_variant<'scale>(
        self,
        value: &mut Variant<'scale, '_>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_variant(value, type_id))
    }
    fn visit_array<'scale>(
        self,
        value: &mut Array<'scale, '_>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_array(value, type_id))
    }
    fn visit_bitsequence<'scale>(
        self,
        value: &mut BitSequence<'scale>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_bitsequence(value, type_id))
    }
    fn visit_compact_u8<'scale>(
        self,
        value: Compact<u8>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_compact_u8(value, type_id))
    }
    fn visit_compact_u16<'scale>(
        self,
        value: Compact<u16>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_compact_u16(value, type_id))
    }
    fn visit_compact_u32<'scale>(
        self,
        value: Compact<u32>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_compact_u32(value, type_id))
    }
    fn visit_compact_u64<'scale>(
        self,
        value: Compact<u64>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_compact_u64(value, type_id))
    }
    fn visit_compact_u128<'scale>(
        self,
        value: Compact<u128>,
        type_id: TypeId,
    ) -> Result<Self::Value<'scale>, Self::Error> {
        (self.mapper)(self.visitor.visit_compact_u128(value, type_id))
    }
}
