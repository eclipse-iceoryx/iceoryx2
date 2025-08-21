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

use crate::api::{
    iox2_event_id_t, iox2_notifier_h, iox2_notifier_t, iox2_service_type_e, AssertNonNullHandle,
    HandleToType, IntoCInt, NotifierUnion, IOX2_OK,
};

use iceoryx2::port::notifier::NotifierCreateError;
use iceoryx2::service::port_factory::notifier::PortFactoryNotifier;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::iceoryx2_ffi;
use iceoryx2_ffi_macros::CStrRepr;

use core::ffi::{c_char, c_int};
use core::mem::ManuallyDrop;

// BEGIN types definition

#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_notifier_create_error_e {
    EXCEEDS_MAX_SUPPORTED_NOTIFIERS = IOX2_OK as isize + 1,
    FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY,
}

impl IntoCInt for NotifierCreateError {
    fn into_c_int(self) -> c_int {
        (match self {
            NotifierCreateError::ExceedsMaxSupportedNotifiers => {
                iox2_notifier_create_error_e::EXCEEDS_MAX_SUPPORTED_NOTIFIERS
            }
            NotifierCreateError::FailedToDeployThreadsafetyPolicy => {
                iox2_notifier_create_error_e::FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY
            }
        }) as c_int
    }
}

pub(super) union PortFactoryNotifierBuilderUnion {
    ipc: ManuallyDrop<PortFactoryNotifier<'static, crate::IpcService>>,
    local: ManuallyDrop<PortFactoryNotifier<'static, crate::LocalService>>,
}

impl PortFactoryNotifierBuilderUnion {
    pub(super) fn new_ipc(port_factory: PortFactoryNotifier<'static, crate::IpcService>) -> Self {
        Self {
            ipc: ManuallyDrop::new(port_factory),
        }
    }
    pub(super) fn new_local(
        port_factory: PortFactoryNotifier<'static, crate::LocalService>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(port_factory),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<PortFactoryNotifierBuilderUnion>
pub struct iox2_port_factory_notifier_builder_storage_t {
    internal: [u8; 24], // magic number obtained with size_of::<Option<PortFactoryNotifierBuilderUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PortFactoryNotifierBuilderUnion)]
pub struct iox2_port_factory_notifier_builder_t {
    service_type: iox2_service_type_e,
    value: iox2_port_factory_notifier_builder_storage_t,
    deleter: fn(*mut iox2_port_factory_notifier_builder_t),
}

impl iox2_port_factory_notifier_builder_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PortFactoryNotifierBuilderUnion,
        deleter: fn(*mut iox2_port_factory_notifier_builder_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_port_factory_notifier_builder_h_t;
/// The owning handle for `iox2_port_factory_notifier_builder_t`. Passing the handle to an function transfers the ownership.
pub type iox2_port_factory_notifier_builder_h = *mut iox2_port_factory_notifier_builder_h_t;
/// The non-owning handle for `iox2_port_factory_notifier_builder_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_port_factory_notifier_builder_h_ref = *const iox2_port_factory_notifier_builder_h;

impl AssertNonNullHandle for iox2_port_factory_notifier_builder_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_port_factory_notifier_builder_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_port_factory_notifier_builder_h {
    type Target = *mut iox2_port_factory_notifier_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_port_factory_notifier_builder_h_ref {
    type Target = *mut iox2_port_factory_notifier_builder_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

/// Returns a string literal describing the provided [`iox2_notifier_create_error_e`].
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
#[no_mangle]
pub unsafe extern "C" fn iox2_notifier_create_error_string(
    error: iox2_notifier_create_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// Sets the default event id for the builder
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_notifier_builder_h_ref`]
///   obtained by [`iox2_port_factory_event_notifier_builder`](crate::iox2_port_factory_event_notifier_builder).
/// * `value` - The value to set the default event id to
///
/// # Safety
///
/// * `port_factory_handle` must be valid handles
/// * `value` must not be a NULL pointer but a pointer to an initialized `iox2_event_id_t`
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_notifier_builder_set_default_event_id(
    port_factory_handle: iox2_port_factory_notifier_builder_h_ref,
    value: *const iox2_event_id_t,
) {
    port_factory_handle.assert_non_null();

    let value = (*value).into();

    let port_factory_struct = unsafe { &mut *port_factory_handle.as_type() };
    match port_factory_struct.service_type {
        iox2_service_type_e::IPC => {
            let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().ipc);

            port_factory_struct.set(PortFactoryNotifierBuilderUnion::new_ipc(
                port_factory.default_event_id(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().local);

            port_factory_struct.set(PortFactoryNotifierBuilderUnion::new_local(
                port_factory.default_event_id(value),
            ));
        }
    }
}

/// Creates a notifier and consumes the builder
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_notifier_builder_h`] obtained by [`iox2_port_factory_event_notifier_builder`](crate::iox2_port_factory_event_notifier_builder).
/// * `notifier_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_notifier_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `notifier_handle_ptr` - An uninitialized or dangling [`iox2_notifier_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_notifier_create_error_e`] otherwise.
///
/// # Safety
///
/// * The `port_factory_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_port_factory_notifier_builder_t`]
///   can be re-used with a call to  [`iox2_port_factory_event_notifier_builder`](crate::iox2_port_factory_event_notifier_builder)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_notifier_builder_create(
    port_factory_handle: iox2_port_factory_notifier_builder_h,
    notifier_struct_ptr: *mut iox2_notifier_t,
    notifier_handle_ptr: *mut iox2_notifier_h,
) -> c_int {
    debug_assert!(!port_factory_handle.is_null());
    debug_assert!(!notifier_handle_ptr.is_null());

    let mut notifier_struct_ptr = notifier_struct_ptr;
    fn no_op(_: *mut iox2_notifier_t) {}
    let mut deleter: fn(*mut iox2_notifier_t) = no_op;
    if notifier_struct_ptr.is_null() {
        notifier_struct_ptr = iox2_notifier_t::alloc();
        deleter = iox2_notifier_t::dealloc;
    }
    debug_assert!(!notifier_struct_ptr.is_null());

    let notifier_builder_struct = unsafe { &mut *port_factory_handle.as_type() };
    let service_type = notifier_builder_struct.service_type;
    let notifier_builder = notifier_builder_struct
        .value
        .as_option_mut()
        .take()
        .unwrap_or_else(|| {
            panic!("Trying to use an invalid 'iox2_port_factory_notifier_builder_h'!")
        });
    (notifier_builder_struct.deleter)(notifier_builder_struct);

    match service_type {
        iox2_service_type_e::IPC => {
            let notifier_builder = ManuallyDrop::into_inner(notifier_builder.ipc);

            match notifier_builder.create() {
                Ok(notifier) => {
                    (*notifier_struct_ptr).init(
                        service_type,
                        NotifierUnion::new_ipc(notifier),
                        deleter,
                    );
                }
                Err(error) => {
                    deleter(notifier_struct_ptr);
                    return error.into_c_int();
                }
            }
        }
        iox2_service_type_e::LOCAL => {
            let notifier_builder = ManuallyDrop::into_inner(notifier_builder.local);

            match notifier_builder.create() {
                Ok(notifier) => {
                    (*notifier_struct_ptr).init(
                        service_type,
                        NotifierUnion::new_local(notifier),
                        deleter,
                    );
                }
                Err(error) => {
                    deleter(notifier_struct_ptr);
                    return error.into_c_int();
                }
            }
        }
    }

    *notifier_handle_ptr = (*notifier_struct_ptr).as_handle();

    IOX2_OK
}

// END C API
