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
    pub values: Vec<Kind>,
}

/// An index to a specific value description within the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueIndex(pub u16);

/// The interface to retrieve the values described by the model.
pub trait Values {
    type Error;

    /// Returns the scalar value at `index`, or an error when the source
    /// cannot provide it.
    fn scalar(&self, index: ValueIndex) -> Result<ScalarValue, Self::Error>;

    /// Returns the bytes of the string value at `index`, or an error when
    /// the source cannot provide it.
    fn string(&self, index: ValueIndex) -> Result<&[u8], Self::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Scalar(ScalarType),
    String,
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

impl ScalarValue {
    /// The value of the type `scalar` stored little-endian at the start of `bytes`.
    pub fn from_le_bytes(scalar: ScalarType, bytes: &[u8]) -> Self {
        macro_rules! from_le_bytes {
            ($ty:ty) => {
                <$ty>::from_le_bytes(
                    bytes[..size_of::<$ty>()]
                        .try_into()
                        .expect("the slice is the size of the scalar"),
                )
            };
        }
        match scalar {
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
}
