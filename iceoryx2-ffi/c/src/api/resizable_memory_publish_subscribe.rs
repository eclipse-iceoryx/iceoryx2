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

use crate::api::{AssertNonNullHandle, HandleToType, iox2_service_type_e};
use iceoryx2::sample_mut::SampleMutSharedState;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_flatbuffers::ResizableMemory;
use iceoryx2_cal::shared_memory::ShmPointer;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::mem::ManuallyDrop;

// BEGIN types definition

pub(super) union ResizableMemoryPublishSubscribeUnion {
    ipc: ManuallyDrop<ResizableMemory<ShmPointer, SampleMutSharedState<crate::IpcService>>>,
    local: ManuallyDrop<ResizableMemory<ShmPointer, SampleMutSharedState<crate::LocalService>>>,
}

impl ResizableMemoryPublishSubscribeUnion {
    pub(super) fn new_ipc(
        value: ResizableMemory<ShmPointer, SampleMutSharedState<crate::IpcService>>,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(value),
        }
    }
    pub(super) fn new_local(
        value: ResizableMemory<ShmPointer, SampleMutSharedState<crate::LocalService>>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(value),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<NotifierUnion>
pub struct iox2_resizable_memory_publish_subscribe_storage_t {
    internal: [u8; 64], // magic number obtained with size_of::<Option<NotifierUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(ResizableMemoryPublishSubscribeUnion)]
pub struct iox2_resizable_memory_publish_subscribe_t {
    service_type: iox2_service_type_e,
    value: iox2_resizable_memory_publish_subscribe_storage_t,
    deleter: fn(*mut iox2_resizable_memory_publish_subscribe_t),
}

impl iox2_resizable_memory_publish_subscribe_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: ResizableMemoryPublishSubscribeUnion,
        deleter: fn(*mut iox2_resizable_memory_publish_subscribe_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_resizable_memory_publish_subscribe_h_t;
/// The owning handle for `iox2_resizable_memory_publish_subscribe_t`. Passing the handle to an function transfers the ownership.
pub type iox2_resizable_memory_publish_subscribe_h =
    *mut iox2_resizable_memory_publish_subscribe_h_t;
/// The non-owning handle for `iox2_notifier_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_resizable_memory_publish_subscribe_h_ref =
    *const iox2_resizable_memory_publish_subscribe_h;

impl AssertNonNullHandle for iox2_resizable_memory_publish_subscribe_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_resizable_memory_publish_subscribe_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_resizable_memory_publish_subscribe_h {
    type Target = *mut iox2_resizable_memory_publish_subscribe_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_resizable_memory_publish_subscribe_h_ref {
    type Target = *mut iox2_resizable_memory_publish_subscribe_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_resizable_memory_publish_subscribe_grow_downwards(
    handle: iox2_resizable_memory_publish_subscribe_h_ref,
    new_size: usize,
) -> *mut u8 {
    handle.assert_non_null();
    unsafe {
        let resizable_memory = &mut *handle.as_type();

        match resizable_memory.service_type {
            iox2_service_type_e::IPC => {
                resizable_memory
                    .value
                    .as_mut()
                    .ipc
                    .grow_downwards_with_size(new_size);
                resizable_memory.value.as_mut().ipc.as_mut_ptr()
            }
            iox2_service_type_e::LOCAL => {
                resizable_memory
                    .value
                    .as_mut()
                    .local
                    .grow_downwards_with_size(new_size);
                resizable_memory.value.as_mut().local.as_mut_ptr()
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_resizable_memory_publish_subscribe_ptr(
    handle: iox2_resizable_memory_publish_subscribe_h_ref,
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_resizable_memory_publish_subscribe_len(
    handle: iox2_resizable_memory_publish_subscribe_h_ref,
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_resizable_memory_publish_subscribe_drop(
    handle: iox2_resizable_memory_publish_subscribe_h,
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
