use scale_info::{
    PortableRegistry,
    Field,
    form::PortableForm,
};

pub trait Visitor {
    type Value;
    type Error: From<DecodeError>;

    fn visit_bool(self, bool: bool) -> Result<Self::Value, Self::Error>;
    fn visit_char(self, value: char) -> Result<Self::Value, Self::Error>;
    fn visit_u8(self, value: u8) -> Result<Self::Value, Self::Error>;
    fn visit_u16(self, value: u16) -> Result<Self::Value, Self::Error>;
    fn visit_u32(self, value: u32) -> Result<Self::Value, Self::Error>;
    fn visit_u64(self, value: u64) -> Result<Self::Value, Self::Error>;
    fn visit_u128(self, value: u128) -> Result<Self::Value, Self::Error>;
    fn visit_u256(self, value: &[u8]) -> Result<Self::Value, Self::Error>;
    fn visit_i8(self, value: i8) -> Result<Self::Value, Self::Error>;
    fn visit_i16(self, value: i16) -> Result<Self::Value, Self::Error>;
    fn visit_i32(self, value: i32) -> Result<Self::Value, Self::Error>;
    fn visit_i64(self, value: i64) -> Result<Self::Value, Self::Error>;
    fn visit_i128(self, value: i128) -> Result<Self::Value, Self::Error>;
    fn visit_i256(self, value: &[u8]) -> Result<Self::Value, Self::Error>;
    fn visit_sequence(self, items: &mut Sequence<'_>) -> Result<Self::Value, Self::Error>;
    fn visit_composite(self, items: &mut Fields<'_>) -> Result<Self::Value, Self::Error>;
    fn visit_tuple(self, value: &mut Tuple<'_>) -> Result<Self::Value, Self::Error>;

    // fn visit_variant(self, name: &str, fields: &mut Items<'_>) -> Result<Self::Value, Self::Error>;
    // fn visit_array(self, value: Array<'_>) -> Result<Self::Value, Self::Error>;
    // fn visit_compact(self, value: u32) -> Result<Self::Value, Self::Error>;
    // fn visit_str(self, value: Str<'_>) -> Result<Self::Value, Self::Error>;

    // // A weird one; want to avoid decoding into bitsequence if we can but let's see.
    // fn visit_bitsequence(self, value: BitSequence<'_>) -> Result<Self::Value, Self::Error>;
}

// This enables a visitor to decode information out of composite or variant fields.
pub struct Tuple<'a> {
    bytes: &'a [u8],
    fields: &'a [scale_info::interner::UntrackedSymbol<std::any::TypeId>],
    types: &'a PortableRegistry,
    len: usize,
}

impl <'a> Tuple<'a> {
    pub (crate) fn new(
        bytes: &'a [u8],
        fields: &'a [scale_info::interner::UntrackedSymbol<std::any::TypeId>],
        types: &'a PortableRegistry,
    ) -> Tuple<'a> {
        Tuple { len: fields.len(), bytes, fields, types }
    }
    pub (crate) fn bytes(&self) -> &'a [u8] {
        self.bytes
    }
    pub (crate) fn skip_rest(&mut self) -> Result<(), DecodeError> {
        while !self.fields.is_empty() {
            self.decode_item(IgnoreVisitor)?;
        }
        Ok(())
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn remaining(&self) -> usize {
        self.fields.len()
    }
    pub fn decode_item<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, V::Error> {
        if self.fields.is_empty() {
            return Err(DecodeError::NoItemsLeft.into())
        }

        let field = &self.fields[0];
        self.fields = &self.fields[1..];

        let b = &mut self.bytes;
        // Don't return here; decrement bytes properly first and then return, so that
        // calling decode_item again works as expected.
        let res = super::decode(b, field.id(), self.types, visitor);
        self.bytes = *b;
        res
    }
}

// This enables a visitor to decode information out of composite or variant fields.
pub struct Fields<'a> {
    bytes: &'a [u8],
    fields: &'a [Field<PortableForm>],
    types: &'a PortableRegistry,
    len: usize,
}

impl <'a> Fields<'a> {
    pub (crate) fn new(
        bytes: &'a [u8],
        fields: &'a [Field<PortableForm>],
        types: &'a PortableRegistry,
    ) -> Fields<'a> {
        Fields { len: fields.len(), bytes, fields, types }
    }
    pub (crate) fn bytes(&self) -> &'a [u8] {
        self.bytes
    }
    pub (crate) fn skip_rest(&mut self) -> Result<(), DecodeError> {
        while !self.fields.is_empty() {
            self.decode_item(IgnoreVisitor)?;
        }
        Ok(())
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn remaining(&self) -> usize {
        self.fields.len()
    }
    pub fn next_field_name(&self) -> Option<&str> {
        self.fields.get(0).and_then(|f| f.name().map(|n| &**n))
    }
    pub fn decode_item<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, V::Error> {
        if self.fields.is_empty() {
            return Err(DecodeError::NoItemsLeft.into())
        }

        let field = &self.fields[0];
        self.fields = &self.fields[1..];

        let b = &mut self.bytes;
        // Don't return here; decrement bytes properly first and then return, so that
        // calling decode_item again works as expected.
        let res = super::decode(b, field.ty().id(), self.types, visitor);
        self.bytes = *b;
        res
    }
}

/// This enables a visitor to decode items from array or sequence types.
pub struct Sequence<'a> {
    bytes: &'a [u8],
    type_id: u32,
    len: usize,
    types: &'a PortableRegistry,
    remaining: usize,
}

impl <'a> Sequence<'a> {
    pub (crate) fn new(
        bytes: &'a [u8],
        type_id: u32,
        len: usize,
        types: &'a PortableRegistry,
    ) -> Sequence<'a> {
        Sequence {
            bytes,
            type_id,
            len,
            types,
            remaining: len
        }
    }
    pub (crate) fn bytes(&self) -> &'a [u8] {
        self.bytes
    }
    pub (crate) fn skip_rest(&mut self) -> Result<(), DecodeError> {
        while self.remaining() > 0 {
            self.decode_item(IgnoreVisitor)?;
        }
        Ok(())
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn remaining(&self) -> usize {
        self.remaining
    }
    pub fn decode_item<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, V::Error> {
        if self.remaining == 0 {
            return Err(DecodeError::NoItemsLeft.into())
        }

        let b = &mut self.bytes;
        // Don't return here; decrement bytes and remaining properly first and then return, so that
        // calling decode_item again works as expected.
        let res = super::decode(b, self.type_id, self.types, visitor);
        self.bytes = *b;
        self.remaining -= 1;
        res
    }
}

/// An error decoding SCALE bytes into a [`Value`].
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum DecodeError {


    // /// The type we're trying to decode is supposed to be compact encoded, but that is not possible.
	// #[error("Could not decode compact encoded type into {0:?}")]
	// CannotDecodeCompactIntoType(Type),
	// /// We ran into an error trying to decode a bit sequence.
	// #[error("Cannot decode bit sequence: {0}")]
	// BitSequenceError(BitSequenceError),

	/// We could not convert the [`u32`] that we found into a valid [`char`].
	#[error("{0} is expected to be a valid char, but is not")]
	InvalidChar(u32),
	/// We expected more bytes to finish decoding, but could not find them.
	#[error("Ran out of data during decoding")]
	Eof,
	/// We found a variant that does not match with any in the type we're trying to decode from.
	#[error("Could not find variant with index {0} in {1:?}")]
	VariantNotFound(u8, scale_info::TypeDefVariant<PortableForm>),
	/// Some error emitted from a [`codec::Decode`] impl.
	#[error("{0}")]
	CodecError(#[from] codec::Error),
    /// We could not find the type given in the type registry provided.
	#[error("Cannot find type with ID {0}")]
	TypeIdNotFound(u32),
    /// You hit this is you try to decode a field from an
    #[error("No fields left to decode")]
    NoItemsLeft
}

/// A [`Visitor`] implementation that just ignores all of the bytes.
pub struct IgnoreVisitor;

impl Visitor for IgnoreVisitor {
    type Value = ();
    type Error = DecodeError;

    fn visit_bool(self, bool: bool) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_char(self, value: char) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_u8(self, value: u8) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_u16(self, value: u16) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_u32(self, value: u32) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_u64(self, value: u64) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_u128(self, value: u128) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_u256(self, value: &[u8]) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_i8(self, value: i8) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_i16(self, value: i16) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_i32(self, value: i32) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_i64(self, value: i64) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_i128(self, value: i128) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_i256(self, value: &[u8]) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_sequence(self, items: &mut Sequence<'_>) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_composite(self, items: &mut Fields<'_>) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
    fn visit_tuple(self, value: &mut Tuple<'_>) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
}