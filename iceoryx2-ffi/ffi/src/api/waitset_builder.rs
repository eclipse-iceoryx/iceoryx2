// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use std::{ffi::c_int, mem::ManuallyDrop};

use crate::{
    api::IntoCInt, iox2_service_type_e, iox2_waitset_h, iox2_waitset_t, WaitSetUnion, IOX2_OK,
};

use super::{AssertNonNullHandle, HandleToType};
use iceoryx2::{
    prelude::WaitSetBuilder,
    service::{ipc, local},
};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

struct WaitSetBuilderInternal {
    value: ManuallyDrop<WaitSetBuilder>,
}

#[repr(C)]
#[repr(align(1))] // alignment of Option<WaitSetBuilder>
pub struct iox2_waitset_builder_storage_t {
    internal: [u8; 1], // magic number obtained with size_of::<Option<WaitSetBuilder>>()
}

#[repr(C)]
#[iceoryx2_ffi(WaitSetBuilderInternal)]
pub struct iox2_waitset_builder_t {
    pub value: iox2_waitset_builder_storage_t,
    deleter: fn(*mut iox2_waitset_builder_t),
}

pub struct iox2_waitset_builder_h_t;
/// The owning handle for `iox2_waitset_builder_t`. Passing the handle to an function transfers the ownership.
pub type iox2_waitset_builder_h = *mut iox2_waitset_builder_h_t;

/// The non-owning handle for `iox2_waitset_builder_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_waitset_builder_h_ref = *const iox2_waitset_builder_h;

// NOTE check the README.md for using opaque types with renaming
/// The immutable pointer to the underlying `NodeName`
pub type iox2_waitset_builder_ptr = *const WaitSetBuilder;
/// The mutable pointer to the underlying `NodeName`
pub type iox2_waitset_builder_ptr_mut = *mut WaitSetBuilder;

impl AssertNonNullHandle for iox2_waitset_builder_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_waitset_builder_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_waitset_builder_h {
    type Target = *mut iox2_waitset_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_waitset_builder_h_ref {
    type Target = *mut iox2_waitset_builder_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_builder_new(
    struct_ptr: *mut iox2_waitset_builder_t,
    handle_ptr: *mut iox2_waitset_builder_h,
) {
    debug_assert!(!handle_ptr.is_null());

    *handle_ptr = std::ptr::null_mut();

    let mut struct_ptr = struct_ptr;
    fn no_op(_: *mut iox2_waitset_builder_t) {}
    let mut deleter: fn(*mut iox2_waitset_builder_t) = no_op;
    if struct_ptr.is_null() {
        struct_ptr = iox2_waitset_builder_t::alloc();
        deleter = iox2_waitset_builder_t::dealloc;
    }
    debug_assert!(!struct_ptr.is_null());

    (*struct_ptr).deleter = deleter;
    (*struct_ptr).value.init(WaitSetBuilderInternal {
        value: ManuallyDrop::new(WaitSetBuilder::new()),
    });

    *handle_ptr = (*struct_ptr).as_handle();
}

#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_builder_drop(handle: iox2_waitset_builder_h) {
    debug_assert!(!handle.is_null());

    let waitset_builder = &mut *handle.as_type();
    core::ptr::drop_in_place(waitset_builder.value.as_option_mut());
    (waitset_builder.deleter)(waitset_builder);
}

// Returns [`iox2_waitset_create_error_e`]
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_builder_create_ipc(
    handle: iox2_waitset_builder_h,
    struct_ptr: *mut iox2_waitset_t,
    handle_ptr: *mut iox2_waitset_h,
) -> c_int {
    debug_assert!(!handle.is_null());

    let handle = unsafe { &mut *handle.as_type() };
    let waitset_builder = ManuallyDrop::take(&mut handle.value.as_mut().value);

    let waitset = match waitset_builder.create::<ipc::Service>() {
        Ok(waitset) => waitset,
        Err(e) => return e.into_c_int(),
    };

    *handle_ptr = std::ptr::null_mut();
    let mut struct_ptr = struct_ptr;
    fn no_op(_: *mut iox2_waitset_t) {}
    let mut deleter: fn(*mut iox2_waitset_t) = no_op;
    if struct_ptr.is_null() {
        struct_ptr = iox2_waitset_t::alloc();
        deleter = iox2_waitset_t::dealloc;
    }
    debug_assert!(!struct_ptr.is_null());

    (*struct_ptr).deleter = deleter;
    (*struct_ptr).init(
        iox2_service_type_e::IPC,
        WaitSetUnion::new_ipc(waitset),
        deleter,
    );

    IOX2_OK
}

// Returns [`iox2_waitset_create_error_e`]
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_builder_create_local(
    handle: iox2_waitset_builder_h,
    struct_ptr: *mut iox2_waitset_t,
    handle_ptr: *mut iox2_waitset_h,
) -> c_int {
    debug_assert!(!handle.is_null());

    let handle = unsafe { &mut *handle.as_type() };
    let waitset_builder = ManuallyDrop::take(&mut handle.value.as_mut().value);

    let waitset = match waitset_builder.create::<local::Service>() {
        Ok(waitset) => waitset,
        Err(e) => return e.into_c_int(),
    };

    *handle_ptr = std::ptr::null_mut();
    let mut struct_ptr = struct_ptr;
    fn no_op(_: *mut iox2_waitset_t) {}
    let mut deleter: fn(*mut iox2_waitset_t) = no_op;
    if struct_ptr.is_null() {
        struct_ptr = iox2_waitset_t::alloc();
        deleter = iox2_waitset_t::dealloc;
    }
    debug_assert!(!struct_ptr.is_null());

    (*struct_ptr).deleter = deleter;
    (*struct_ptr).init(
        iox2_service_type_e::LOCAL,
        WaitSetUnion::new_local(waitset),
        deleter,
    );

    IOX2_OK
}

// END C API
