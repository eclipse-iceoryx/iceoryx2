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

#![allow(non_camel_case_types)]

use crate::{
    IOX2_OK,
    api::{AssertNonNullHandle, HandleToType, IntoCInt, iox2_service_type_e},
};
use core::ffi::c_int;
use iceoryx2::response_mut_uninit::FlatbufferMemory;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::mem::ManuallyDrop;

// BEGIN types definition

pub(super) union ResizableMemoryResponseUnion {
    ipc: ManuallyDrop<FlatbufferMemory<crate::IpcService>>,
    local: ManuallyDrop<FlatbufferMemory<crate::LocalService>>,
}

impl ResizableMemoryResponseUnion {
    pub(super) fn new_ipc(value: FlatbufferMemory<crate::IpcService>) -> Self {
        Self {
            ipc: ManuallyDrop::new(value),
        }
    }
    pub(super) fn new_local(value: FlatbufferMemory<crate::LocalService>) -> Self {
        Self {
            local: ManuallyDrop::new(value),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<ResizableMemoryPublishSubscribeUnion>
pub struct iox2_resizable_memory_response_storage_t {
    internal: [u8; 64], // magic number obtained with size_of::<Option<ResizableMemoryPublishSubscribeUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(ResizableMemoryResponseUnion)]
pub struct iox2_resizable_memory_response_t {
    service_type: iox2_service_type_e,
    value: iox2_resizable_memory_response_storage_t,
    deleter: fn(*mut iox2_resizable_memory_response_t),
}

impl iox2_resizable_memory_response_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: ResizableMemoryResponseUnion,
        deleter: fn(*mut iox2_resizable_memory_response_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_resizable_memory_response_h_t;
/// The owning handle for `iox2_resizable_memory_publish_subscribe_t`. Passing the handle to an function transfers the ownership.
pub type iox2_resizable_memory_response_h = *mut iox2_resizable_memory_response_h_t;
/// The non-owning handle for `iox2_resizable_memory_publish_subscribe_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_resizable_memory_response_h_ref = *const iox2_resizable_memory_response_h;

impl AssertNonNullHandle for iox2_resizable_memory_response_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_resizable_memory_response_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_resizable_memory_response_h {
    type Target = *mut iox2_resizable_memory_response_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_resizable_memory_response_h_ref {
    type Target = *mut iox2_resizable_memory_response_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

/// Resizes the underlying memory downwards. All contents are copied to the end
/// of the resized memory chunk. When `in_use_front != 0` those first bytes are
/// copied to the beginning of the resized memory chunk.
/// The `new_size` argument must be greater than the previous size, otherwise
/// this function will fail.
///
/// # Safety
///
/// * `handle` is valid and non-null
/// * `new_ptr` must be a valid pointer pointing to `*mut u8`
///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_resizable_memory_response_grow_downwards(
    handle: iox2_resizable_memory_response_h_ref,
    new_size: usize,
    in_use_front: usize,
    new_ptr: *mut *mut u8,
) -> c_int {
    handle.assert_non_null();
    unsafe {
        let resizable_memory = &mut *handle.as_type();

        match resizable_memory.service_type {
            iox2_service_type_e::IPC => {
                if let Err(e) = resizable_memory
                    .value
                    .as_mut()
                    .ipc
                    .grow_downwards_with_size(new_size, in_use_front)
                {
                    return e.into_c_int();
                }
                *new_ptr = resizable_memory.value.as_mut().ipc.as_mut_ptr();
            }
            iox2_service_type_e::LOCAL => {
                if let Err(e) = resizable_memory
                    .value
                    .as_mut()
                    .local
                    .grow_downwards_with_size(new_size, in_use_front)
                {
                    return e.into_c_int();
                }
                *new_ptr = resizable_memory.value.as_mut().local.as_mut_ptr();
            }
        }

        IOX2_OK
    }
}

/// Returns the current payload pointer that is managed by the resizable memory.
///
/// # Safety
///
/// * `handle` is valid and non-null
///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_resizable_memory_response_ptr(
    handle: iox2_resizable_memory_response_h_ref,
) -> *mut u8 {
    handle.assert_non_null();
    unsafe {
        let resizable_memory = &mut *handle.as_type();

        match resizable_memory.service_type {
            iox2_service_type_e::IPC => resizable_memory.value.as_mut().ipc.as_mut_ptr(),
            iox2_service_type_e::LOCAL => resizable_memory.value.as_mut().local.as_mut_ptr(),
        }
    }
}

/// Returns the current length of the memory that is managed by the resizable memory.
///
/// # Safety
///
/// * `handle` is valid and non-null
///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_resizable_memory_response_len(
    handle: iox2_resizable_memory_response_h_ref,
) -> usize {
    handle.assert_non_null();
    unsafe {
        let resizable_memory = &mut *handle.as_type();

        match resizable_memory.service_type {
            iox2_service_type_e::IPC => resizable_memory.value.as_mut().ipc.len(),
            iox2_service_type_e::LOCAL => resizable_memory.value.as_mut().local.len(),
        }
    }
}

/// Cleans up the resizable memory.
///
/// # Safety
///
/// * `handle` is valid and non-null
/// * after this call the `handle` is no longer valid
///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_resizable_memory_response_drop(
    handle: iox2_resizable_memory_response_h,
) {
    debug_assert!(!handle.is_null());
    unsafe {
        let memory = &mut *handle.as_type();

        match memory.service_type {
            iox2_service_type_e::IPC => {
                ManuallyDrop::drop(&mut memory.value.as_mut().ipc);
            }
            iox2_service_type_e::LOCAL => {
                ManuallyDrop::drop(&mut memory.value.as_mut().local);
            }
        }
        (memory.deleter)(memory);
    }
}

// END C API
