// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

#![allow(non_camel_case_types, non_snake_case)]
#![allow(clippy::missing_safety_doc)]
#![allow(unused_variables)]

use alloc::alloc::{alloc, dealloc, Layout};

use crate::posix::types::*;

pub unsafe fn malloc(size: size_t) -> *mut void {
    if size == 0 {
        return core::ptr::null_mut();
    }

    let layout = Layout::from_size_align(size as usize, core::mem::align_of::<usize>())
        .unwrap_or(Layout::new::<u8>());

    let ptr = alloc(layout);
    if ptr.is_null() {
        return core::ptr::null_mut();
    }

    ptr as *mut void
}

pub unsafe fn calloc(nmemb: size_t, size: size_t) -> *mut void {
    let total_size = nmemb.checked_mul(size);
    match total_size {
        Some(total) => {
            let ptr = malloc(total);
            if !ptr.is_null() {
                core::ptr::write_bytes(ptr, 0, total as usize);
            }
            ptr
        }
        None => core::ptr::null_mut(),
    }
}

pub unsafe fn realloc(ptr: *mut void, size: size_t) -> *mut void {
    if ptr.is_null() {
        return malloc(size);
    }

    if size == 0 {
        free(ptr);
        return core::ptr::null_mut();
    }

    let new_layout = Layout::from_size_align(size as usize, core::mem::align_of::<usize>())
        .unwrap_or(Layout::new::<u8>());

    let new_ptr = malloc(size);
    if !new_ptr.is_null() {
        core::ptr::copy_nonoverlapping(
            ptr,
            new_ptr,
            core::cmp::min(size as usize, size as usize), // This is simplified, ideally we'd know the old size
        );
        free(ptr);
    }

    new_ptr
}

pub unsafe fn free(ptr: *mut void) {
    if ptr.is_null() {
        return;
    }

    let layout =
        Layout::from_size_align(1, core::mem::align_of::<usize>()).unwrap_or(Layout::new::<u8>());

    dealloc(ptr as *mut u8, layout);
}
