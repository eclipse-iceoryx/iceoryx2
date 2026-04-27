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

use super::IntoCInt;
use super::{AssertNonNullHandle, HandleToType};
use super::{
    PayloadFfi, UnsafeCallbackContextSendWorkaround, UserHeaderFfi, c_size_t,
    degradation_info_cast, iox2_allocation_strategy_e, iox2_callback_context, iox2_client_h,
    iox2_client_t, iox2_degradation_handler, iox2_service_type_e, iox2_unable_to_deliver_handler,
    iox2_unable_to_deliver_strategy_e, unable_to_deliver_info_cast,
};
use crate::IOX2_OK;
use crate::api::ClientUnion;
use core::ffi::{c_char, c_int};
use core::mem::ManuallyDrop;
use iceoryx2::service::port_factory::client::{ClientCreateError, PortFactoryClient};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::{CStrRepr, iceoryx2_ffi};

// BEGIN types definition
#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_client_create_error_e {
    UNABLE_TO_CREATE_DATA_SEGMENT = IOX2_OK as isize + 1,
    EXCEEDS_MAX_SUPPORTED_CLIENTS,
    FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY,
}

impl IntoCInt for ClientCreateError {
    fn into_c_int(self) -> c_int {
        (match self {
            ClientCreateError::UnableToCreateDataSegment => {
                iox2_client_create_error_e::UNABLE_TO_CREATE_DATA_SEGMENT
            }
            ClientCreateError::ExceedsMaxSupportedClients => {
                iox2_client_create_error_e::EXCEEDS_MAX_SUPPORTED_CLIENTS
            }
            ClientCreateError::FailedToDeployThreadsafetyPolicy => {
                iox2_client_create_error_e::FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY
            }
        }) as c_int
    }
}

pub(super) union PortFactoryClientBuilderUnion {
    ipc: ManuallyDrop<
        PortFactoryClient<
            'static,
            crate::IpcService,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    >,
    local: ManuallyDrop<
        PortFactoryClient<
            'static,
            crate::LocalService,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    >,
}

impl PortFactoryClientBuilderUnion {
    pub(super) fn new_ipc(
        port_factory: PortFactoryClient<
            'static,
            crate::IpcService,
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
        port_factory: PortFactoryClient<
            'static,
            crate::LocalService,
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
#[repr(align(16))] // alignment of Option<PortFactoryClientBuilderUnion>
pub struct iox2_port_factory_client_builder_storage_t {
    internal: [u8; 256], // magic number obtained with size_of::<Option<PortFactoryClientBuilderUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PortFactoryClientBuilderUnion)]
pub struct iox2_port_factory_client_builder_t {
    service_type: iox2_service_type_e,
    value: iox2_port_factory_client_builder_storage_t,
    deleter: fn(*mut iox2_port_factory_client_builder_t),
}

impl iox2_port_factory_client_builder_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PortFactoryClientBuilderUnion,
        deleter: fn(*mut iox2_port_factory_client_builder_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_port_factory_client_builder_h_t;
/// The owning handle for `iox2_port_factory_client_builder_t`. Passing the handle to a function transfers the ownership.
pub type iox2_port_factory_client_builder_h = *mut iox2_port_factory_client_builder_h_t;
/// The non-owning handle for `iox2_port_factory_client_builder_t`. Passing the handle to a function does not transfer the ownership.
pub type iox2_port_factory_client_builder_h_ref = *const iox2_port_factory_client_builder_h;

impl AssertNonNullHandle for iox2_port_factory_client_builder_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_port_factory_client_builder_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_port_factory_client_builder_h {
    type Target = *mut iox2_port_factory_client_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_port_factory_client_builder_h_ref {
    type Target = *mut iox2_port_factory_client_builder_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

/// The callback for [`iox2_port_factory_client_builder_override_requests_preallocation`]
///
/// # Arguments
///
/// * `number_of_preallocated_requests` - the worst case number of requests that need to be
///   preallocated so that iceoryx2 can guarantee that it never runs out of memory.
/// * `iox2_callback_context` -> provided by the user and can be `NULL`
///
/// Returns the override value of preallocated requests. The return value is clamped between `1`
/// and the worst case number of preallocated requests (`number_of_preallocated_requests`).
pub type iox2_preallocated_requests_override = extern "C" fn(usize, iox2_callback_context) -> usize;

// END type definition

// BEGIN C API
/// Returns a string literal describing the provided [`iox2_client_create_error_e`].
///
/// # Arguments
///
/// * `error` - The error value for which a description should be returned
///
/// # Returns
///
/// A pointer to a null-terminated string containing the error message.
/// The string is stored in the .rodata section of the binary.
///
/// # Safety
///
/// The returned pointer must not be modified or freed and is valid as long as the program runs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_client_create_error_string(
    error: iox2_client_create_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// Defines a callback to reduce the number of preallocated requests.
/// The input argument is the worst case number of preallocated requests required
/// to guarantee that the client never runs out of requests to loan
/// and send.
/// The return value is clamped between `1` and the worst case number of
/// preallocated requests.
///
/// # Important
///
/// If the user reduces the number of preallocated requests, iceoryx2 can
/// no longer guarantee, that the client can always loan a request
/// to send.
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_client_builder_h_ref`]
///   obtained by [`iox2_port_factory_request_response_client_builder`](crate::iox2_port_factory_request_response_client_builder).
/// * callback - the override callback
/// * callback_ctx - a context pointer provided to the override callback as input argument
///
/// # Safety
///
/// * `port_factory_handle` must be a valid handle
///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_port_factory_client_builder_override_requests_preallocation(
    port_factory_handle: iox2_port_factory_client_builder_h_ref,
    callback: iox2_preallocated_requests_override,
    callback_ctx: iox2_callback_context,
) {
    port_factory_handle.assert_non_null();
    unsafe {
        let port_factory_struct = &mut *port_factory_handle.as_type();
        match port_factory_struct.service_type {
            iox2_service_type_e::IPC => {
                let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().ipc);

                port_factory_struct.set(PortFactoryClientBuilderUnion::new_ipc(
                    port_factory.override_request_preallocation(move |v| callback(v, callback_ctx)),
                ));
            }
            iox2_service_type_e::LOCAL => {
                let port_factory =
                    ManuallyDrop::take(&mut port_factory_struct.value.as_mut().local);

                port_factory_struct.set(PortFactoryClientBuilderUnion::new_local(
                    port_factory.override_request_preallocation(move |v| callback(v, callback_ctx)),
                ));
            }
        }
    }
}

/// Sets the [`iox2_allocation_strategy_e`] for the client
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_client_builder_h_ref`]
///   obtained by [`iox2_port_factory_request_response_client_builder`](crate::iox2_port_factory_request_response_client_builder).
/// * `value` - The value to set the allocation strategy to
///
/// # Safety
///
/// * `port_factory_handle` must be a valid handle
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_port_factory_client_builder_set_allocation_strategy(
    port_factory_handle: iox2_port_factory_client_builder_h_ref,
    value: iox2_allocation_strategy_e,
) {
    port_factory_handle.assert_non_null();
    unsafe {
        let port_factory_struct = &mut *port_factory_handle.as_type();
        match port_factory_struct.service_type {
            iox2_service_type_e::IPC => {
                let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().ipc);

                port_factory_struct.set(PortFactoryClientBuilderUnion::new_ipc(
                    port_factory.allocation_strategy(value.into()),
                ));
            }
            iox2_service_type_e::LOCAL => {
                let port_factory =
                    ManuallyDrop::take(&mut port_factory_struct.value.as_mut().local);

                port_factory_struct.set(PortFactoryClientBuilderUnion::new_local(
                    port_factory.allocation_strategy(value.into()),
                ));
            }
        }
    }
}

/// Sets the request degradation handler for the client
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_client_builder_h_ref`]
///   obtained by [`iox2_port_factory_request_response_client_builder`](crate::iox2_port_factory_request_response_client_builder).
/// * `handler` is the [`iox2_degradation_handler`](crate::iox2_degradation_handler)
/// * `ctx` is an user defined [`iox2_callback_context`](crate::iox2_callback_context)
///
/// # Safety
///
/// * `port_factory_handle` must be a valid handle
/// * `ctx` is stored for later use; if the client, including the send and receive functions,
///   is accessed from multiple threads, the `ctx` must be thread-safe
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_port_factory_client_builder_set_request_degradation_handler(
    port_factory_handle: iox2_port_factory_client_builder_h_ref,
    handler: iox2_degradation_handler,
    ctx: iox2_callback_context,
) {
    port_factory_handle.assert_non_null();

    let ctx = UnsafeCallbackContextSendWorkaround { ctx };

    unsafe {
        let port_factory_struct = &mut *port_factory_handle.as_type();
        match port_factory_struct.service_type {
            iox2_service_type_e::IPC => {
                let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().ipc);

                port_factory_struct.set(PortFactoryClientBuilderUnion::new_ipc(
                    port_factory.set_request_degradation_handler(move |cause, info| {
                        let ctx = ctx;
                        handler(cause.into(), degradation_info_cast(info), ctx.ctx).into()
                    }),
                ));
            }
            iox2_service_type_e::LOCAL => {
                let port_factory =
                    ManuallyDrop::take(&mut port_factory_struct.value.as_mut().local);

                port_factory_struct.set(PortFactoryClientBuilderUnion::new_local(
                    port_factory.set_request_degradation_handler(move |cause, info| {
                        let ctx = ctx;
                        handler(cause.into(), degradation_info_cast(info), ctx.ctx).into()
                    }),
                ));
            }
        }
    }
}

/// Sets the response degradation handler for the client
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_client_builder_h_ref`]
///   obtained by [`iox2_port_factory_request_response_client_builder`](crate::iox2_port_factory_request_response_client_builder).
/// * `handler` is the [`iox2_degradation_handler`](crate::iox2_degradation_handler)
/// * `ctx` is an user defined [`iox2_callback_context`](crate::iox2_callback_context)
///
/// # Safety
///
/// * `port_factory_handle` must be a valid handle
/// * `ctx` is stored for later use; if the client, including the send and receive functions,
///   is accessed from multiple threads, the `ctx` must be thread-safe
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_port_factory_client_builder_set_response_degradation_handler(
    port_factory_handle: iox2_port_factory_client_builder_h_ref,
    handler: iox2_degradation_handler,
    ctx: iox2_callback_context,
) {
    port_factory_handle.assert_non_null();

    let ctx = UnsafeCallbackContextSendWorkaround { ctx };

    unsafe {
        let port_factory_struct = &mut *port_factory_handle.as_type();
        match port_factory_struct.service_type {
            iox2_service_type_e::IPC => {
                let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().ipc);

                port_factory_struct.set(PortFactoryClientBuilderUnion::new_ipc(
                    port_factory.set_response_degradation_handler(move |cause, info| {
                        let ctx = ctx;
                        handler(cause.into(), degradation_info_cast(info), ctx.ctx).into()
                    }),
                ));
            }
            iox2_service_type_e::LOCAL => {
                let port_factory =
                    ManuallyDrop::take(&mut port_factory_struct.value.as_mut().local);

                port_factory_struct.set(PortFactoryClientBuilderUnion::new_local(
                    port_factory.set_response_degradation_handler(move |cause, info| {
                        let ctx = ctx;
                        handler(cause.into(), degradation_info_cast(info), ctx.ctx).into()
                    }),
                ));
            }
        }
    }
}

/// Sets the unable to deliver handler for the client to be able to execute custom code if a request cannot be delivered
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_client_builder_h_ref`]
///   obtained by [`iox2_port_factory_request_response_client_builder`](crate::iox2_port_factory_request_response_client_builder).
/// * `handler` is the [`iox2_unable_to_deliver_handler`](crate::iox2_unable_to_deliver_handler)
/// * `ctx` is an user defined [`iox2_callback_context`](crate::iox2_callback_context)
///
/// # Safety
///
/// * `port_factory_handle` must be valid handles
/// * `ctx` is stored for later use; if the client, including the send function,
///   is accessed from multiple threads, the `ctx` must be thread-safe
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_port_factory_client_builder_set_unable_to_deliver_handler(
    port_factory_handle: iox2_port_factory_client_builder_h_ref,
    handler: iox2_unable_to_deliver_handler,
    ctx: iox2_callback_context,
) {
    port_factory_handle.assert_non_null();

    let ctx = UnsafeCallbackContextSendWorkaround { ctx };

    unsafe {
        let port_factory_struct = &mut *port_factory_handle.as_type();
        match port_factory_struct.service_type {
            iox2_service_type_e::IPC => {
                let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().ipc);

                port_factory_struct.set(PortFactoryClientBuilderUnion::new_ipc(
                    port_factory.set_unable_to_deliver_handler(move |info| {
                        let ctx = ctx;
                        handler(unable_to_deliver_info_cast(info), ctx.ctx).into()
                    }),
                ));
            }
            iox2_service_type_e::LOCAL => {
                let port_factory =
                    ManuallyDrop::take(&mut port_factory_struct.value.as_mut().local);

                port_factory_struct.set(PortFactoryClientBuilderUnion::new_local(
                    port_factory.set_unable_to_deliver_handler(move |info| {
                        let ctx = ctx;
                        handler(unable_to_deliver_info_cast(info), ctx.ctx).into()
                    }),
                ));
            }
        }
    }
}

/// Sets the max slice length for the client
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_client_builder_h_ref`]
///   obtained by [`iox2_port_factory_request_response_client_builder`](crate::iox2_port_factory_request_response_client_builder).
/// * `value` - The value to set max slice length to
///
/// # Safety
///
/// * `port_factory_handle` must be valid handles
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_port_factory_client_builder_set_initial_max_slice_len(
    port_factory_handle: iox2_port_factory_client_builder_h_ref,
    value: c_size_t,
) {
    port_factory_handle.assert_non_null();
    unsafe {
        let port_factory_struct = &mut *port_factory_handle.as_type();
        match port_factory_struct.service_type {
            iox2_service_type_e::IPC => {
                let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().ipc);

                port_factory_struct.set(PortFactoryClientBuilderUnion::new_ipc(
                    port_factory.initial_max_slice_len(value),
                ));
            }
            iox2_service_type_e::LOCAL => {
                let port_factory =
                    ManuallyDrop::take(&mut port_factory_struct.value.as_mut().local);

                port_factory_struct.set(PortFactoryClientBuilderUnion::new_local(
                    port_factory.initial_max_slice_len(value),
                ));
            }
        }
    }
}

/// Sets the unable to deliver strategy for the client
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_client_builder_h_ref`]
///   obtained by [`iox2_port_factory_request_response_client_builder`](crate::iox2_port_factory_request_response_client_builder).
/// * `value` - The value to set the unable to deliver strategy to
///
/// # Safety
///
/// * `port_factory_handle` must be valid handles
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_port_factory_client_builder_unable_to_deliver_strategy(
    port_factory_handle: iox2_port_factory_client_builder_h_ref,
    value: iox2_unable_to_deliver_strategy_e,
) {
    port_factory_handle.assert_non_null();
    unsafe {
        let handle = &mut *port_factory_handle.as_type();
        match handle.service_type {
            iox2_service_type_e::IPC => {
                let builder = ManuallyDrop::take(&mut handle.value.as_mut().ipc);

                handle.set(PortFactoryClientBuilderUnion::new_ipc(
                    builder.unable_to_deliver_strategy(value.into()),
                ));
            }
            iox2_service_type_e::LOCAL => {
                let builder = ManuallyDrop::take(&mut handle.value.as_mut().local);

                handle.set(PortFactoryClientBuilderUnion::new_local(
                    builder.unable_to_deliver_strategy(value.into()),
                ));
            }
        }
    }
}

/// Creates a client and consumes the builder
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_client_builder_h`] obtained by [`iox2_port_factory_request_response_client_builder`](crate::iox2_port_factory_request_response_client_builder).
/// * `struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_client_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `handle_ptr` - An uninitialized or dangling [`iox2_client_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_client_create_error_e`] otherwise.
///
/// # Safety
///
/// * The `port_factory_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_port_factory_client_builder_t`]
///   can be re-used with a call to  [`iox2_port_factory_request_response_client_builder`](crate::iox2_port_factory_request_response_client_builder)!
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_port_factory_client_builder_create(
    port_factory_handle: iox2_port_factory_client_builder_h,
    struct_ptr: *mut iox2_client_t,
    handle_ptr: *mut iox2_client_h,
) -> c_int {
    debug_assert!(!port_factory_handle.is_null());
    debug_assert!(!handle_ptr.is_null());

    let mut struct_ptr = struct_ptr;
    fn no_op(_: *mut iox2_client_t) {}
    let mut deleter: fn(*mut iox2_client_t) = no_op;
    if struct_ptr.is_null() {
        struct_ptr = iox2_client_t::alloc();
        deleter = iox2_client_t::dealloc;
    }
    debug_assert!(!struct_ptr.is_null());
    unsafe {
        let builder_struct = &mut *port_factory_handle.as_type();
        let service_type = builder_struct.service_type;
        let builder = builder_struct
            .value
            .as_option_mut()
            .take()
            .unwrap_or_else(|| {
                panic!("Trying to use an invalid 'iox2_port_factory_client_builder_h'!")
            });
        (builder_struct.deleter)(builder_struct);

        match service_type {
            iox2_service_type_e::IPC => {
                let builder = ManuallyDrop::into_inner(builder.ipc);

                match builder.create() {
                    Ok(client) => {
                        (*struct_ptr).init(service_type, ClientUnion::new_ipc(client), deleter);
                    }
                    Err(error) => {
                        deleter(struct_ptr);
                        return error.into_c_int();
                    }
                }
            }
            iox2_service_type_e::LOCAL => {
                let builder = ManuallyDrop::into_inner(builder.local);

                match builder.create() {
                    Ok(client) => {
                        (*struct_ptr).init(service_type, ClientUnion::new_local(client), deleter);
                    }
                    Err(error) => {
                        deleter(struct_ptr);
                        return error.into_c_int();
                    }
                }
            }
        }

        *handle_ptr = (*struct_ptr).as_handle();
    }
    IOX2_OK
}

// END C API
