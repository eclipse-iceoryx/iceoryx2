// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]
#![allow(unused_variables)]

use crate::posix::types::*;

pub const ACL_READ: acl_perm_t = 1;
pub const ACL_WRITE: acl_perm_t = 2;
pub const ACL_EXECUTE: acl_perm_t = 4;

pub const ACL_UNDEFINED_TAG: acl_tag_t = 0;
pub const ACL_USER_OBJ: acl_tag_t = 1;
pub const ACL_USER: acl_tag_t = 2;
pub const ACL_GROUP_OBJ: acl_tag_t = 4;
pub const ACL_GROUP: acl_tag_t = 8;
pub const ACL_MASK: acl_tag_t = 16;
pub const ACL_OTHER: acl_tag_t = 32;

pub const ACL_FIRST_ENTRY: int = 0;
pub const ACL_NEXT_ENTRY: int = 1;

pub type acl_t = u64;
pub type acl_permset_t = u64;
pub type acl_entry_t = u64;

pub struct acl_type_t {}
impl Struct for acl_type_t {}

pub type acl_tag_t = u64;
pub type acl_perm_t = u32;

pub unsafe fn acl_get_perm(permset: acl_permset_t, perm: acl_perm_t) -> int {
    -1
}

pub unsafe fn acl_init(count: int) -> acl_t {
    acl_t::MAX
}

pub unsafe fn acl_free(data: *mut void) -> int {
    -1
}

pub unsafe fn acl_valid(acl: acl_t) -> int {
    -1
}

pub unsafe fn acl_create_entry(acl: *mut acl_t, entry: *mut acl_entry_t) -> int {
    -1
}

pub unsafe fn acl_get_entry(acl: acl_t, entry_id: int, entry: *mut acl_entry_t) -> int {
    -1
}

pub unsafe fn acl_add_perm(permset: acl_permset_t, perm: acl_perm_t) -> int {
    -1
}

pub unsafe fn acl_clear_perms(permset: acl_permset_t) -> int {
    -1
}

pub unsafe fn acl_get_permset(entry: acl_entry_t, permset: *mut acl_permset_t) -> int {
    -1
}

pub unsafe fn acl_set_permset(entry: acl_entry_t, permset: acl_permset_t) -> int {
    -1
}

pub unsafe fn acl_get_qualifier(entry: acl_entry_t) -> *mut void {
    core::ptr::null_mut::<void>()
}

pub unsafe fn acl_set_qualifier(entry: acl_entry_t, tag_qualifier: *const void) -> int {
    -1
}

pub unsafe fn acl_get_tag_type(entry: acl_entry_t, acl_tag_type: *mut acl_tag_t) -> int {
    -1
}

pub unsafe fn acl_set_tag_type(entry: acl_entry_t, acl_tag_type: acl_tag_t) -> int {
    -1
}

pub unsafe fn acl_get_fd(fd: int) -> acl_t {
    acl_t::MAX
}

pub unsafe fn acl_set_fd(fd: int, acl: acl_t) -> int {
    -1
}

pub unsafe fn acl_to_text(acl: acl_t, len_p: *mut ssize_t) -> *const c_char {
    core::ptr::null::<c_char>()
}

pub unsafe fn acl_from_text(buf_p: *const c_char) -> acl_t {
    acl_t::MAX
}
