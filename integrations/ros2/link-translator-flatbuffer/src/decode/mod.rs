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

//! The decoding of the CDR of a ROS 2 message into a flatbuffer.
//!
//! The CDR is read top-down in member order and the flatbuffer is built
//! bottom-up, so the members of a table are read and held pending until
//! the table is started. A scalar at its default is omitted.

use flatbuffers::{
    Allocator, FlatBufferBuilder, TableFinishedWIPOffset, UnionWIPOffset, WIPOffset,
};
use iceoryx2_integrations_ros2_link_adapter::cdr::reader::{self, Reader};

use crate::pairing::{
    ElementKind, FieldKind, Scalar, StructKind, StructLayout, Table, TableField, Value,
};

mod builder;

use builder::{push_scalar_vector, push_struct, push_struct_vector, push_value};

/// Why a CDR message does not decode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// The CDR ends before the message does.
    Truncated,
    /// The CDR names an encapsulation other than plain CDR.
    UnsupportedEncapsulation,
    /// A bool is neither 0 nor 1.
    InvalidBool,
    /// A string has no terminating NUL.
    InvalidString,
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DecodeError::{self:?}")
    }
}

impl core::error::Error for DecodeError {}

impl From<reader::CreateError> for DecodeError {
    fn from(error: reader::CreateError) -> Self {
        match error {
            reader::CreateError::Truncated => DecodeError::Truncated,
            reader::CreateError::UnsupportedEncapsulation => DecodeError::UnsupportedEncapsulation,
        }
    }
}

impl From<reader::ScalarError> for DecodeError {
    fn from(error: reader::ScalarError) -> Self {
        match error {
            reader::ScalarError::Truncated => DecodeError::Truncated,
        }
    }
}

impl From<reader::BoolError> for DecodeError {
    fn from(error: reader::BoolError) -> Self {
        match error {
            reader::BoolError::Truncated => DecodeError::Truncated,
            reader::BoolError::InvalidBool => DecodeError::InvalidBool,
        }
    }
}

impl From<reader::StringError> for DecodeError {
    fn from(error: reader::StringError) -> Self {
        match error {
            reader::StringError::Truncated => DecodeError::Truncated,
            reader::StringError::InvalidString => DecodeError::InvalidString,
        }
    }
}

/// Decodes the CDR `cdr` of the message `table` pairs with into the
/// a flatbuffer built by the provided `builder`.
///
/// Fails when the CDR cannot be read.
pub fn flatbuffer<A: Allocator>(
    table: &Table,
    cdr: &[u8],
    builder: &mut FlatBufferBuilder<'_, A>,
) -> Result<(), DecodeError> {
    let mut reader = Reader::new(cdr)?;
    let root = read_table(&table.fields, &mut reader, builder)?;
    builder.finish(root, None);

    Ok(())
}

/// A field read from the CDR and pending its push into the builder.
enum Pending {
    Scalar {
        value: Value,
        default: Value,
    },
    Offset(WIPOffset<UnionWIPOffset>),
    /// The little-endian image of a struct.
    Struct {
        image: Vec<u8>,
        alignment: usize,
    },
}

fn read_table<A: Allocator>(
    fields: &[TableField],
    reader: &mut Reader<'_>,
    builder: &mut FlatBufferBuilder<'_, A>,
) -> Result<WIPOffset<TableFinishedWIPOffset>, DecodeError> {
    let mut pending = Vec::with_capacity(fields.len());
    for field in fields {
        let member = match &field.kind {
            FieldKind::Scalar { scalar, default } => Pending::Scalar {
                value: read_value(*scalar, reader)?,
                default: Value::default(*scalar, *default),
            },
            FieldKind::String => Pending::Offset(read_string(reader, builder)?),
            FieldKind::Table(nested) => {
                Pending::Offset(read_table(&nested.fields, reader, builder)?.as_union_value())
            }
            FieldKind::Struct(nested) => Pending::Struct {
                image: read_struct(nested, reader)?,
                alignment: nested.alignment,
            },
            FieldKind::Sequence(element) => {
                Pending::Offset(read_sequence(element, reader, builder)?)
            }
            FieldKind::Array { len, element } => {
                Pending::Offset(read_array(*len, element, reader, builder)?)
            }
        };
        pending.push(member);
    }

    let table = builder.start_table();
    for (field, member) in fields.iter().zip(pending) {
        match member {
            Pending::Scalar { value, default } => {
                if value != default {
                    push_value(builder, field.vtable_offset, value)
                }
            }
            Pending::Offset(offset) => builder.push_slot_always(field.vtable_offset, offset),
            Pending::Struct { image, alignment } => {
                push_struct(builder, field.vtable_offset, &image, alignment)
            }
        }
    }
    Ok(builder.end_table(table))
}

fn read_value(scalar: Scalar, reader: &mut Reader<'_>) -> Result<Value, DecodeError> {
    Ok(match scalar {
        Scalar::Bool => Value::Bool(reader.read_bool()?),
        Scalar::U8 => Value::U8(reader.read_u8()?),
        Scalar::I8 => Value::I8(reader.read_i8()?),
        Scalar::U16 => Value::U16(reader.read_u16()?),
        Scalar::I16 => Value::I16(reader.read_i16()?),
        Scalar::U32 => Value::U32(reader.read_u32()?),
        Scalar::I32 => Value::I32(reader.read_i32()?),
        Scalar::U64 => Value::U64(reader.read_u64()?),
        Scalar::I64 => Value::I64(reader.read_i64()?),
        Scalar::F32 => Value::F32(reader.read_f32()?),
        Scalar::F64 => Value::F64(reader.read_f64()?),
    })
}

fn read_string<A: Allocator>(
    reader: &mut Reader<'_>,
    builder: &mut FlatBufferBuilder<'_, A>,
) -> Result<WIPOffset<UnionWIPOffset>, DecodeError> {
    let bytes = reader.read_string()?;
    Ok(builder.create_byte_string(bytes).as_union_value())
}

/// Reads a struct into its little-endian image.
///
/// The members are copied twice, into the image and from the image into
/// the builder. The builder takes a struct whole, and only once its table
/// or vector is built, while the reader hands out one member at a time.
///
/// The image is one allocation per struct and per sample. Keeping the
/// CDR offset instead and re-reading the struct into a stack buffer when
/// its table ends would remove it.
fn read_struct(layout: &StructLayout, reader: &mut Reader<'_>) -> Result<Vec<u8>, DecodeError> {
    let mut image = vec![0; layout.size];
    read_struct_into(layout, reader, &mut image)?;
    Ok(image)
}

fn read_struct_into(
    layout: &StructLayout,
    reader: &mut Reader<'_>,
    image: &mut [u8],
) -> Result<(), DecodeError> {
    for field in &layout.fields {
        let image = &mut image[field.offset as usize..];
        match &field.kind {
            StructKind::Scalar(scalar) => read_value(*scalar, reader)?.write_le_bytes(image),
            StructKind::Struct(nested) => read_struct_into(nested, reader, image)?,
        }
    }
    Ok(())
}

fn read_sequence<A: Allocator>(
    element: &ElementKind,
    reader: &mut Reader<'_>,
    builder: &mut FlatBufferBuilder<'_, A>,
) -> Result<WIPOffset<UnionWIPOffset>, DecodeError> {
    match element {
        ElementKind::Scalar(scalar) => {
            let bytes = reader.read_sequence(scalar.size())?;
            Ok(push_scalar_vector(
                builder,
                *scalar,
                bytes,
                reader.endianness(),
            ))
        }
        _ => {
            let len = reader.read_length()?;
            read_elements(len, element, reader, builder)
        }
    }
}

fn read_array<A: Allocator>(
    len: usize,
    element: &ElementKind,
    reader: &mut Reader<'_>,
    builder: &mut FlatBufferBuilder<'_, A>,
) -> Result<WIPOffset<UnionWIPOffset>, DecodeError> {
    match element {
        ElementKind::Scalar(scalar) => {
            let bytes = reader.read_array(len, scalar.size())?;
            Ok(push_scalar_vector(
                builder,
                *scalar,
                bytes,
                reader.endianness(),
            ))
        }
        _ => read_elements(len, element, reader, builder),
    }
}

fn read_elements<A: Allocator>(
    len: usize,
    element: &ElementKind,
    reader: &mut Reader<'_>,
    builder: &mut FlatBufferBuilder<'_, A>,
) -> Result<WIPOffset<UnionWIPOffset>, DecodeError> {
    match element {
        ElementKind::Scalar(_) => unreachable!("scalar vectors are pushed whole"),
        ElementKind::String => {
            let mut offsets = Vec::with_capacity(len);
            for _ in 0..len {
                offsets.push(read_string(reader, builder)?);
            }
            Ok(builder.create_vector(&offsets).as_union_value())
        }
        ElementKind::Table(nested) => {
            let mut offsets = Vec::with_capacity(len);
            for _ in 0..len {
                offsets.push(read_table(&nested.fields, reader, builder)?.as_union_value());
            }
            Ok(builder.create_vector(&offsets).as_union_value())
        }
        ElementKind::Struct(nested) => {
            // The images are copied twice, as in `read_struct`.
            let mut images = vec![0; len * nested.size];
            for image in images.chunks_exact_mut(nested.size) {
                read_struct_into(nested, reader, image)?;
            }
            Ok(push_struct_vector(builder, &images, len, nested.alignment))
        }
    }
}
