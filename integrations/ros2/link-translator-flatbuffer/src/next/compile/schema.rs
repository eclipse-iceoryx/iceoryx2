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

use iceoryx2_bb_flatbuffers::reflection::{self, BaseType, Object};
use iceoryx2_log::{fail, origin};

use crate::next::Refusal;
use crate::next::model::{ScalarType, ScalarValue};

pub(super) fn field<'a>(object: Object<'a>, name: &str) -> Result<reflection::Field<'a>, Refusal> {
    let origin = origin!("compile::schema::field");

    let Some(field) = object.fields().iter().find(|field| field.name() == name) else {
        fail!(
            from origin,
            with Refusal::MissingField,
            "Table '{}' has no field '{}'", object.name(), name
        );
    };
    Ok(field)
}

/// The table or struct `field` holds.
///
/// Returns None when the field holds another type.
pub(super) fn object<'a>(
    schema: reflection::Schema<'a>,
    field: reflection::Field<'_>,
) -> Option<Object<'a>> {
    if field.type_().base_type() != BaseType::Obj {
        return None;
    }
    Some(schema.objects().get(field.type_().index() as usize))
}

/// The table or struct that the elements of the vector `field` holds are.
///
/// Returns None when the field holds no vector of tables or structs.
pub(super) fn element_object<'a>(
    schema: reflection::Schema<'a>,
    field: reflection::Field<'_>,
) -> Option<Object<'a>> {
    if field.type_().base_type() != BaseType::Vector || field.type_().element() != BaseType::Obj {
        return None;
    }
    Some(schema.objects().get(field.type_().index() as usize))
}

/// The type of `field` as written in a schema e.g. `[double]`.
pub(super) fn field_type(field: reflection::Field<'_>) -> String {
    let ty = field.type_();
    match ty.base_type() {
        BaseType::Vector => format!("[{}]", ty.element().variant_name().unwrap_or("?")),
        BaseType::Array => format!(
            "[{}:{}]",
            ty.element().variant_name().unwrap_or("?"),
            ty.fixed_length()
        ),
        base_type => base_type.variant_name().unwrap_or("?").to_string(),
    }
}

pub(super) fn scalar_type(field: reflection::Field<'_>) -> Option<ScalarType> {
    scalar_of(field.type_().base_type())
}

/// The scalar type of the elements of the vector `field` holds.
///
/// Returns None when the field holds no vector of scalars.
pub(super) fn element_scalar_type(field: reflection::Field<'_>) -> Option<ScalarType> {
    if field.type_().base_type() != BaseType::Vector {
        return None;
    }
    scalar_of(field.type_().element())
}

/// The scalar type and the number of elements of the array `field` holds.
///
/// Returns None when the field holds no array of scalars.
pub(super) fn scalar_array(field: reflection::Field<'_>) -> Option<(ScalarType, u32)> {
    if field.type_().base_type() != BaseType::Array {
        return None;
    }
    let scalar = scalar_of(field.type_().element())?;
    Some((scalar, u32::from(field.type_().fixed_length())))
}

pub(super) fn is_string_vector(field: reflection::Field<'_>) -> bool {
    field.type_().base_type() == BaseType::Vector && field.type_().element() == BaseType::String
}

fn scalar_of(base_type: BaseType) -> Option<ScalarType> {
    match base_type {
        BaseType::Bool => Some(ScalarType::Bool),
        BaseType::UByte => Some(ScalarType::U8),
        BaseType::Byte => Some(ScalarType::I8),
        BaseType::UShort => Some(ScalarType::U16),
        BaseType::Short => Some(ScalarType::I16),
        BaseType::UInt => Some(ScalarType::U32),
        BaseType::Int => Some(ScalarType::I32),
        BaseType::ULong => Some(ScalarType::U64),
        BaseType::Long => Some(ScalarType::I64),
        BaseType::Float => Some(ScalarType::F32),
        BaseType::Double => Some(ScalarType::F64),
        _ => None,
    }
}

/// The default of `field` as a value of the type `scalar`.
pub(super) fn default_value(field: reflection::Field<'_>, scalar: ScalarType) -> ScalarValue {
    let (integer, real) = (field.default_integer(), field.default_real());
    match scalar {
        ScalarType::Bool => ScalarValue::Bool(integer != 0),
        ScalarType::U8 => ScalarValue::U8(integer as u8),
        ScalarType::I8 => ScalarValue::I8(integer as i8),
        ScalarType::U16 => ScalarValue::U16(integer as u16),
        ScalarType::I16 => ScalarValue::I16(integer as i16),
        ScalarType::U32 => ScalarValue::U32(integer as u32),
        ScalarType::I32 => ScalarValue::I32(integer as i32),
        ScalarType::U64 => ScalarValue::U64(integer as u64),
        ScalarType::I64 => ScalarValue::I64(integer),
        ScalarType::F32 => ScalarValue::F32(real as f32),
        ScalarType::F64 => ScalarValue::F64(real),
    }
}
