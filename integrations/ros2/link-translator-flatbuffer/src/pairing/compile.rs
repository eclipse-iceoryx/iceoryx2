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

use iceoryx2_bb_flatbuffers::reflection::{self, BaseType, Object, Schema};
use iceoryx2_log::{fail, origin};
use r2r_rcl::{
    rosidl_typesupport_introspection_c__MessageMember as MessageMember,
    rosidl_typesupport_introspection_c__MessageMembers as MessageMembers,
};

use super::{
    Default, ElementKind, FieldKind, PairingError, Refusal, Scalar, StructField, StructKind,
    StructLayout, Table, TableField, flatbuffer, ros2,
};

/// Pairs the members of `message` with the fields of the table `object`.
///
/// # Safety
///
/// `message` must point to the members of a loaded introspection typesupport.
pub(super) unsafe fn table(
    object: Object<'_>,
    message: *const MessageMembers,
    schema: &Schema<'_>,
) -> Result<Table, PairingError> {
    let mut fields = Vec::new();

    for member in unsafe { ros2::members(message) } {
        let name = unsafe { ros2::member_name(member) };
        let field =
            unsafe { table_field(object, member, name, schema) }.map_err(|error| error.at(name))?;
        fields.push(field);
    }

    Ok(Table { fields })
}

/// Pairs `member` with the field of `object` named `name`.
///
/// # Safety
///
/// `member` must belong to a loaded introspection typesupport.
unsafe fn table_field(
    object: Object<'_>,
    member: &MessageMember,
    name: &str,
    schema: &Schema<'_>,
) -> Result<TableField, PairingError> {
    let field = flatbuffer::field(object, name)?;
    let kind = unsafe { field_kind(member, field.type_(), field, schema)? };

    Ok(TableField {
        vtable_offset: field.offset(),
        kind,
    })
}

/// Pairs the members of `message` with the fields of the struct `object`.
///
/// # Safety
///
/// `message` must point to the members of a loaded introspection typesupport.
unsafe fn struct_layout(
    object: Object<'_>,
    message: *const MessageMembers,
    schema: &Schema<'_>,
) -> Result<StructLayout, PairingError> {
    let mut fields = Vec::new();

    for member in unsafe { ros2::members(message) } {
        let name = unsafe { ros2::member_name(member) };
        let field = unsafe { struct_field(object, member, name, schema) }
            .map_err(|error| error.at(name))?;
        fields.push(field);
    }

    Ok(StructLayout {
        size: object.bytesize() as usize,
        alignment: object.minalign() as usize,
        fields,
    })
}

/// Pairs `member` with the struct field of `object` named `name`.
///
/// # Safety
///
/// `member` must belong to a loaded introspection typesupport.
unsafe fn struct_field(
    object: Object<'_>,
    member: &MessageMember,
    name: &str,
    schema: &Schema<'_>,
) -> Result<StructField, PairingError> {
    let field = flatbuffer::field(object, name)?;

    if member.is_array_ {
        return Err(mismatch(member, field));
    }
    let kind = match (member.type_id_, field.type_().base_type()) {
        (ros2::MESSAGE, BaseType::Obj) => {
            let nested = flatbuffer::object(schema, field.type_().index());
            if !nested.is_struct() {
                return Err(mismatch(member, field));
            }
            StructKind::Struct(unsafe { struct_layout(nested, ros2::message(member), schema)? })
        }
        (_, base_type) => StructKind::Scalar(scalar(member, base_type, field)?),
    };

    Ok(StructField {
        offset: field.offset(),
        kind,
    })
}

/// The kind of value `member` and the field of type `ty` agree on.
///
/// # Safety
///
/// `member` must belong to a loaded introspection typesupport.
unsafe fn field_kind(
    member: &MessageMember,
    ty: reflection::Type<'_>,
    field: reflection::Field<'_>,
    schema: &Schema<'_>,
) -> Result<FieldKind, PairingError> {
    if member.is_array_ {
        if ty.base_type() != BaseType::Vector {
            return Err(mismatch(member, field));
        }
        let element = unsafe { element_kind(member, ty.element(), ty.index(), field, schema)? };
        return Ok(if member.array_size_ > 0 && !member.is_upper_bound_ {
            FieldKind::Array {
                len: member.array_size_,
                element,
            }
        } else {
            FieldKind::Sequence(element)
        });
    }
    match (member.type_id_, ty.base_type()) {
        (ros2::STRING, BaseType::String) => Ok(FieldKind::String),
        (ros2::MESSAGE, BaseType::Obj) => {
            let nested = flatbuffer::object(schema, ty.index());
            Ok(if nested.is_struct() {
                FieldKind::Struct(unsafe { struct_layout(nested, ros2::message(member), schema)? })
            } else {
                FieldKind::Table(unsafe { table(nested, ros2::message(member), schema)? })
            })
        }
        (_, base_type) => Ok(FieldKind::Scalar {
            scalar: scalar(member, base_type, field)?,
            default: Default {
                integer: field.default_integer(),
                real: field.default_real(),
            },
        }),
    }
}

/// The element a vector and the sequence or array member agree on.
///
/// # Safety
///
/// `member` must belong to a loaded introspection typesupport.
unsafe fn element_kind(
    member: &MessageMember,
    base_type: BaseType,
    index: i32,
    field: reflection::Field<'_>,
    schema: &Schema<'_>,
) -> Result<ElementKind, PairingError> {
    match (member.type_id_, base_type) {
        (ros2::STRING, BaseType::String) => Ok(ElementKind::String),
        (ros2::MESSAGE, BaseType::Obj) => {
            let nested = flatbuffer::object(schema, index);
            Ok(if nested.is_struct() {
                ElementKind::Struct(unsafe {
                    struct_layout(nested, ros2::message(member), schema)?
                })
            } else {
                ElementKind::Table(unsafe { table(nested, ros2::message(member), schema)? })
            })
        }
        (_, base_type) => Ok(ElementKind::Scalar(scalar(member, base_type, field)?)),
    }
}

/// The primitive `member` and a field of `base_type` agree on.
fn scalar(
    member: &MessageMember,
    base_type: BaseType,
    field: reflection::Field<'_>,
) -> Result<Scalar, PairingError> {
    let origin = origin!("pairing::compile::scalar");

    if matches!(
        member.type_id_,
        ros2::WSTRING | ros2::WCHAR | ros2::LONG_DOUBLE
    ) {
        fail!(
            from origin,
            with PairingError { path: String::new(), refusal: Refusal::Unsupported { member: ros2::member_type(member) } },
            "A {} member has no flatbuffer field that carries it", ros2::member_type(member)
        );
    }
    let scalar = match (member.type_id_, base_type) {
        (ros2::BOOLEAN, BaseType::Bool) => Scalar::Bool,
        (ros2::OCTET | ros2::UINT8 | ros2::CHAR, BaseType::UByte) => Scalar::U8,
        (ros2::INT8, BaseType::Byte) => Scalar::I8,
        (ros2::UINT16, BaseType::UShort) => Scalar::U16,
        (ros2::INT16, BaseType::Short) => Scalar::I16,
        (ros2::UINT32, BaseType::UInt) => Scalar::U32,
        (ros2::INT32, BaseType::Int) => Scalar::I32,
        (ros2::UINT64, BaseType::ULong) => Scalar::U64,
        (ros2::INT64, BaseType::Long) => Scalar::I64,
        (ros2::FLOAT, BaseType::Float) => Scalar::F32,
        (ros2::DOUBLE, BaseType::Double) => Scalar::F64,
        _ => return Err(mismatch(member, field)),
    };

    Ok(scalar)
}

fn mismatch(member: &MessageMember, field: reflection::Field<'_>) -> PairingError {
    let origin = origin!("pairing::compile::mismatch");
    let (member, field) = (ros2::member_type(member), flatbuffer::field_type(field));
    iceoryx2_log::debug!(
        from origin,
        "A {} member does not pair with a {} field", member, field
    );
    PairingError {
        path: String::new(),
        refusal: Refusal::Mismatch { member, field },
    }
}
