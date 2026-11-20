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

//! The pairing of a flatbuffer with a ROS 2 message, compiled once from
//! the schema and the message's introspection.
//!
//! A [`Table`] holds its fields in the member order of the message,
//! which is the order of the CDR. Each field records
//! its slot and the [`FieldKind`] both sides agree on. The first members
//! of `sensor_msgs/JointState` compile to the tree below.
//!
//! ```text
//! pairing                                     ROS 2 member               flatbuffer field
//! Table                                       sensor_msgs/JointState     table JointState
//! ├─ TableField slot 4  Table                 std_msgs/Header header     header:Header
//! │  ├─ TableField slot 4  Table              Time stamp                 stamp:Time
//! │  │  ├─ TableField slot 4  Scalar I32      int32 sec                  sec:int
//! │  │  └─ TableField slot 6  Scalar U32      uint32 nanosec             nanosec:uint
//! │  └─ TableField slot 6  String             string frame_id            frame_id:string
//! ├─ TableField slot 6  Sequence(String)      string[] name              name:[string]
//! └─ TableField slot 8  Sequence(Scalar F64)  float64[] position         position:[double]
//! ```

mod compile;
mod flatbuffer;
mod ros2;

use iceoryx2_bb_flatbuffers::reflection::root_as_schema;
use iceoryx2_integrations_ros2_link_adapter::typesupport::{self, IntrospectionKind, LoadError};
use iceoryx2_log::{fail, origin};

/// A flatbuffer table paired member by member with a ROS 2 message.
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    /// In the member order of the corresponding CDR.
    pub fields: Vec<TableField>,
}

/// A field of a flatbuffer table paired with a member of a ROS 2 message.
#[derive(Debug, Clone, PartialEq)]
pub struct TableField {
    /// The slot in the vtable that locates the field.
    pub vtable_offset: u16,
    pub kind: FieldKind,
}

/// The kinds a flatbuffer table can hold.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldKind {
    Scalar { scalar: Scalar, default: Default },
    String,
    Table(Table),
    Struct(StructLayout),
    Sequence(ElementKind),
    Array { len: usize, element: ElementKind },
}

/// The element type of a sequence.
#[derive(Debug, Clone, PartialEq)]
pub enum ElementKind {
    Scalar(Scalar),
    String,
    Table(Table),
    Struct(StructLayout),
}

/// The layout of a flatbuffer struct whose fields pair with the members of a ROS 2 message.
#[derive(Debug, Clone, PartialEq)]
pub struct StructLayout {
    pub size: usize,
    pub alignment: usize,
    /// In member order of the CDR.
    pub fields: Vec<StructField>,
}

/// A field of a flatbuffer struct paired with a member of a ROS 2 message.
#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    /// The byte offset in the struct.
    pub offset: u16,
    pub kind: StructKind,
}

/// The kinds a flatbuffer struct can hold.
#[derive(Debug, Clone, PartialEq)]
pub enum StructKind {
    Scalar(Scalar),
    Struct(StructLayout),
}

/// A primitive with the same width and signedness on both sides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scalar {
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

impl Scalar {
    /// The size of the primitive in bytes.
    pub fn size(self) -> usize {
        match self {
            Scalar::Bool | Scalar::U8 | Scalar::I8 => 1,
            Scalar::U16 | Scalar::I16 => 2,
            Scalar::U32 | Scalar::I32 | Scalar::F32 => 4,
            Scalar::U64 | Scalar::I64 | Scalar::F64 => 8,
        }
    }
}

/// The value a scalar field has when it is absent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Default {
    pub integer: i64,
    pub real: f64,
}

/// The value of a scalar in the host's representation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
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

impl Value {
    /// The `default` of an absent `scalar`.
    pub fn default(scalar: Scalar, default: Default) -> Self {
        match scalar {
            Scalar::Bool => Value::Bool(default.integer != 0),
            Scalar::U8 => Value::U8(default.integer as u8),
            Scalar::I8 => Value::I8(default.integer as i8),
            Scalar::U16 => Value::U16(default.integer as u16),
            Scalar::I16 => Value::I16(default.integer as i16),
            Scalar::U32 => Value::U32(default.integer as u32),
            Scalar::I32 => Value::I32(default.integer as i32),
            Scalar::U64 => Value::U64(default.integer as u64),
            Scalar::I64 => Value::I64(default.integer),
            Scalar::F32 => Value::F32(default.real as f32),
            Scalar::F64 => Value::F64(default.real),
        }
    }

    /// The little-endian `scalar` at the start of `bytes`.
    pub fn from_le_bytes(scalar: Scalar, bytes: &[u8]) -> Self {
        macro_rules! from_le_bytes {
            ($ty:ty) => {
                <$ty>::from_le_bytes(
                    bytes[..size_of::<$ty>()]
                        .try_into()
                        .expect("the slice is the scalar's size"),
                )
            };
        }
        match scalar {
            Scalar::Bool => Value::Bool(bytes[0] != 0),
            Scalar::U8 => Value::U8(bytes[0]),
            Scalar::I8 => Value::I8(bytes[0] as i8),
            Scalar::U16 => Value::U16(from_le_bytes!(u16)),
            Scalar::I16 => Value::I16(from_le_bytes!(i16)),
            Scalar::U32 => Value::U32(from_le_bytes!(u32)),
            Scalar::I32 => Value::I32(from_le_bytes!(i32)),
            Scalar::U64 => Value::U64(from_le_bytes!(u64)),
            Scalar::I64 => Value::I64(from_le_bytes!(i64)),
            Scalar::F32 => Value::F32(from_le_bytes!(f32)),
            Scalar::F64 => Value::F64(from_le_bytes!(f64)),
        }
    }

    /// Writes the value little-endian at the start of `bytes`.
    pub fn write_le_bytes(self, bytes: &mut [u8]) {
        macro_rules! write_le_bytes {
            ($value:expr) => {{
                let le_bytes = $value.to_le_bytes();
                bytes[..le_bytes.len()].copy_from_slice(&le_bytes);
            }};
        }
        match self {
            Value::Bool(value) => bytes[0] = value as u8,
            Value::U8(value) => bytes[0] = value,
            Value::I8(value) => bytes[0] = value as u8,
            Value::U16(value) => write_le_bytes!(value),
            Value::I16(value) => write_le_bytes!(value),
            Value::U32(value) => write_le_bytes!(value),
            Value::I32(value) => write_le_bytes!(value),
            Value::U64(value) => write_le_bytes!(value),
            Value::I64(value) => write_le_bytes!(value),
            Value::F32(value) => write_le_bytes!(value),
            Value::F64(value) => write_le_bytes!(value),
        }
    }
}

/// Why a message and a schema do not pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The schema is not a binary flatbuffer schema or declares no root type.
    Schema(iceoryx2_bb_flatbuffers::SchemaError),
    /// The message's introspection typesupport could not be loaded.
    Introspection(LoadError),
    /// The member has no field of its name.
    MissingField,
    /// The member and the field carry different kinds of value.
    Mismatch { member: String, field: String },
    /// No flatbuffer field carries the member's type.
    Unsupported { member: String },
}

/// A member of a message, by path, that does not pair with its schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairingError {
    /// The member's path from the root, `header.stamp.sec`, empty for the
    /// root itself.
    pub path: String,
    pub refusal: Refusal,
}

impl core::fmt::Display for PairingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}: {:?}", self.path, self.refusal)
    }
}

impl core::error::Error for PairingError {}

impl PairingError {
    /// The error as seen from the message holding the member `name`.
    fn at(mut self, name: &str) -> Self {
        self.path = if self.path.is_empty() {
            name.to_string()
        } else {
            format!("{name}.{}", self.path)
        };
        self
    }
}

/// Compiles the pairing of `schema` with the ROS 2 message `message`.
///
/// Fails when the schema or the message cannot be read, or when a
/// member has no field of its name and kind.
pub fn compile(schema: &[u8], message: &str) -> Result<Table, PairingError> {
    let origin = origin!("pairing::compile");

    let schema = fail!(
        from origin,
        when root_as_schema(schema),
        with PairingError { path: String::new(), refusal: Refusal::Schema(iceoryx2_bb_flatbuffers::SchemaError::NotABinarySchema) },
        "The {} bytes are not a binary flatbuffer schema", schema.len()
    );
    let Some(root) = schema.root_table() else {
        fail!(
            from origin,
            with PairingError { path: String::new(), refusal: Refusal::Schema(iceoryx2_bb_flatbuffers::SchemaError::NoRootType) },
            "The schema declares no root_type"
        );
    };
    let introspection = match typesupport::get::<IntrospectionKind>(message) {
        Ok(introspection) => introspection,
        Err(error) => {
            fail!(
                from origin,
                with PairingError { path: String::new(), refusal: Refusal::Introspection(error) },
                "The introspection of '{}' could not be loaded", message
            );
        }
    };

    // SAFETY: the typesupport keeps the members valid while it is loaded.
    let table = match unsafe { compile::table(root, introspection.members(), &schema) } {
        Ok(table) => table,
        Err(error) => {
            fail!(
                from origin,
                with error,
                "Member '{}' of '{}' does not pair with its field in '{}'", error.path, message, root.name()
            );
        }
    };

    Ok(table)
}
