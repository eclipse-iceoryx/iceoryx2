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

use crate::posix::types::*;

pub unsafe fn acl_get_perm(_permset: acl_permset_t, _perm: acl_perm_t) -> int {
    -1
}

pub unsafe fn acl_init(_count: int) -> acl_t {
    0
}

pub unsafe fn acl_free(_data: *mut void) -> int {
    -1
}

pub unsafe fn acl_valid(_acl: acl_t) -> int {
    -1
}

pub unsafe fn acl_create_entry(_acl: *mut acl_t, _entry: *mut acl_entry_t) -> int {
    -1
}

pub unsafe fn acl_get_entry(_acl: acl_t, _entry_id: int, _entry: *mut acl_entry_t) -> int {
    -1
}

pub unsafe fn acl_add_perm(_permset: acl_permset_t, _perm: acl_perm_t) -> int {
    -1
}

pub unsafe fn acl_clear_perms(_permset: acl_permset_t) -> int {
    -1
}

pub unsafe fn acl_get_permset(_entry: acl_entry_t, _permset: *mut acl_permset_t) -> int {
    -1
}

pub unsafe fn acl_set_permset(_entry: acl_entry_t, _permset: acl_permset_t) -> int {
    -1
}

pub unsafe fn acl_get_qualifier(_entry: acl_entry_t) -> *mut void {
    core::ptr::null_mut::<void>()
}

pub unsafe fn acl_set_qualifier(_entry: acl_entry_t, _tag_qualifier: *const void) -> int {
    -1
}

pub unsafe fn acl_get_tag_type(_entry: acl_entry_t, _acl_tag_type: *mut acl_tag_t) -> int {
    -1
}

pub unsafe fn acl_set_tag_type(_entry: acl_entry_t, _acl_tag_type: acl_tag_t) -> int {
    -1
}

pub unsafe fn acl_get_fd(_fd: int) -> acl_t {
    0
}

pub unsafe fn acl_set_fd(_fd: int, _acl: acl_t) -> int {
    -1
}

pub unsafe fn acl_to_text(_acl: acl_t, _len_p: *mut ssize_t) -> *const char {
    core::ptr::null::<char>()
}

pub unsafe fn acl_from_text(_buf_p: *const char) -> acl_t {
    0
}
