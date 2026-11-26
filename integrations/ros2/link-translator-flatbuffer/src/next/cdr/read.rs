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

use iceoryx2_integrations_ros2_link_adapter::cdr::reader::{self, Location};
use iceoryx2_log::{fail, origin};

use super::layout::{Layout, MemberKind};
use crate::next::model::{ScalarType, ScalarValue, ValueIndex, Values};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadError {
    /// The CDR ends before the message does.
    Truncated,
    /// The CDR names an encapsulation other than plain CDR.
    UnsupportedEncapsulation,
    /// A bool is neither 0 nor 1.
    InvalidBool,
    /// A string has no terminating NUL.
    InvalidString,
}

impl core::fmt::Display for ReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReadError::{self:?}")
    }
}

impl core::error::Error for ReadError {}

impl From<reader::CreateError> for ReadError {
    fn from(error: reader::CreateError) -> Self {
        match error {
            reader::CreateError::Truncated => ReadError::Truncated,
            reader::CreateError::UnsupportedEncapsulation => ReadError::UnsupportedEncapsulation,
        }
    }
}

impl From<reader::ScalarError> for ReadError {
    fn from(error: reader::ScalarError) -> Self {
        match error {
            reader::ScalarError::Truncated => ReadError::Truncated,
        }
    }
}

impl From<reader::BoolError> for ReadError {
    fn from(error: reader::BoolError) -> Self {
        match error {
            reader::BoolError::Truncated => ReadError::Truncated,
            reader::BoolError::InvalidBool => ReadError::InvalidBool,
        }
    }
}

impl From<reader::StringError> for ReadError {
    fn from(error: reader::StringError) -> Self {
        match error {
            reader::StringError::Truncated => ReadError::Truncated,
            reader::StringError::InvalidString => ReadError::InvalidString,
        }
    }
}

#[derive(Debug)]
pub struct Locations<'a> {
    layout: &'a Layout,
    reader: reader::Reader<'a>,
    /// By value index.
    locations: Vec<Location>,
}

impl Locations<'_> {
    fn location(&self, index: ValueIndex) -> Location {
        self.locations[usize::from(index.0)]
    }
}

impl Values for Locations<'_> {
    type Error = ReadError;

    fn scalar(&self, index: ValueIndex) -> Result<ScalarValue, ReadError> {
        let MemberKind::Scalar(scalar) = self.layout.member(index).kind else {
            unreachable!("the value is a scalar in the model")
        };

        // Read the member from its saved location.
        read_scalar(&mut self.reader.at(self.location(index)), scalar)
    }

    fn string(&self, index: ValueIndex) -> Result<&[u8], ReadError> {
        let MemberKind::String = self.layout.member(index).kind else {
            unreachable!("the value is a string in the model")
        };

        // Read the member from its saved location.
        read_string(&mut self.reader.at(self.location(index)))
    }
}

/// Locates the values `layout` places in `cdr`.
///
/// Fails when the bytes do not hold a message of the layout.
pub fn locate<'a>(layout: &'a Layout, cdr: &'a [u8]) -> Result<Locations<'a>, ReadError> {
    let origin = origin!("cdr::locate");

    let mut reader = fail!(
        from origin,
        when reader::Reader::new(cdr),
        to ReadError,
        "Unable to read the header of the CDR of {} bytes", cdr.len()
    );

    let locations = fail!(
        from origin,
        when locate_message(layout, &mut reader),
        "Unable to locate the values in the CDR of {} bytes", cdr.len()
    );

    Ok(locations)
}

fn locate_message<'a>(
    layout: &'a Layout,
    reader: &mut reader::Reader<'a>,
) -> Result<Locations<'a>, ReadError> {
    let mut locations = vec![reader.location(); layout.members.len()];
    walk_message(layout, reader, &mut |index, location| {
        locations[usize::from(index.0)] = location;
    })?;

    Ok(Locations {
        layout,
        reader: reader.at(reader.location()),
        locations,
    })
}

/// Walks the members of one message of `layout` from where `reader` is.
///
/// `visit` receives the location of each member before the member is read.
/// The reader ends at the first byte after the message. Fails when the bytes
/// do not hold a message of the layout.
fn walk_message(
    layout: &Layout,
    reader: &mut reader::Reader<'_>,
    visit: &mut dyn FnMut(ValueIndex, Location),
) -> Result<(), ReadError> {
    let origin = origin!("cdr::walk_message");

    // Go through the members in CDR order and report where each one starts.
    for index in &layout.order {
        visit(*index, reader.location());

        // Read the member to move the reader to the start of the next one.
        // The value itself is dropped.
        let read = match layout.member(*index).kind {
            MemberKind::Scalar(scalar) => read_scalar(reader, scalar).map(|_| ()),
            MemberKind::String => read_string(reader).map(|_| ()),
        };
        fail!(
            from origin,
            when read,
            "Unable to read the value {}", index.0
        );
    }

    Ok(())
}

fn read_scalar(
    reader: &mut reader::Reader<'_>,
    scalar: ScalarType,
) -> Result<ScalarValue, ReadError> {
    Ok(match scalar {
        ScalarType::Bool => ScalarValue::Bool(reader.read_bool()?),
        ScalarType::U8 => ScalarValue::U8(reader.read_u8()?),
        ScalarType::I8 => ScalarValue::I8(reader.read_i8()?),
        ScalarType::U16 => ScalarValue::U16(reader.read_u16()?),
        ScalarType::I16 => ScalarValue::I16(reader.read_i16()?),
        ScalarType::U32 => ScalarValue::U32(reader.read_u32()?),
        ScalarType::I32 => ScalarValue::I32(reader.read_i32()?),
        ScalarType::U64 => ScalarValue::U64(reader.read_u64()?),
        ScalarType::I64 => ScalarValue::I64(reader.read_i64()?),
        ScalarType::F32 => ScalarValue::F32(reader.read_f32()?),
        ScalarType::F64 => ScalarValue::F64(reader.read_f64()?),
    })
}

fn read_string<'a>(reader: &mut reader::Reader<'a>) -> Result<&'a [u8], ReadError> {
    Ok(reader.read_string()?)
}
