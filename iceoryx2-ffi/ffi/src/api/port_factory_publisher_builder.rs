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
    c_size_t, iox2_publisher_h, iox2_publisher_t, iox2_service_type_e, HandleToType, IntoCInt,
    NoUserHeaderFfi, PayloadFfi, PublisherUnion, IOX2_OK,
};

use iceoryx2::port::publisher::PublisherCreateError;
use iceoryx2::prelude::*;
use iceoryx2::service::port_factory::publisher::PortFactoryPublisher;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::ffi::c_int;
use core::mem::ManuallyDrop;

// BEGIN types definition

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_publisher_create_error_e {
    EXCEEDS_MAX_SUPPORTED_PUBLISHERS = IOX2_OK as isize + 1,
    UNABLE_TO_CREATE_DATA_SEGMENT,
}

impl IntoCInt for PublisherCreateError {
    fn into_c_int(self) -> c_int {
        (match self {
            PublisherCreateError::ExceedsMaxSupportedPublishers => {
                iox2_publisher_create_error_e::EXCEEDS_MAX_SUPPORTED_PUBLISHERS
            }
            PublisherCreateError::UnableToCreateDataSegment => {
                iox2_publisher_create_error_e::UNABLE_TO_CREATE_DATA_SEGMENT
            }
        }) as c_int
    }
}

pub(super) union PortFactoryPublisherBuilderUnion {
    ipc: ManuallyDrop<PortFactoryPublisher<'static, ipc::Service, PayloadFfi, NoUserHeaderFfi>>,
    local: ManuallyDrop<PortFactoryPublisher<'static, local::Service, PayloadFfi, NoUserHeaderFfi>>,
}

impl PortFactoryPublisherBuilderUnion {
    pub(super) fn new_ipc(
        port_factory: PortFactoryPublisher<'static, ipc::Service, PayloadFfi, NoUserHeaderFfi>,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(port_factory),
        }
    }
    pub(super) fn new_local(
        port_factory: PortFactoryPublisher<'static, local::Service, PayloadFfi, NoUserHeaderFfi>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(port_factory),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<PortFactoryPublisherBuilderUnion>
pub struct iox2_port_factory_publisher_builder_storage_t {
    internal: [u8; 128], // magic number obtained with size_of::<Option<PortFactoryPublisherBuilderUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PortFactoryPublisherBuilderUnion)]
pub struct iox2_port_factory_publisher_builder_t {
    service_type: iox2_service_type_e,
    value: iox2_port_factory_publisher_builder_storage_t,
    deleter: fn(*mut iox2_port_factory_publisher_builder_t),
}

impl iox2_port_factory_publisher_builder_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PortFactoryPublisherBuilderUnion,
        deleter: fn(*mut iox2_port_factory_publisher_builder_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_port_factory_publisher_builder_h_t;
/// The owning handle for `iox2_port_factory_publisher_builder_t`. Passing the handle to an function transfers the ownership.
pub type iox2_port_factory_publisher_builder_h = *mut iox2_port_factory_publisher_builder_h_t;

pub struct iox2_port_factory_publisher_builder_ref_h_t;
/// The non-owning handle for `iox2_port_factory_publisher_builder_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_port_factory_publisher_builder_ref_h =
    *mut iox2_port_factory_publisher_builder_ref_h_t;

impl HandleToType for iox2_port_factory_publisher_builder_h {
    type Target = *mut iox2_port_factory_publisher_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_port_factory_publisher_builder_ref_h {
    type Target = *mut iox2_port_factory_publisher_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

// END type definition

// BEGIN C API

/// This function casts an owning [`iox2_port_factory_publisher_builder_h`] into a non-owning [`iox2_port_factory_publisher_builder_ref_h`]
///
/// # Arguments
///
/// * `port_factory_handle` obtained by [`iox2_port_factory_pub_sub_publisher_builder`](crate::iox2_port_factory_pub_sub_publisher_builder)
///
/// Returns a [`iox2_port_factory_publisher_builder_ref_h`]
///
/// # Safety
///
/// * The `port_factory_handle` must be a valid handle.
/// * The `port_factory_handle` is still valid after the call to this function.
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_port_factory_publisher_builder_ref_h(
    port_factory_handle: iox2_port_factory_publisher_builder_h,
) -> iox2_port_factory_publisher_builder_ref_h {
    debug_assert!(!port_factory_handle.is_null());

    (*port_factory_handle.as_type()).as_ref_handle() as *mut _ as _
}

/// Sets the max loaned samples for the publisher
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_publisher_builder_ref_h`]
///   obtained by [`iox2_port_factory_pub_sub_publisher_builder`](crate::iox2_port_factory_pub_sub_publisher_builder) and
///   casted by [`iox2_cast_port_factory_publisher_builder_ref_h`].
/// * `value` - The value to set max loaned samples to
///
/// # Safety
///
/// * `port_factory_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_publisher_builder_set_max_loaned_samples(
    port_factory_handle: iox2_port_factory_publisher_builder_ref_h,
    value: c_size_t,
) {
    debug_assert!(!port_factory_handle.is_null());

    let port_factory_struct = unsafe { &mut *port_factory_handle.as_type() };
    match port_factory_struct.service_type {
        iox2_service_type_e::IPC => {
            let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().ipc);

            port_factory_struct.set(PortFactoryPublisherBuilderUnion::new_ipc(
                port_factory.max_loaned_samples(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().local);

            port_factory_struct.set(PortFactoryPublisherBuilderUnion::new_local(
                port_factory.max_loaned_samples(value),
            ));
        }
    }
}

// TODO [#210] add all the other setter methods

/// Creates a publisher and consumes the builder
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_publisher_builder_h`] obtained by [`iox2_port_factory_pub_sub_publisher_builder`](crate::iox2_port_factory_pub_sub_publisher_builder).
/// * `publisher_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_publisher_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `publisher_handle_ptr` - An uninitialized or dangling [`iox2_publisher_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_publisher_create_error_e`] otherwise.
///
/// # Safety
///
/// * The `port_factory_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_port_factory_publisher_builder_t`]
///   can be re-used with a call to  [`iox2_port_factory_pub_sub_publisher_builder`](crate::iox2_port_factory_pub_sub_publisher_builder)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_publisher_builder_create(
    port_factory_handle: iox2_port_factory_publisher_builder_h,
    publisher_struct_ptr: *mut iox2_publisher_t,
    publisher_handle_ptr: *mut iox2_publisher_h,
) -> c_int {
    debug_assert!(!port_factory_handle.is_null());
    debug_assert!(!publisher_handle_ptr.is_null());

    let mut publisher_struct_ptr = publisher_struct_ptr;
    fn no_op(_: *mut iox2_publisher_t) {}
    let mut deleter: fn(*mut iox2_publisher_t) = no_op;
    if publisher_struct_ptr.is_null() {
        publisher_struct_ptr = iox2_publisher_t::alloc();
        deleter = iox2_publisher_t::dealloc;
    }
    debug_assert!(!publisher_struct_ptr.is_null());

    let publisher_builder_struct = unsafe { &mut *port_factory_handle.as_type() };
    let service_type = publisher_builder_struct.service_type;
    let publisher_builder = publisher_builder_struct
        .value
        .as_option_mut()
        .take()
        .unwrap_or_else(|| {
            panic!("Trying to use an invalid 'iox2_port_factory_publisher_builder_h'!")
        });
    (publisher_builder_struct.deleter)(publisher_builder_struct);

    match service_type {
        iox2_service_type_e::IPC => {
            let publisher_builder = ManuallyDrop::into_inner(publisher_builder.ipc);

            match publisher_builder.create() {
                Ok(publisher) => {
                    (*publisher_struct_ptr).init(
                        service_type,
                        PublisherUnion::new_ipc(publisher),
                        deleter,
                    );
                }
                Err(error) => {
                    return error.into_c_int();
                }
            }
        }
        iox2_service_type_e::LOCAL => {
            let publisher_builder = ManuallyDrop::into_inner(publisher_builder.local);

            match publisher_builder.create() {
                Ok(publisher) => {
                    (*publisher_struct_ptr).init(
                        service_type,
                        PublisherUnion::new_local(publisher),
                        deleter,
                    );
                }
                Err(error) => {
                    return error.into_c_int();
                }
            }
        }
    }

    *publisher_handle_ptr = (*publisher_struct_ptr).as_handle();

    IOX2_OK
}

// END C API
