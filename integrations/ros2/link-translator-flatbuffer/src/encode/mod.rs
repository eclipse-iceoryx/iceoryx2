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

//! The encoding of a flatbuffer into the CDR of its ROS 2 message.
//!
//! The flatbuffer is read through the table's offsets and is not verified.
//! An absent field encodes as its default, a vector of the wrong length
//! for a fixed array is refused.

use iceoryx2_integrations_ros2_link_adapter::cdr::{Writer, writer};
use iceoryx2_link_backend::wire::region::{ForwardRegion, ResizeError};
use iceoryx2_log::{fail, origin};

use crate::pairing::{
    ElementKind, FieldKind, Scalar, StructKind, StructLayout, Table, TableField, Value,
};

mod flatbuffer;

use flatbuffer::RawVector;

/// Why a flatbuffer does not encode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// The flatbuffer is shorter than its root offset.
    Truncated,
    /// The region cannot hold the CDR.
    Region(ResizeError),
    /// A string or sequence exceeds its length prefix.
    TooLong,
    /// A vector paired with a fixed array has the wrong length.
    ArrayLength { expected: usize, actual: usize },
}

impl core::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EncodeError::{self:?}")
    }
}

impl core::error::Error for EncodeError {}

impl From<writer::CreateError> for EncodeError {
    fn from(error: writer::CreateError) -> Self {
        match error {
            writer::CreateError::Region(error) => EncodeError::Region(error),
        }
    }
}

impl From<writer::ScalarError> for EncodeError {
    fn from(error: writer::ScalarError) -> Self {
        match error {
            writer::ScalarError::Region(error) => EncodeError::Region(error),
        }
    }
}

impl From<writer::LengthError> for EncodeError {
    fn from(error: writer::LengthError) -> Self {
        match error {
            writer::LengthError::Region(error) => EncodeError::Region(error),
            writer::LengthError::TooLong => EncodeError::TooLong,
        }
    }
}

impl From<writer::SequenceError> for EncodeError {
    fn from(error: writer::SequenceError) -> Self {
        match error {
            writer::SequenceError::Region(error) => EncodeError::Region(error),
            writer::SequenceError::TooLong => EncodeError::TooLong,
        }
    }
}

impl From<writer::StringError> for EncodeError {
    fn from(error: writer::StringError) -> Self {
        match error {
            writer::StringError::Region(error) => EncodeError::Region(error),
            writer::StringError::TooLong => EncodeError::TooLong,
        }
    }
}

/// Encodes the flatbuffer `flatbuffer` as the CDR of the message `table`
/// pairs it with into `cdr`.
///
/// Fails when the flatbuffer has no root, the region rejects the CDR or a
/// fixed array receives the wrong number of elements.
pub fn cdr<R: ForwardRegion>(table: &Table, flatbuffer: &[u8], cdr: R) -> Result<(), EncodeError> {
    let origin = origin!("encode::cdr");

    let Some(root) = flatbuffer::root(flatbuffer) else {
        fail!(
            from origin,
            with EncodeError::Truncated,
            "The flatbuffer of {} bytes holds no root offset", flatbuffer.len()
        );
    };
    let mut writer = Writer::new(cdr)?;
    write_table(&table.fields, root, &mut writer)
}

/// Writes the `fields` of `table`, each present field from the flatbuffer
/// and each absent one as its default.
fn write_table<R: ForwardRegion>(
    fields: &[TableField],
    table: flatbuffers::Table<'_>,
    writer: &mut Writer<R>,
) -> Result<(), EncodeError> {
    for field in fields {
        match flatbuffer::position(table, field.vtable_offset) {
            Some(position) => write_field(&field.kind, table.buf(), position, writer)?,
            None => write_absent(&field.kind, writer)?,
        }
    }
    Ok(())
}

fn write_field<R: ForwardRegion>(
    kind: &FieldKind,
    buf: &[u8],
    position: usize,
    writer: &mut Writer<R>,
) -> Result<(), EncodeError> {
    match kind {
        FieldKind::Scalar { scalar, .. } => {
            write_value(flatbuffer::scalar(buf, position, *scalar), writer)
        }
        FieldKind::String => Ok(writer.write_string(flatbuffer::string(buf, position))?),
        FieldKind::Table(nested) => {
            write_table(&nested.fields, flatbuffer::table(buf, position), writer)
        }
        FieldKind::Struct(layout) => {
            write_struct(layout, flatbuffer::struct_bytes(buf, position), writer)
        }
        FieldKind::Sequence(element) => {
            write_sequence(element, flatbuffer::vector(buf, position), writer)
        }
        FieldKind::Array { len, element } => {
            write_array(*len, element, flatbuffer::vector(buf, position), writer)
        }
    }
}

fn write_sequence<R: ForwardRegion>(
    element: &ElementKind,
    vector: RawVector<'_>,
    writer: &mut Writer<R>,
) -> Result<(), EncodeError> {
    match element {
        ElementKind::Scalar(scalar) => write_scalar_vector(*scalar, vector, true, writer),
        _ => {
            writer.write_length(vector.len)?;
            write_elements(element, vector, writer)
        }
    }
}

/// Writes `vector` as an array of `len`.
///
/// Fails when the vector does not hold `len` elements.
fn write_array<R: ForwardRegion>(
    len: usize,
    element: &ElementKind,
    vector: RawVector<'_>,
    writer: &mut Writer<R>,
) -> Result<(), EncodeError> {
    let origin = origin!("encode::write_array");

    if vector.len != len {
        fail!(
            from origin,
            with EncodeError::ArrayLength { expected: len, actual: vector.len },
            "An array of {} elements received a vector of {}", len, vector.len
        );
    }
    match element {
        ElementKind::Scalar(scalar) => write_scalar_vector(*scalar, vector, false, writer),
        _ => write_elements(element, vector, writer),
    }
}

fn write_elements<R: ForwardRegion>(
    element: &ElementKind,
    vector: RawVector<'_>,
    writer: &mut Writer<R>,
) -> Result<(), EncodeError> {
    match element {
        ElementKind::Scalar(_) => unreachable!("scalar vectors are written whole"),
        ElementKind::String => {
            for string in vector.strings() {
                writer.write_string(string)?;
            }
        }
        ElementKind::Table(nested) => {
            for table in vector.tables() {
                write_table(&nested.fields, table, writer)?;
            }
        }
        ElementKind::Struct(nested) => {
            let elements = vector.elements(nested.size);
            for position in (0..vector.len).map(|index| index * nested.size) {
                write_struct(nested, &elements[position..], writer)?;
            }
        }
    }
    Ok(())
}

/// Writes a vector of scalars, `prefixed` by its length for a sequence.
fn write_scalar_vector<R: ForwardRegion>(
    scalar: Scalar,
    vector: RawVector<'_>,
    prefixed: bool,
    writer: &mut Writer<R>,
) -> Result<(), EncodeError> {
    let elements = vector.elements(scalar.size());
    if cfg!(target_endian = "little") {
        if prefixed {
            writer.write_sequence(elements, scalar.size())?;
        } else {
            writer.write_array(elements, scalar.size())?;
        }
    } else {
        if prefixed {
            writer.write_length(vector.len)?;
        }
        for position in (0..vector.len).map(|index| index * scalar.size()) {
            write_value(Value::from_le_bytes(scalar, &elements[position..]), writer)?;
        }
    }
    Ok(())
}

fn write_struct<R: ForwardRegion>(
    layout: &StructLayout,
    bytes: &[u8],
    writer: &mut Writer<R>,
) -> Result<(), EncodeError> {
    for field in &layout.fields {
        let bytes = &bytes[field.offset as usize..];
        match &field.kind {
            StructKind::Scalar(scalar) => {
                write_value(Value::from_le_bytes(*scalar, bytes), writer)?
            }
            StructKind::Struct(nested) => write_struct(nested, bytes, writer)?,
        }
    }
    Ok(())
}

/// Writes the default of an absent field of `kind`.
///
/// Fails when the field is a fixed array of more than zero elements.
fn write_absent<R: ForwardRegion>(
    kind: &FieldKind,
    writer: &mut Writer<R>,
) -> Result<(), EncodeError> {
    let origin = origin!("encode::write_absent");

    match kind {
        FieldKind::Scalar { scalar, default } => {
            write_value(Value::default(*scalar, *default), writer)
        }
        FieldKind::String => Ok(writer.write_string(&[])?),
        FieldKind::Table(nested) => {
            for field in &nested.fields {
                write_absent(&field.kind, writer)?;
            }
            Ok(())
        }
        FieldKind::Struct(layout) => write_absent_struct(layout, writer),
        FieldKind::Sequence(_) => Ok(writer.write_length(0)?),
        FieldKind::Array { len: 0, .. } => Ok(()),
        FieldKind::Array { len, .. } => {
            fail!(
                from origin,
                with EncodeError::ArrayLength { expected: *len, actual: 0 },
                "An array of {} elements received no vector", len
            );
        }
    }
}

fn write_absent_struct<R: ForwardRegion>(
    layout: &StructLayout,
    writer: &mut Writer<R>,
) -> Result<(), EncodeError> {
    const ZEROS: [u8; 8] = [0; 8];
    for field in &layout.fields {
        match &field.kind {
            StructKind::Scalar(scalar) => {
                write_value(Value::from_le_bytes(*scalar, &ZEROS), writer)?
            }
            StructKind::Struct(nested) => write_absent_struct(nested, writer)?,
        }
    }
    Ok(())
}

fn write_value<R: ForwardRegion>(value: Value, writer: &mut Writer<R>) -> Result<(), EncodeError> {
    match value {
        Value::Bool(value) => writer.write_bool(value)?,
        Value::U8(value) => writer.write_u8(value)?,
        Value::I8(value) => writer.write_i8(value)?,
        Value::U16(value) => writer.write_u16(value)?,
        Value::I16(value) => writer.write_i16(value)?,
        Value::U32(value) => writer.write_u32(value)?,
        Value::I32(value) => writer.write_i32(value)?,
        Value::U64(value) => writer.write_u64(value)?,
        Value::I64(value) => writer.write_i64(value)?,
        Value::F32(value) => writer.write_f32(value)?,
        Value::F64(value) => writer.write_f64(value)?,
    }
    Ok(())
}
