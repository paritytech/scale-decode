#![no_std]
extern crate alloc;

use scale_decode::DecodeAsType;

pub struct NotDecodeAsType;

// Enums with generic params and even lifetimes can impl DecodeAsType.
#[derive(DecodeAsType)]
pub enum Bar<'a, T, U, V> {
    Wibble(bool, T, U, V),
    Wobble,
    Boo(alloc::borrow::Cow<'a, str>),
}

// This impls DecodeAsType ok; we set no default trait bounds.
#[derive(DecodeAsType)]
#[decode_as_type(trait_bounds = "")]
pub enum NoTraitBounds<T> {
    Wibble(core::marker::PhantomData<T>),
}

// Structs (and const bounds) impl DecodeAsType OK.
#[derive(DecodeAsType)]
pub struct MyStruct<const V: usize, Bar: Clone + PartialEq> {
    pub array: [Bar; V],
}

pub fn can_decode_as_type<T: DecodeAsType>() {}
