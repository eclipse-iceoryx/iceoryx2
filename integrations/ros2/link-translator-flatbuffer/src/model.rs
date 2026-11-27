// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

/// Describes the values that appear within a message.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Model {
    /// Values that appear once per message without any repitition.
    pub values: Vec<Kind>,
    /// Describes a recurring model of values within a message.
    pub repeated: Vec<Model>,
}

/// The position of a value in a [`Model`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueIndex(pub u16);

/// The position of a repeated model in a [`Model`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelIndex(pub u16);

/// The interface to retrieve the values described by the model.
pub trait Values {
    type Error;

    /// The string elements of a value in their order.
    type StringElementIter<'a>: ExactSizeIterator<Item = Result<&'a [u8], Self::Error>>
    where
        Self: 'a;

    /// The message elements of a value.
    type MessageElements<'a>: MessageElements<Error = Self::Error>
    where
        Self: 'a;

    /// Returns the scalar value at `index`, or an error when the source
    /// cannot provide it.
    fn scalar(&self, index: ValueIndex) -> Result<ScalarValue, Self::Error>;

    /// Returns the bytes of the string value at `index`, or an error when
    /// the source cannot provide it.
    fn string(&self, index: ValueIndex) -> Result<&[u8], Self::Error>;

    /// Returns the elements of the scalar sequence or scalar array value at
    /// `index`, or an error when the source cannot provide them.
    ///
    /// The source of an array may hold no elements.
    fn scalar_elements(&self, index: ValueIndex) -> Result<ScalarElements<'_>, Self::Error>;

    /// Returns the elements of the string sequence or string array value at
    /// `index`, or an error when the source cannot provide them.
    ///
    /// The source of an array may hold no elements.
    fn string_elements(
        &self,
        index: ValueIndex,
    ) -> Result<Self::StringElementIter<'_>, Self::Error>;

    /// Returns the elements of the message sequence or message array value at
    /// `index`, or an error when the source cannot provide them.
    ///
    /// The source of an array may hold no elements.
    fn message_elements(&self, index: ValueIndex)
    -> Result<Self::MessageElements<'_>, Self::Error>;
}

/// The interface to retrieve the message elements of a value.
pub trait MessageElements {
    type Error;
    type Element<'a>: Values<Error = Self::Error>
    where
        Self: 'a;

    fn number_of_elements(&self) -> usize;
    fn element(&self, position: usize) -> Self::Element<'_>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Scalar(ScalarType),
    String,
    ScalarSequence(ScalarType),
    ScalarArray {
        scalar_type: ScalarType,
        number_of_elements: u32,
    },
    StringSequence,
    StringArray {
        number_of_elements: u32,
    },
    MessageSequence(ModelIndex),
    MessageArray {
        model: ModelIndex,
        number_of_elements: u32,
    },
}

/// The order of the bytes of a scalar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    Little,
    Big,
}

impl ByteOrder {
    #[cfg(target_endian = "little")]
    pub const HOST: ByteOrder = ByteOrder::Little;
    #[cfg(target_endian = "big")]
    pub const HOST: ByteOrder = ByteOrder::Big;
}

/// The elements of a sequence or an array of scalars as they lie in their
/// source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScalarElements<'a> {
    pub bytes: &'a [u8],
    pub byte_order: ByteOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarType {
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScalarValue {
    Bool(bool),
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl ScalarType {
    pub fn number_of_bytes(self) -> usize {
        match self {
            ScalarType::Bool | ScalarType::U8 | ScalarType::I8 => 1,
            ScalarType::U16 | ScalarType::I16 => 2,
            ScalarType::U32 | ScalarType::I32 | ScalarType::F32 => 4,
            ScalarType::U64 | ScalarType::I64 | ScalarType::F64 => 8,
        }
    }
}

impl ScalarValue {
    /// The little-endian bytes of the value, followed by zeros.
    pub fn to_le_bytes(self) -> [u8; 8] {
        let mut bytes = [0; 8];
        match self {
            ScalarValue::Bool(value) => bytes[0] = u8::from(value),
            ScalarValue::U8(value) => bytes[0] = value,
            ScalarValue::I8(value) => bytes[..1].copy_from_slice(&value.to_le_bytes()),
            ScalarValue::U16(value) => bytes[..2].copy_from_slice(&value.to_le_bytes()),
            ScalarValue::I16(value) => bytes[..2].copy_from_slice(&value.to_le_bytes()),
            ScalarValue::U32(value) => bytes[..4].copy_from_slice(&value.to_le_bytes()),
            ScalarValue::I32(value) => bytes[..4].copy_from_slice(&value.to_le_bytes()),
            ScalarValue::U64(value) => bytes.copy_from_slice(&value.to_le_bytes()),
            ScalarValue::I64(value) => bytes.copy_from_slice(&value.to_le_bytes()),
            ScalarValue::F32(value) => bytes[..4].copy_from_slice(&value.to_le_bytes()),
            ScalarValue::F64(value) => bytes.copy_from_slice(&value.to_le_bytes()),
        }
        bytes
    }

    /// The value of the type `scalar_type` stored in `byte_order` at the start
    /// of `bytes`.
    pub fn from_bytes(scalar_type: ScalarType, bytes: &[u8], byte_order: ByteOrder) -> Self {
        match byte_order {
            ByteOrder::Little => Self::from_le_bytes(scalar_type, bytes),
            ByteOrder::Big => Self::from_be_bytes(scalar_type, bytes),
        }
    }

    /// The value of the type `scalar_type` stored little-endian at the start
    /// of `bytes`.
    pub fn from_le_bytes(scalar_type: ScalarType, bytes: &[u8]) -> Self {
        macro_rules! from_le_bytes {
            ($ty:ty) => {
                <$ty>::from_le_bytes(
                    bytes[..size_of::<$ty>()]
                        .try_into()
                        .expect("the slice is the size of the scalar"),
                )
            };
        }
        match scalar_type {
            ScalarType::Bool => ScalarValue::Bool(bytes[0] != 0),
            ScalarType::U8 => ScalarValue::U8(bytes[0]),
            ScalarType::I8 => ScalarValue::I8(bytes[0] as i8),
            ScalarType::U16 => ScalarValue::U16(from_le_bytes!(u16)),
            ScalarType::I16 => ScalarValue::I16(from_le_bytes!(i16)),
            ScalarType::U32 => ScalarValue::U32(from_le_bytes!(u32)),
            ScalarType::I32 => ScalarValue::I32(from_le_bytes!(i32)),
            ScalarType::U64 => ScalarValue::U64(from_le_bytes!(u64)),
            ScalarType::I64 => ScalarValue::I64(from_le_bytes!(i64)),
            ScalarType::F32 => ScalarValue::F32(from_le_bytes!(f32)),
            ScalarType::F64 => ScalarValue::F64(from_le_bytes!(f64)),
        }
    }
    /// The value of the type `scalar_type` stored big-endian at the start of
    /// `bytes`.
    pub fn from_be_bytes(scalar_type: ScalarType, bytes: &[u8]) -> Self {
        macro_rules! from_be_bytes {
            ($ty:ty) => {
                <$ty>::from_be_bytes(
                    bytes[..size_of::<$ty>()]
                        .try_into()
                        .expect("the slice is the size of the scalar"),
                )
            };
        }
        match scalar_type {
            ScalarType::Bool => ScalarValue::Bool(bytes[0] != 0),
            ScalarType::U8 => ScalarValue::U8(bytes[0]),
            ScalarType::I8 => ScalarValue::I8(bytes[0] as i8),
            ScalarType::U16 => ScalarValue::U16(from_be_bytes!(u16)),
            ScalarType::I16 => ScalarValue::I16(from_be_bytes!(i16)),
            ScalarType::U32 => ScalarValue::U32(from_be_bytes!(u32)),
            ScalarType::I32 => ScalarValue::I32(from_be_bytes!(i32)),
            ScalarType::U64 => ScalarValue::U64(from_be_bytes!(u64)),
            ScalarType::I64 => ScalarValue::I64(from_be_bytes!(i64)),
            ScalarType::F32 => ScalarValue::F32(from_be_bytes!(f32)),
            ScalarType::F64 => ScalarValue::F64(from_be_bytes!(f64)),
        }
    }
}
