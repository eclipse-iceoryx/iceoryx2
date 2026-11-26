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

use iceoryx2_log::{fail, origin};

use super::{CDR_BE, CDR_LE, ENCAPSULATION_LEN, IDENTIFIER_BYTES};

/// The byte order a message is encoded in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    Little,
    Big,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateError {
    /// The bytes end before the header does.
    Truncated,
    /// The header names an encapsulation other than plain CDR.
    UnsupportedEncapsulation,
}

impl core::fmt::Display for CreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreateError::{self:?}")
    }
}

impl core::error::Error for CreateError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarError {
    /// The bytes end before the value does.
    Truncated,
}

impl core::fmt::Display for ScalarError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ScalarError::{self:?}")
    }
}

impl core::error::Error for ScalarError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolError {
    /// The bytes end before the bool does.
    Truncated,
    /// A bool is neither 0 nor 1.
    InvalidBool,
}

impl core::fmt::Display for BoolError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BoolError::{self:?}")
    }
}

impl core::error::Error for BoolError {}

impl From<ScalarError> for BoolError {
    fn from(error: ScalarError) -> Self {
        match error {
            ScalarError::Truncated => Self::Truncated,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringError {
    /// The bytes end before the string does.
    Truncated,
    /// A string has no terminating NUL.
    InvalidString,
}

impl core::fmt::Display for StringError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "StringError::{self:?}")
    }
}

impl core::error::Error for StringError {}

impl From<ScalarError> for StringError {
    fn from(error: ScalarError) -> Self {
        match error {
            ScalarError::Truncated => Self::Truncated,
        }
    }
}

/// A saved location within the bytes of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location(usize);

/// An XCDR1 reader over the bytes of a message.
#[derive(Debug)]
pub struct Reader<'a> {
    bytes: &'a [u8],
    /// The offset of the next byte to read, counted from the start of the
    /// header.
    cursor: usize,
    endianness: Endianness,
}

impl<'a> Reader<'a> {
    /// A reader over the message `bytes`, in the byte order their
    /// encapsulation header names.
    ///
    /// Fails when the bytes hold no header or the header names an
    /// encapsulation other than plain CDR.
    pub fn new(bytes: &'a [u8]) -> Result<Self, CreateError> {
        let origin = origin!("cdr::Reader::new");

        let Some(header) = bytes.get(..ENCAPSULATION_LEN) else {
            fail!(
                from origin,
                with CreateError::Truncated,
                "The {} bytes do not hold the encapsulation header", bytes.len()
            );
        };
        let endianness = match &header[IDENTIFIER_BYTES] {
            identifier if identifier == CDR_LE => Endianness::Little,
            identifier if identifier == CDR_BE => Endianness::Big,
            other => {
                fail!(
                    from origin,
                    with CreateError::UnsupportedEncapsulation,
                    "The representation identifier {:#04x}{:02x} is not plain CDR", other[0], other[1]
                );
            }
        };

        Ok(Self {
            bytes,
            cursor: ENCAPSULATION_LEN,
            endianness,
        })
    }

    pub fn endianness(&self) -> Endianness {
        self.endianness
    }

    /// Whether every byte of the message has been read.
    pub fn is_exhausted(&self) -> bool {
        self.cursor == self.bytes.len()
    }

    /// Save the current location within the bytes of the message.
    pub fn location(&self) -> Location {
        Location(self.cursor)
    }

    /// Produces a reader at the provided location.
    pub fn at(&self, location: Location) -> Reader<'a> {
        Reader {
            bytes: self.bytes,
            cursor: location.0.min(self.bytes.len()),
            endianness: self.endianness,
        }
    }

    pub fn read_bool(&mut self) -> Result<bool, BoolError> {
        let origin = origin!("cdr::Reader::read_bool");

        let byte = fail!(
            from origin,
            when self.read_u8(),
            to BoolError,
            "Unable to read the byte of a bool"
        );
        match byte {
            0 => Ok(false),
            1 => Ok(true),
            other => {
                fail!(
                    from origin,
                    with BoolError::InvalidBool,
                    "The byte {} is not a bool", other
                );
            }
        }
    }

    pub fn read_u8(&mut self) -> Result<u8, ScalarError> {
        let origin = origin!("cdr::Reader::read_u8");

        let bytes = fail!(
            from origin,
            when self.read_scalar::<{ size_of::<u8>() }>(),
            "Unable to read a u8"
        );
        Ok(match self.endianness {
            Endianness::Little => u8::from_le_bytes(bytes),
            Endianness::Big => u8::from_be_bytes(bytes),
        })
    }

    pub fn read_i8(&mut self) -> Result<i8, ScalarError> {
        let origin = origin!("cdr::Reader::read_i8");

        let bytes = fail!(
            from origin,
            when self.read_scalar::<{ size_of::<i8>() }>(),
            "Unable to read a i8"
        );
        Ok(match self.endianness {
            Endianness::Little => i8::from_le_bytes(bytes),
            Endianness::Big => i8::from_be_bytes(bytes),
        })
    }

    pub fn read_u16(&mut self) -> Result<u16, ScalarError> {
        let origin = origin!("cdr::Reader::read_u16");

        let bytes = fail!(
            from origin,
            when self.read_scalar::<{ size_of::<u16>() }>(),
            "Unable to read a u16"
        );
        Ok(match self.endianness {
            Endianness::Little => u16::from_le_bytes(bytes),
            Endianness::Big => u16::from_be_bytes(bytes),
        })
    }

    pub fn read_i16(&mut self) -> Result<i16, ScalarError> {
        let origin = origin!("cdr::Reader::read_i16");

        let bytes = fail!(
            from origin,
            when self.read_scalar::<{ size_of::<i16>() }>(),
            "Unable to read a i16"
        );
        Ok(match self.endianness {
            Endianness::Little => i16::from_le_bytes(bytes),
            Endianness::Big => i16::from_be_bytes(bytes),
        })
    }

    pub fn read_u32(&mut self) -> Result<u32, ScalarError> {
        let origin = origin!("cdr::Reader::read_u32");

        let bytes = fail!(
            from origin,
            when self.read_scalar::<{ size_of::<u32>() }>(),
            "Unable to read a u32"
        );
        Ok(match self.endianness {
            Endianness::Little => u32::from_le_bytes(bytes),
            Endianness::Big => u32::from_be_bytes(bytes),
        })
    }

    pub fn read_i32(&mut self) -> Result<i32, ScalarError> {
        let origin = origin!("cdr::Reader::read_i32");

        let bytes = fail!(
            from origin,
            when self.read_scalar::<{ size_of::<i32>() }>(),
            "Unable to read a i32"
        );
        Ok(match self.endianness {
            Endianness::Little => i32::from_le_bytes(bytes),
            Endianness::Big => i32::from_be_bytes(bytes),
        })
    }

    pub fn read_u64(&mut self) -> Result<u64, ScalarError> {
        let origin = origin!("cdr::Reader::read_u64");

        let bytes = fail!(
            from origin,
            when self.read_scalar::<{ size_of::<u64>() }>(),
            "Unable to read a u64"
        );
        Ok(match self.endianness {
            Endianness::Little => u64::from_le_bytes(bytes),
            Endianness::Big => u64::from_be_bytes(bytes),
        })
    }

    pub fn read_i64(&mut self) -> Result<i64, ScalarError> {
        let origin = origin!("cdr::Reader::read_i64");

        let bytes = fail!(
            from origin,
            when self.read_scalar::<{ size_of::<i64>() }>(),
            "Unable to read a i64"
        );
        Ok(match self.endianness {
            Endianness::Little => i64::from_le_bytes(bytes),
            Endianness::Big => i64::from_be_bytes(bytes),
        })
    }

    pub fn read_f32(&mut self) -> Result<f32, ScalarError> {
        let origin = origin!("cdr::Reader::read_f32");

        let bytes = fail!(
            from origin,
            when self.read_scalar::<{ size_of::<f32>() }>(),
            "Unable to read a f32"
        );
        Ok(match self.endianness {
            Endianness::Little => f32::from_le_bytes(bytes),
            Endianness::Big => f32::from_be_bytes(bytes),
        })
    }

    pub fn read_f64(&mut self) -> Result<f64, ScalarError> {
        let origin = origin!("cdr::Reader::read_f64");

        let bytes = fail!(
            from origin,
            when self.read_scalar::<{ size_of::<f64>() }>(),
            "Unable to read a f64"
        );
        Ok(match self.endianness {
            Endianness::Little => f64::from_le_bytes(bytes),
            Endianness::Big => f64::from_be_bytes(bytes),
        })
    }

    /// Reads the length prefix of a sequence.
    pub fn read_length(&mut self) -> Result<usize, ScalarError> {
        let origin = origin!("cdr::Reader::read_length");

        let number_of_elements = fail!(
            from origin,
            when self.read_u32(),
            "Unable to read a length prefix"
        );

        Ok(number_of_elements as usize)
    }

    /// Reads a string, returning its bytes without the terminating NUL.
    ///
    /// Fails when the string has no length or no NUL.
    pub fn read_string(&mut self) -> Result<&'a [u8], StringError> {
        let origin = origin!("cdr::Reader::read_string");

        let number_of_bytes = fail!(
            from origin,
            when self.read_length(),
            to StringError,
            "Unable to read the length of a string"
        );
        let bytes = fail!(
            from origin,
            when self.read_bytes(number_of_bytes),
            to StringError,
            "Unable to read the {} bytes of a string", number_of_bytes
        );
        let Some((0, string)) = bytes.split_last() else {
            fail!(
                from origin,
                with StringError::InvalidString,
                "A string of {} bytes is not NUL terminated", number_of_bytes
            );
        };
        Ok(string)
    }

    /// Reads a sequence of primitives of `size` bytes each, returning its
    /// elements as they lie in the message, in the message's byte order.
    ///
    /// Fails when the message ends before the sequence does.
    pub fn read_sequence(&mut self, size: usize) -> Result<&'a [u8], ScalarError> {
        let origin = origin!("cdr::Reader::read_sequence");

        let number_of_elements = fail!(
            from origin,
            when self.read_length(),
            "Unable to read the length of a sequence"
        );
        self.read_array(number_of_elements, size)
    }

    /// Reads the elements of an array of `number_of_elements` primitives of
    /// `size` bytes each as they lie in the message, in the message's byte
    /// order. An array has no length prefix.
    ///
    /// Fails when the message ends before the array does.
    pub fn read_array(
        &mut self,
        number_of_elements: usize,
        size: usize,
    ) -> Result<&'a [u8], ScalarError> {
        let origin = origin!("cdr::Reader::read_array");

        if number_of_elements == 0 {
            return Ok(&[]);
        }
        fail!(
            from origin,
            when self.align(size),
            "Unable to skip the padding of an array of elements of {} bytes", size
        );
        let Some(number_of_bytes) = number_of_elements.checked_mul(size) else {
            fail!(
                from origin,
                with ScalarError::Truncated,
                "The message cannot hold {} elements of {} bytes", number_of_elements, size
            );
        };
        self.read_bytes(number_of_bytes)
    }

    /// Reads a primitive aligned to its size.
    fn read_scalar<const N: usize>(&mut self) -> Result<[u8; N], ScalarError> {
        let origin = origin!("cdr::Reader::read_scalar");

        fail!(
            from origin,
            when self.align(N),
            "Unable to skip the padding of a primitive of {} bytes", N
        );
        let bytes = fail!(
            from origin,
            when self.read_bytes(N),
            "Unable to read a primitive of {} bytes", N
        );

        Ok(bytes.try_into().expect("the slice is N bytes long"))
    }

    /// Skips the padding up to a multiple of `alignment` after the header.
    fn align(&mut self, alignment: usize) -> Result<(), ScalarError> {
        let read = self.cursor - ENCAPSULATION_LEN;
        let padding = read.next_multiple_of(alignment) - read;
        self.advance(padding)
    }

    /// The next `number_of_bytes` of the message.
    ///
    /// Fails when fewer remain.
    fn read_bytes(&mut self, number_of_bytes: usize) -> Result<&'a [u8], ScalarError> {
        let origin = origin!("cdr::Reader::read_bytes");

        let start = self.cursor;
        fail!(
            from origin,
            when self.advance(number_of_bytes),
            "Unable to read {} bytes", number_of_bytes
        );

        Ok(&self.bytes[start..self.cursor])
    }

    /// Moves the cursor over the next `number_of_bytes` of the message.
    ///
    /// Fails when fewer remain.
    fn advance(&mut self, number_of_bytes: usize) -> Result<(), ScalarError> {
        let origin = origin!("cdr::Reader::advance");

        let Some(end) = self
            .cursor
            .checked_add(number_of_bytes)
            .filter(|end| *end <= self.bytes.len())
        else {
            fail!(
                from origin,
                with ScalarError::Truncated,
                "The message ends {} bytes before the value does",
                number_of_bytes.saturating_sub(self.bytes.len() - self.cursor)
            );
        };
        self.cursor = end;

        Ok(())
    }
}
