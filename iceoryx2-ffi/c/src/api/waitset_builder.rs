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

use core::ffi::c_int;
use core::mem::ManuallyDrop;

use crate::{
    IOX2_OK, WaitSetUnion, api::IntoCInt, iox2_service_type_e, iox2_waitset_h, iox2_waitset_t,
};

use super::{AssertNonNullHandle, HandleToType, iox2_signal_handling_mode_e};
use iceoryx2::prelude::WaitSetBuilder;
use iceoryx2_ffi_macros::iceoryx2_ffi;

pub(super) union WaitSetBuilderUnion {
    ipc: ManuallyDrop<WaitSetBuilder<crate::IpcService>>,
    local: ManuallyDrop<WaitSetBuilder<crate::LocalService>>,
}

impl WaitSetBuilderUnion {
    pub(super) fn new_ipc(builder: WaitSetBuilder<crate::IpcService>) -> Self {
        Self {
            ipc: ManuallyDrop::new(builder),
        }
    }
    pub(super) fn new_local(builder: WaitSetBuilder<crate::LocalService>) -> Self {
        Self {
            local: ManuallyDrop::new(builder),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<WaitSetBuilderUnion>
pub struct iox2_waitset_builder_storage_t {
    internal: [u8; 32], // magic number obtained with size_of::<Option<WaitSetBuilder>>()
}

#[repr(C)]
#[iceoryx2_ffi(WaitSetBuilderUnion)]
pub struct iox2_waitset_builder_t {
    service_type: iox2_service_type_e,
    value: iox2_waitset_builder_storage_t,
    deleter: fn(*mut iox2_waitset_builder_t),
}

impl iox2_waitset_builder_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: WaitSetBuilderUnion,
        deleter: fn(*mut iox2_waitset_builder_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_waitset_builder_h_t;
/// The owning handle for `iox2_waitset_builder_t`. Passing the handle to an function transfers the ownership.
pub type iox2_waitset_builder_h = *mut iox2_waitset_builder_h_t;

/// The non-owning handle for `iox2_waitset_builder_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_waitset_builder_h_ref = *const iox2_waitset_builder_h;

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
/// Drops a [`iox2_waitset_builder_h`] and calls all corresponding cleanup functions.
///
/// # Safety
///
///  * `handle` must be acquired with [`iox2_node_waitset_builder()`](crate::iox2_node_waitset_builder)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_waitset_builder_drop(handle: iox2_waitset_builder_h) {
    handle.assert_non_null();
    unsafe {
        let builder = &mut *handle.as_type();

        match builder.service_type {
            iox2_service_type_e::IPC => {
                ManuallyDrop::drop(&mut builder.value.as_mut().ipc);
            }
            iox2_service_type_e::LOCAL => {
                ManuallyDrop::drop(&mut builder.value.as_mut().local);
            }
        }
        (builder.deleter)(builder);
    }
}

/// Creates a new [`iox2_waitset_t`].
///
/// # Returns
///
///  [`IOX2_OK`] on success otherwise
///  [`iox2_waitset_create_error_e`](crate::iox2_waitset_create_error_e).
///
/// # Safety
///
///  * `handle` must be acquired with [`iox2_node_waitset_builder()`](crate::iox2_node_waitset_builder)
///  * `handle` is invalidated after a successful operation and cannot used again
///    with this function.
///  * `struct_ptr` must be either a valid pointer to uninitialized memory or `null`
///  * `handle_ptr` must point to a valid uninitialized memory location
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_waitset_builder_create(
    handle: iox2_waitset_builder_h,
    struct_ptr: *mut iox2_waitset_t,
    handle_ptr: *mut iox2_waitset_h,
) -> c_int {
    debug_assert!(!handle.is_null());

    let waitset_builder_struct = unsafe { &mut *handle.as_type() };
    let waitset_builder = waitset_builder_struct.take().unwrap();
    unsafe {
        fn no_op(_: *mut iox2_waitset_t) {}
        let mut deleter: fn(*mut iox2_waitset_t) = no_op;
        let mut struct_ptr = struct_ptr;
        *handle_ptr = core::ptr::null_mut();

        let mut alloc_memory = || {
            if struct_ptr.is_null() {
                struct_ptr = iox2_waitset_t::alloc();
                deleter = iox2_waitset_t::dealloc;
            }
            debug_assert!(!struct_ptr.is_null());

            (*struct_ptr).deleter = deleter;
        };

        match waitset_builder_struct.service_type {
            iox2_service_type_e::IPC => {
                let builder = ManuallyDrop::into_inner(waitset_builder.ipc);
                let waitset = match builder.create() {
                    Ok(waitset) => waitset,
                    Err(e) => return e.into_c_int(),
                };

                alloc_memory();

                (*struct_ptr).init(
                    iox2_service_type_e::IPC,
                    WaitSetUnion::new_ipc(waitset),
                    deleter,
                );
            }
            iox2_service_type_e::LOCAL => {
                let builder = ManuallyDrop::into_inner(waitset_builder.local);
                let waitset = match builder.create() {
                    Ok(waitset) => waitset,
                    Err(e) => return e.into_c_int(),
                };

                alloc_memory();

                (*struct_ptr).init(
                    iox2_service_type_e::LOCAL,
                    WaitSetUnion::new_local(waitset),
                    deleter,
                );
            }
        }

        *handle_ptr = (*struct_ptr).as_handle();
    }
    IOX2_OK
}

/// Sets the [`iox2_signal_handling_mode_e`] for the [`iox2_waitset_h`].
///
/// # Arguments
///
///  * `waitset_builder_handle` must be acquired with [`iox2_node_waitset_builder()`](crate::iox2_node_waitset_builder)
///
/// # Safety
///
/// * `waitset_builder_handle` must be a valid handle
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_waitset_builder_set_signal_handling_mode(
    handle: iox2_waitset_builder_h_ref,
    signal_handling_mode: iox2_signal_handling_mode_e,
) {
    handle.assert_non_null();
    unsafe {
        let builder_struct = &mut *handle.as_type();

        match builder_struct.service_type {
            iox2_service_type_e::IPC => {
                let builder = ManuallyDrop::take(&mut builder_struct.value.as_mut().ipc);
                builder_struct.set(WaitSetBuilderUnion::new_ipc(
                    builder.signal_handling_mode(signal_handling_mode.into()),
                ));
            }
            iox2_service_type_e::LOCAL => {
                let builder = ManuallyDrop::take(&mut builder_struct.value.as_mut().local);
                builder_struct.set(WaitSetBuilderUnion::new_local(
                    builder.signal_handling_mode(signal_handling_mode.into()),
                ));
            }
        };
    }
}

// END C API
