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

use super::BasicVisitor;
use crate::{
    error::Error,
    visitor::{decode_with_visitor, DecodeAsTypeResult, Visitor},
    IntoVisitor,
};
use primitive_types::{H128, H160, H256, H384, H512, H768};
use scale_type_resolver::TypeResolver;

macro_rules! impl_visitor {
    ($ty:ty: $len:literal) => {
        impl<R: TypeResolver> Visitor for BasicVisitor<$ty, R> {
            type Error = Error;
            type Value<'scale, 'resolver> = $ty;
            type TypeResolver = R;

            fn unchecked_decode_as_type<'scale, 'resolver>(
                self,
                input: &mut &'scale [u8],
                type_id: &<Self::TypeResolver as TypeResolver>::TypeId,
                types: &'resolver Self::TypeResolver,
            ) -> crate::visitor::DecodeAsTypeResult<
                Self,
                Result<Self::Value<'scale, 'resolver>, Self::Error>,
            > {
                let res = decode_with_visitor(
                    input,
                    type_id,
                    types,
                    BasicVisitor::<[u8; $len / 8], R> { _marker: core::marker::PhantomData },
                )
                .map(|res| <$ty>::from(res));
                DecodeAsTypeResult::Decoded(res)
            }
        }

        impl IntoVisitor for $ty {
            type AnyVisitor<R: TypeResolver> = BasicVisitor<$ty, R>;
            fn into_visitor<R: TypeResolver>() -> Self::AnyVisitor<R> {
                BasicVisitor { _marker: core::marker::PhantomData }
            }
        }
    };
}
impl_visitor!(H128: 128);
impl_visitor!(H160: 160);
impl_visitor!(H256: 256);
impl_visitor!(H384: 384);
impl_visitor!(H512: 512);
impl_visitor!(H768: 768);
