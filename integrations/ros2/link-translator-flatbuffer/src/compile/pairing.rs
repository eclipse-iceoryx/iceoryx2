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

use r2r_rcl::{
    rosidl_typesupport_introspection_c__MessageMember as MessageMember,
    rosidl_typesupport_introspection_c__MessageMembers as MessageMembers,
};

use super::{introspection, join_path};

/// A member of a message and the field it is paired with.
pub(super) struct Pair<'a> {
    pub member: &'a MessageMember,
    /// The path of the member from the root message.
    pub member_path: String,
    /// The path of the field from the table or the struct of the message.
    pub field_path: &'a str,
}

/// Pairs each member of `message` with the field of its name.
///
/// # Safety
///
/// The introspection typesupport that `message` points into must be loaded.
pub(super) unsafe fn pair_members_with_fields<'a>(
    message: *const MessageMembers,
    message_path: &str,
) -> Vec<Pair<'a>> {
    unsafe { introspection::members(message) }
        .iter()
        .map(|member| {
            let name = unsafe { introspection::member_name(member) };
            Pair {
                member,
                member_path: join_path(message_path, name),
                field_path: name,
            }
        })
        .collect()
}
