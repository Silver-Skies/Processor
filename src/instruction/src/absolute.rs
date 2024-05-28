//! Unsized absolute number. 
//! While Rust has u8, u16... for absolute values, it does not have a simple enum for variable length
//! absolute integers. 

// TODO: Use TryFrom<u8..u16..u32..u64> on Data and use a from_bytes function on Type

// Constants

pub const BYTE: u8 = 1;
pub const WORD: u8 = 2;
pub const DUAL: u8 = 4;
pub const QUAD: u8 = 8;

// Implementations

/// Absolute modes.
/// Base type variants for representing an absolute value.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum Type {
	#[default]
	Byte,
	Word,
	Dual,
	Quad
}

impl From<Data> for Type {
	fn from(value: Data) -> Self {
		match value {
			Data::Byte(_) => Self::Byte,
			Data::Word(_) => Self::Word,
			Data::Dual(_) => Self::Dual,
			Data::Quad(_) => Self::Quad
		}
	}
}

/// Number of bytes as a Rust numeric.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberBytes(pub u8);
/// Error thrown when the number of bytes is beyond the supported exponent 2^0..3 range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeError {}

impl TryFrom<NumberBytes> for Type {
	type Error = RangeError;

	fn try_from(value: NumberBytes) -> Result<Self, Self::Error> {
		match value.0 {
			BYTE => Ok(Type::Byte),
			WORD => Ok(Type::Word),
			DUAL => Ok(Type::Dual),
			QUAD => Ok(Type::Quad),
			_ => Err(RangeError {})
		}
	}
}

/// Variable absolute data type.
/// Complete variants that annotate numbers with their type in the same enum allowing for the data type to be changed
/// during runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Data {
	Byte(u8),
	Word(u16),
	Dual(u32),
	Quad(u64)
}

impl From<Type> for Data {
	fn from(value: Type) -> Self {
		match value {
			Type::Byte => Self::Byte(0),
			Type::Word => Self::Word(0),
			Type::Dual => Self::Dual(0),
			Type::Quad => Self::Quad(0)
		}
	}
}