#![no_std]
extern crate alloc;

use scale_decode::DecodeAsType;

pub struct NotDecodeAsType;

// Enums with generic params can impl EncodeAsType.
#[derive(DecodeAsType)]
pub enum Bar<T, U, V> {
    Wibble(bool, T, U, V),
    Wobble,
}

// This impls EncodeAsType ok; we set no default trait bounds.
#[derive(DecodeAsType)]
#[decode_as_type(trait_bounds = "")]
pub enum NoTraitBounds<T> {
    Wibble(core::marker::PhantomData<T>),
}

// Structs (and const bounds) impl EncodeAsType OK.
#[derive(DecodeAsType)]
pub struct MyStruct<const V: usize, Bar: Clone + PartialEq> {
    _array: [Bar; V],
}

pub fn can_decode_as_type<T: DecodeAsType>() {}
