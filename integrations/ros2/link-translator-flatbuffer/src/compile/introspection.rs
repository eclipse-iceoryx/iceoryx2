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

use core::ffi::CStr;

use r2r_rcl::{
    rosidl_typesupport_introspection_c__MessageMember as MessageMember,
    rosidl_typesupport_introspection_c__MessageMembers as MessageMembers,
    rosidl_typesupport_introspection_c_field_types as FieldType,
};

use crate::model::ScalarType;

pub(super) const FLOAT: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_FLOAT as u8;
pub(super) const DOUBLE: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_DOUBLE as u8;
pub(super) const LONG_DOUBLE: u8 =
    FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_LONG_DOUBLE as u8;
pub(super) const CHAR: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_CHAR as u8;
pub(super) const WCHAR: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_WCHAR as u8;
pub(super) const BOOLEAN: u8 =
    FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_BOOLEAN as u8;
pub(super) const OCTET: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_OCTET as u8;
pub(super) const UINT8: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_UINT8 as u8;
pub(super) const INT8: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_INT8 as u8;
pub(super) const UINT16: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_UINT16 as u8;
pub(super) const INT16: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_INT16 as u8;
pub(super) const UINT32: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_UINT32 as u8;
pub(super) const INT32: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_INT32 as u8;
pub(super) const UINT64: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_UINT64 as u8;
pub(super) const INT64: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_INT64 as u8;
pub(super) const STRING: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_STRING as u8;
pub(super) const WSTRING: u8 =
    FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_WSTRING as u8;
pub(super) const MESSAGE: u8 =
    FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_MESSAGE as u8;

/// The members of the message `member` holds.
///
/// # Safety
///
/// The introspection typesupport of `member` must be loaded and `member` must
/// be a message.
pub(super) unsafe fn message(member: &MessageMember) -> *const MessageMembers {
    unsafe { (*member.members_).data }.cast::<MessageMembers>()
}

/// # Safety
///
/// The introspection typesupport that `message` points into must be loaded.
pub(super) unsafe fn members<'a>(message: *const MessageMembers) -> &'a [MessageMember] {
    let message = unsafe { &*message };
    unsafe { core::slice::from_raw_parts(message.members_, message.member_count_ as usize) }
}

/// # Safety
///
/// The introspection typesupport of `member` must be loaded.
pub(super) unsafe fn member_name(member: &MessageMember) -> &str {
    unsafe { CStr::from_ptr(member.name_) }
        .to_str()
        .expect("rosidl names members in ASCII")
}

/// The type of `member` as written in a `.msg` e.g. `float64[3]`.
pub(super) fn member_type(member: &MessageMember) -> String {
    let base = match member.type_id_ {
        FLOAT => "float32".to_string(),
        DOUBLE => "float64".to_string(),
        LONG_DOUBLE => "long double".to_string(),
        CHAR => "char".to_string(),
        WCHAR => "wchar".to_string(),
        BOOLEAN => "bool".to_string(),
        OCTET => "octet".to_string(),
        UINT8 => "uint8".to_string(),
        INT8 => "int8".to_string(),
        UINT16 => "uint16".to_string(),
        INT16 => "int16".to_string(),
        UINT32 => "uint32".to_string(),
        INT32 => "int32".to_string(),
        UINT64 => "uint64".to_string(),
        INT64 => "int64".to_string(),
        STRING => "string".to_string(),
        WSTRING => "wstring".to_string(),
        MESSAGE => {
            // SAFETY: a message member refers to its members' typesupport.
            let nested = unsafe { &*message(member) };
            let namespace = unsafe { CStr::from_ptr(nested.message_namespace_) }.to_string_lossy();
            let name = unsafe { CStr::from_ptr(nested.message_name_) }.to_string_lossy();
            format!("{}/{}", namespace.replace("__", "/"), name)
        }
        other => format!("type {other}"),
    };
    if !member.is_array_ {
        base
    } else if member.array_size_ == 0 {
        format!("{base}[]")
    } else if member.is_upper_bound_ {
        format!("{base}[<={}]", member.array_size_)
    } else {
        format!("{base}[{}]", member.array_size_)
    }
}

pub(super) fn is_sequence(member: &MessageMember) -> bool {
    member.is_array_ && (member.array_size_ == 0 || member.is_upper_bound_)
}

/// Whether `member` is an array of a fixed number of elements.
pub(super) fn is_array(member: &MessageMember) -> bool {
    member.is_array_ && member.array_size_ > 0 && !member.is_upper_bound_
}

pub(super) fn scalar_type(member: &MessageMember) -> Option<ScalarType> {
    match member.type_id_ {
        BOOLEAN => Some(ScalarType::Bool),
        OCTET | UINT8 | CHAR => Some(ScalarType::U8),
        INT8 => Some(ScalarType::I8),
        UINT16 => Some(ScalarType::U16),
        INT16 => Some(ScalarType::I16),
        UINT32 => Some(ScalarType::U32),
        INT32 => Some(ScalarType::I32),
        UINT64 => Some(ScalarType::U64),
        INT64 => Some(ScalarType::I64),
        FLOAT => Some(ScalarType::F32),
        DOUBLE => Some(ScalarType::F64),
        _ => None,
    }
}
