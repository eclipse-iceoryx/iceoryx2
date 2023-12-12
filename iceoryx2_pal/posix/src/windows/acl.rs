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

pub unsafe fn acl_to_text(acl: acl_t, len_p: *mut ssize_t) -> *const char {
    core::ptr::null::<char>()
}

pub unsafe fn acl_from_text(buf_p: *const char) -> acl_t {
    acl_t::MAX
}
