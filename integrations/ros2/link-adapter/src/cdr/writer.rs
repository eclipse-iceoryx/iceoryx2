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

use iceoryx2_link_backend::wire::region::{ForwardRegion, ResizeError};
use iceoryx2_log::{fail, origin};

use super::{CDR_NE, ENCAPSULATION_LEN, IDENTIFIER_BYTES, OPTIONS_BYTES};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateError {
    /// The region cannot hold the header.
    Region(ResizeError),
}

impl core::fmt::Display for CreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreateError::{self:?}")
    }
}

impl core::error::Error for CreateError {}

impl From<ResizeError> for CreateError {
    fn from(error: ResizeError) -> Self {
        Self::Region(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarError {
    /// The region cannot hold the bytes.
    Region(ResizeError),
}

impl core::fmt::Display for ScalarError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ScalarError::{self:?}")
    }
}

impl core::error::Error for ScalarError {}

impl From<ResizeError> for ScalarError {
    fn from(error: ResizeError) -> Self {
        Self::Region(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthError {
    /// The region cannot hold the bytes.
    Region(ResizeError),
    /// The length exceeds what a length prefix holds.
    TooLong,
}

impl core::fmt::Display for LengthError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "LengthError::{self:?}")
    }
}

impl core::error::Error for LengthError {}

impl From<ScalarError> for LengthError {
    fn from(error: ScalarError) -> Self {
        match error {
            ScalarError::Region(error) => Self::Region(error),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceError {
    /// The region cannot hold the bytes.
    Region(ResizeError),
    /// The sequence has more elements than a length prefix holds.
    TooLong,
}

impl core::fmt::Display for SequenceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SequenceError::{self:?}")
    }
}

impl core::error::Error for SequenceError {}

impl From<ScalarError> for SequenceError {
    fn from(error: ScalarError) -> Self {
        match error {
            ScalarError::Region(error) => Self::Region(error),
        }
    }
}

impl From<LengthError> for SequenceError {
    fn from(error: LengthError) -> Self {
        match error {
            LengthError::Region(error) => Self::Region(error),
            LengthError::TooLong => Self::TooLong,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringError {
    /// The region cannot hold the bytes.
    Region(ResizeError),
    /// The string has more bytes than a length prefix holds.
    TooLong,
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
            ScalarError::Region(error) => Self::Region(error),
        }
    }
}

impl From<LengthError> for StringError {
    fn from(error: LengthError) -> Self {
        match error {
            LengthError::Region(error) => Self::Region(error),
            LengthError::TooLong => Self::TooLong,
        }
    }
}

/// An XCDR1 writer in the host's byte order over a forward region.
#[derive(Debug)]
pub struct Writer<R: ForwardRegion> {
    region: R,
    /// The offset of the next byte to write, counted from the start of the
    /// header.
    cursor: usize,
}

impl<R: ForwardRegion> Writer<R> {
    /// A writer over `region`, with the encapsulation header of the host's
    /// byte order written.
    ///
    /// Fails when the region cannot hold the header.
    pub fn new(mut region: R) -> Result<Self, CreateError> {
        let origin = origin!("cdr::Writer::new");

        fail!(
            from origin,
            when region.resize(ENCAPSULATION_LEN),
            to CreateError,
            "The region rejected the {} bytes of the encapsulation header", ENCAPSULATION_LEN
        );
        write_encapsulation_header(&mut region[..ENCAPSULATION_LEN]);

        Ok(Self {
            region,
            cursor: ENCAPSULATION_LEN,
        })
    }

    /// The bytes written so far, the header included.
    pub fn number_of_bytes(&self) -> usize {
        self.cursor
    }

    pub fn write_bool(&mut self, value: bool) -> Result<(), ScalarError> {
        self.write_scalar([value as u8])
    }

    pub fn write_u8(&mut self, value: u8) -> Result<(), ScalarError> {
        self.write_scalar(value.to_ne_bytes())
    }

    pub fn write_i8(&mut self, value: i8) -> Result<(), ScalarError> {
        self.write_scalar(value.to_ne_bytes())
    }

    pub fn write_u16(&mut self, value: u16) -> Result<(), ScalarError> {
        self.write_scalar(value.to_ne_bytes())
    }

    pub fn write_i16(&mut self, value: i16) -> Result<(), ScalarError> {
        self.write_scalar(value.to_ne_bytes())
    }

    pub fn write_u32(&mut self, value: u32) -> Result<(), ScalarError> {
        self.write_scalar(value.to_ne_bytes())
    }

    pub fn write_i32(&mut self, value: i32) -> Result<(), ScalarError> {
        self.write_scalar(value.to_ne_bytes())
    }

    pub fn write_u64(&mut self, value: u64) -> Result<(), ScalarError> {
        self.write_scalar(value.to_ne_bytes())
    }

    pub fn write_i64(&mut self, value: i64) -> Result<(), ScalarError> {
        self.write_scalar(value.to_ne_bytes())
    }

    pub fn write_f32(&mut self, value: f32) -> Result<(), ScalarError> {
        self.write_scalar(value.to_ne_bytes())
    }

    pub fn write_f64(&mut self, value: f64) -> Result<(), ScalarError> {
        self.write_scalar(value.to_ne_bytes())
    }

    /// Writes the length prefix of a sequence.
    ///
    /// Fails when `number_of_elements` exceeds what the prefix holds.
    pub fn write_length(&mut self, number_of_elements: usize) -> Result<(), LengthError> {
        let origin = origin!("cdr::Writer::write_length");

        let number_of_elements = fail!(
            from origin,
            when u32::try_from(number_of_elements),
            with LengthError::TooLong,
            "A length of {} exceeds the length prefix", number_of_elements
        );
        fail!(
            from origin,
            when self.write_u32(number_of_elements),
            to LengthError,
            "Unable to write the length prefix"
        );

        Ok(())
    }

    /// Writes a string as its length prefix, `bytes` and a NUL.
    ///
    /// Fails when the string exceeds what the prefix holds.
    pub fn write_string(&mut self, bytes: &[u8]) -> Result<(), StringError> {
        let origin = origin!("cdr::Writer::write_string");

        let Some(number_of_bytes) = bytes.len().checked_add(1) else {
            fail!(
                from origin,
                with StringError::TooLong,
                "A string of {} bytes exceeds the length prefix", bytes.len()
            );
        };
        fail!(
            from origin,
            when self.write_length(number_of_bytes),
            to StringError,
            "Unable to write the length of a string"
        );
        let start = fail!(
            from origin,
            when self.extend(number_of_bytes),
            to StringError,
            "Unable to extend the region by the {} bytes of a string", number_of_bytes
        );
        self.region[start..start + bytes.len()].copy_from_slice(bytes);
        self.region[start + bytes.len()] = 0;

        Ok(())
    }

    /// Writes a sequence of primitives of `size` bytes each as its length
    /// prefix and `elements` in the host's byte order, aligned to `size`.
    ///
    /// Fails when the sequence exceeds what the prefix holds.
    pub fn write_sequence(&mut self, elements: &[u8], size: usize) -> Result<(), SequenceError> {
        let origin = origin!("cdr::Writer::write_sequence");

        debug_assert!(
            elements.len().is_multiple_of(size),
            "the elements must be whole primitives"
        );
        fail!(
            from origin,
            when self.write_length(elements.len() / size),
            to SequenceError,
            "Unable to write the length of a sequence"
        );
        fail!(
            from origin,
            when self.write_array(elements, size),
            to SequenceError,
            "Unable to write the elements of a sequence"
        );

        Ok(())
    }

    /// Writes the `elements` of an array of primitives of `size` bytes each
    /// in the host's byte order, aligned to `size`. An array has no length
    /// prefix.
    ///
    /// Fails when the region cannot hold the elements.
    pub fn write_array(&mut self, elements: &[u8], size: usize) -> Result<(), ScalarError> {
        let origin = origin!("cdr::Writer::write_array");

        debug_assert!(
            elements.len().is_multiple_of(size),
            "the elements must be whole primitives"
        );
        if elements.is_empty() {
            return Ok(());
        }
        fail!(
            from origin,
            when self.align(size),
            "Unable to pad to the alignment of an array of elements of {} bytes", size
        );
        let start = fail!(
            from origin,
            when self.extend(elements.len()),
            "Unable to extend the region by the {} bytes of an array", elements.len()
        );
        self.region[start..start + elements.len()].copy_from_slice(elements);

        Ok(())
    }

    /// Writes a primitive aligned to its size.
    fn write_scalar<const N: usize>(&mut self, bytes: [u8; N]) -> Result<(), ScalarError> {
        let origin = origin!("cdr::Writer::write_scalar");

        fail!(
            from origin,
            when self.align(N),
            "Unable to pad to the alignment of a primitive of {} bytes", N
        );
        let start = fail!(
            from origin,
            when self.extend(N),
            "Unable to extend the region by a primitive of {} bytes", N
        );
        self.region[start..start + N].copy_from_slice(&bytes);

        Ok(())
    }

    /// Pads with zeros up to a multiple of `alignment` after the header.
    fn align(&mut self, alignment: usize) -> Result<(), ScalarError> {
        let origin = origin!("cdr::Writer::align");

        let written = self.cursor - ENCAPSULATION_LEN;
        let padding = written.next_multiple_of(alignment) - written;
        if padding == 0 {
            return Ok(());
        }
        let start = fail!(
            from origin,
            when self.extend(padding),
            "Unable to extend the region by {} bytes of padding", padding
        );
        self.region[start..start + padding].fill(0);

        Ok(())
    }

    /// Extends the region by `n` bytes, returning the offset they start at.
    fn extend(&mut self, n: usize) -> Result<usize, ScalarError> {
        let origin = origin!("cdr::Writer::extend");

        let start = self.cursor;
        let number_of_bytes = start + n;
        fail!(
            from origin,
            when self.region.resize(number_of_bytes),
            to ScalarError,
            "The region rejected growing to {} bytes", number_of_bytes
        );
        self.cursor = number_of_bytes;

        Ok(start)
    }
}

/// Writes the encapsulation header of the host's byte order into `header`.
fn write_encapsulation_header(header: &mut [u8]) {
    header[IDENTIFIER_BYTES].copy_from_slice(&CDR_NE);
    header[OPTIONS_BYTES].fill(0);
}
