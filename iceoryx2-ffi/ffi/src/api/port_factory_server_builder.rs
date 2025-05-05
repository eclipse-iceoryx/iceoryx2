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

#![allow(non_camel_case_types)]

use core::mem::ManuallyDrop;

use crate::api::ServerUnion;
use crate::IOX2_OK;

use super::{
    c_size_t, iox2_allocation_strategy_e, iox2_server_h, iox2_server_t, iox2_service_type_e,
    iox2_unable_to_deliver_strategy_e, IntoCInt, PayloadFfi, UserHeaderFfi,
};
use super::{AssertNonNullHandle, HandleToType};
use core::ffi::c_int;
use iceoryx2::prelude::*;
use iceoryx2::service::port_factory::server::{PortFactoryServer, ServerCreateError};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_elementary::AsCStr;
use iceoryx2_ffi_macros::{iceoryx2_ffi, CStrRepr};

// BEGIN types definition
#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_server_create_error_e {
    EXCEEDS_MAX_SUPPORTED_SERVERS = IOX2_OK as isize + 1,
    UNABLE_TO_CREATE_DATA_SEGMENT,
}

impl IntoCInt for ServerCreateError {
    fn into_c_int(self) -> c_int {
        (match self {
            ServerCreateError::UnableToCreateDataSegment => {
                iox2_server_create_error_e::UNABLE_TO_CREATE_DATA_SEGMENT
            }
            ServerCreateError::ExceedsMaxSupportedServers => {
                iox2_server_create_error_e::EXCEEDS_MAX_SUPPORTED_SERVERS
            }
        }) as c_int
    }
}

pub(super) union PortFactoryServerBuilderUnion {
    ipc: ManuallyDrop<
        PortFactoryServer<
            'static,
            ipc::Service,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    >,
    local: ManuallyDrop<
        PortFactoryServer<
            'static,
            local::Service,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    >,
}

impl PortFactoryServerBuilderUnion {
    pub(super) fn new_ipc(
        port_factory: PortFactoryServer<
            'static,
            ipc::Service,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(port_factory),
        }
    }
    pub(super) fn new_local(
        port_factory: PortFactoryServer<
            'static,
            local::Service,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(port_factory),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<PortFactoryServerBuilderUnion>
pub struct iox2_port_factory_server_builder_storage_t {
    internal: [u8; 176], // magic number obtained with size_of::<Option<PortFactoryServerBuilderUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PortFactoryServerBuilderUnion)]
pub struct iox2_port_factory_server_builder_t {
    service_type: iox2_service_type_e,
    value: iox2_port_factory_server_builder_storage_t,
    deleter: fn(*mut iox2_port_factory_server_builder_t),
}

impl iox2_port_factory_server_builder_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PortFactoryServerBuilderUnion,
        deleter: fn(*mut iox2_port_factory_server_builder_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_port_factory_server_builder_h_t;
/// The owning handle for `iox2_port_factory_server_builder_t`. Passing the handle to an function transfers the ownership.
pub type iox2_port_factory_server_builder_h = *mut iox2_port_factory_server_builder_h_t;
/// The non-owning handle for `iox2_port_factory_publisher_builder_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_port_factory_server_builder_h_ref = *const iox2_port_factory_server_builder_h;

impl AssertNonNullHandle for iox2_port_factory_server_builder_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_port_factory_server_builder_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_port_factory_server_builder_h {
    type Target = *mut iox2_port_factory_server_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_port_factory_server_builder_h_ref {
    type Target = *mut iox2_port_factory_server_builder_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_server_builder_set_allocation_strategy(
    port_factory_handle: iox2_port_factory_server_builder_h_ref,
    value: iox2_allocation_strategy_e,
) {
    port_factory_handle.assert_non_null();

    let port_factory_struct = unsafe { &mut *port_factory_handle.as_type() };
    match port_factory_struct.service_type {
        iox2_service_type_e::IPC => {
            let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().ipc);

            port_factory_struct.set(PortFactoryServerBuilderUnion::new_ipc(
                port_factory.allocation_strategy(value.into()),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().local);

            port_factory_struct.set(PortFactoryServerBuilderUnion::new_local(
                port_factory.allocation_strategy(value.into()),
            ));
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_server_builder_set_initial_max_slice_len(
    port_factory_handle: iox2_port_factory_server_builder_h_ref,
    value: c_size_t,
) {
    port_factory_handle.assert_non_null();

    let port_factory_struct = unsafe { &mut *port_factory_handle.as_type() };
    match port_factory_struct.service_type {
        iox2_service_type_e::IPC => {
            let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().ipc);

            port_factory_struct.set(PortFactoryServerBuilderUnion::new_ipc(
                port_factory.initial_max_slice_len(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().local);

            port_factory_struct.set(PortFactoryServerBuilderUnion::new_local(
                port_factory.initial_max_slice_len(value),
            ));
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_server_builder_set_max_loaned_responses_per_request(
    port_factory_handle: iox2_port_factory_server_builder_h_ref,
    value: c_size_t,
) {
    port_factory_handle.assert_non_null();

    let port_factory_struct = unsafe { &mut *port_factory_handle.as_type() };
    match port_factory_struct.service_type {
        iox2_service_type_e::IPC => {
            let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().ipc);

            port_factory_struct.set(PortFactoryServerBuilderUnion::new_ipc(
                port_factory.max_loaned_responses_per_request(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().local);

            port_factory_struct.set(PortFactoryServerBuilderUnion::new_local(
                port_factory.max_loaned_responses_per_request(value),
            ));
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_server_builder_unable_to_deliver_strategy(
    port_factory_handle: iox2_port_factory_server_builder_h_ref,
    value: iox2_unable_to_deliver_strategy_e,
) {
    port_factory_handle.assert_non_null();

    let handle = unsafe { &mut *port_factory_handle.as_type() };
    match handle.service_type {
        iox2_service_type_e::IPC => {
            let builder = ManuallyDrop::take(&mut handle.value.as_mut().ipc);

            handle.set(PortFactoryServerBuilderUnion::new_ipc(
                builder.unable_to_deliver_strategy(value.into()),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let builder = ManuallyDrop::take(&mut handle.value.as_mut().local);

            handle.set(PortFactoryServerBuilderUnion::new_local(
                builder.unable_to_deliver_strategy(value.into()),
            ));
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_server_builder_create(
    port_factory_handle: iox2_port_factory_server_builder_h,
    struct_ptr: *mut iox2_server_t,
    handle_ptr: *mut iox2_server_h,
) -> c_int {
    debug_assert!(!port_factory_handle.is_null());
    debug_assert!(!handle_ptr.is_null());

    let mut struct_ptr = struct_ptr;
    fn no_op(_: *mut iox2_server_t) {}
    let mut deleter: fn(*mut iox2_server_t) = no_op;
    if struct_ptr.is_null() {
        struct_ptr = iox2_server_t::alloc();
        deleter = iox2_server_t::dealloc;
    }
    debug_assert!(!struct_ptr.is_null());

    let builder_struct = unsafe { &mut *port_factory_handle.as_type() };
    let service_type = builder_struct.service_type;
    let builder = builder_struct
        .value
        .as_option_mut()
        .take()
        .unwrap_or_else(|| {
            panic!("Trying to use an invalid 'iox2_port_factory_server_builder_h'!")
        });
    (builder_struct.deleter)(builder_struct);

    match service_type {
        iox2_service_type_e::IPC => {
            let builder = ManuallyDrop::into_inner(builder.ipc);

            match builder.create() {
                Ok(server) => {
                    (*struct_ptr).init(service_type, ServerUnion::new_ipc(server), deleter);
                }
                Err(error) => {
                    return error.into_c_int();
                }
            }
        }
        iox2_service_type_e::LOCAL => {
            let builder = ManuallyDrop::into_inner(builder.local);

            match builder.create() {
                Ok(publisher) => {
                    (*struct_ptr).init(service_type, ServerUnion::new_local(publisher), deleter);
                }
                Err(error) => {
                    return error.into_c_int();
                }
            }
        }
    }

    *handle_ptr = (*struct_ptr).as_handle();

    IOX2_OK
}

// END C API
