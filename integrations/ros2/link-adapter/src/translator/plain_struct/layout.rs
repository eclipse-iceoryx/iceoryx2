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

use core::alloc::Layout;

use iceoryx2_log::{fail, origin};
use r2r_rcl::{
    rosidl_typesupport_introspection_c__MessageMembers as MessageMembers,
    rosidl_typesupport_introspection_c_field_types as FieldType,
};

use super::TranslationError;
use crate::typesupport;

const FLOAT: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_FLOAT as u8;
const DOUBLE: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_DOUBLE as u8;
const CHAR: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_CHAR as u8;
const WCHAR: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_WCHAR as u8;
const BOOLEAN: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_BOOLEAN as u8;
const OCTET: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_OCTET as u8;
const UINT8: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_UINT8 as u8;
const INT8: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_INT8 as u8;
const UINT16: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_UINT16 as u8;
const INT16: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_INT16 as u8;
const UINT32: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_UINT32 as u8;
const INT32: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_INT32 as u8;
const UINT64: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_UINT64 as u8;
const INT64: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_INT64 as u8;
const MESSAGE: u8 = FieldType::rosidl_typesupport_introspection_c__ROS_TYPE_MESSAGE as u8;

/// The layout of the C struct of the ROS 2 type `type_name`.
pub(super) fn layout_of(type_name: &str) -> Result<Layout, TranslationError> {
    let origin = origin!("PlainStructTranslator::layout_of");
    let introspection = fail!(
        from origin,
        when typesupport::load_introspection(type_name),
        with TranslationError::Introspection,
        "Failed to load introspection for type '{}'", type_name
    );
    let members = unsafe { (*introspection.handle()).data }.cast::<MessageMembers>();
    match unsafe { introspected_layout(members) } {
        IntrospectedLayout::FixedSize(layout) => Ok(layout),
        IntrospectedLayout::Unsupported => {
            fail!(
                from origin,
                with TranslationError::UnsupportedType,
                "ROS 2 type '{}' has dynamically sized or platform-defined members", type_name
            );
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum IntrospectedLayout {
    /// Every member is stored inline.
    FixedSize(Layout),
    /// A member is pointer-backed, strings and sequences, or has a
    /// platform-defined layout, long double.
    Unsupported,
}

/// Computes the layout of the rosidl C struct described by `members`.
///
/// # Safety
///
/// `members` must point to a valid introspection member table whose
/// library stays loaded for the duration of the call.
unsafe fn introspected_layout(members: *const MessageMembers) -> IntrospectedLayout {
    let table = unsafe { &*members };

    let mut alignment = 1;
    for index in 0..table.member_count_ as usize {
        let member = unsafe { &*table.members_.add(index) };

        if member.is_array_ && (member.array_size_ == 0 || member.is_upper_bound_) {
            return IntrospectedLayout::Unsupported;
        }

        let member_alignment = if member.type_id_ == MESSAGE {
            let nested = unsafe { (*member.members_).data }.cast::<MessageMembers>();
            match unsafe { introspected_layout(nested) } {
                IntrospectedLayout::FixedSize(layout) => layout.align(),
                IntrospectedLayout::Unsupported => return IntrospectedLayout::Unsupported,
            }
        } else {
            match basic_member_alignment(member.type_id_) {
                Some(member_alignment) => member_alignment,
                None => return IntrospectedLayout::Unsupported,
            }
        };
        alignment = alignment.max(member_alignment);
    }

    IntrospectedLayout::FixedSize(
        Layout::from_size_align(table.size_of_, alignment)
            .expect("introspected size and alignment form a valid layout"),
    )
}

/// The alignment of a basic member type in the rosidl C struct. Strings and
/// long double are unsupported and have none.
fn basic_member_alignment(type_id: u8) -> Option<usize> {
    match type_id {
        BOOLEAN | OCTET | CHAR | UINT8 | INT8 => Some(1),
        WCHAR | UINT16 | INT16 => Some(2),
        FLOAT | UINT32 | INT32 => Some(4),
        DOUBLE | UINT64 | INT64 => Some(8),
        _ => None,
    }
}
