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

use crate::{
    api::{
        iox2_port_factory_publisher_builder_h, iox2_port_factory_publisher_builder_t,
        iox2_port_factory_subscriber_builder_h, iox2_port_factory_subscriber_builder_t,
        iox2_service_type_e, iox2_static_config_publish_subscribe_t, AssertNonNullHandle,
        HandleToType, IntoCInt, PayloadFfi, PortFactoryPublisherBuilderUnion,
        PortFactorySubscriberBuilderUnion, UserHeaderFfi,
    },
    iox2_node_list_impl, IOX2_OK,
};

use iceoryx2::service::dynamic_config::publish_subscribe::SubscriberDetails;
use iceoryx2::service::{
    dynamic_config::publish_subscribe::PublisherDetails,
    port_factory::publish_subscribe::PortFactory,
};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::{
    ffi::{c_char, c_int},
    mem::ManuallyDrop,
};

use super::{
    iox2_attribute_set_ptr, iox2_callback_context, iox2_callback_progression_e,
    iox2_node_list_callback, iox2_publisher_details_ptr, iox2_service_name_ptr,
    iox2_subscriber_details_ptr,
};

// BEGIN types definition

pub(super) union PortFactoryPubSubUnion {
    ipc: ManuallyDrop<PortFactory<crate::IpcService, PayloadFfi, UserHeaderFfi>>,
    local: ManuallyDrop<PortFactory<crate::LocalService, PayloadFfi, UserHeaderFfi>>,
}

impl PortFactoryPubSubUnion {
    pub(super) fn new_ipc(
        port_factory: PortFactory<crate::IpcService, PayloadFfi, UserHeaderFfi>,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(port_factory),
        }
    }
    pub(super) fn new_local(
        port_factory: PortFactory<crate::LocalService, PayloadFfi, UserHeaderFfi>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(port_factory),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<PortFactoryPubSubUnion>
pub struct iox2_port_factory_pub_sub_storage_t {
    internal: [u8; 1656], // magic number obtained with size_of::<Option<PortFactoryPubSubUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PortFactoryPubSubUnion)]
pub struct iox2_port_factory_pub_sub_t {
    service_type: iox2_service_type_e,
    value: iox2_port_factory_pub_sub_storage_t,
    deleter: fn(*mut iox2_port_factory_pub_sub_t),
}

impl iox2_port_factory_pub_sub_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PortFactoryPubSubUnion,
        deleter: fn(*mut iox2_port_factory_pub_sub_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_port_factory_pub_sub_h_t;
/// The owning handle for `iox2_port_factory_pub_sub_t`. Passing the handle to an function transfers the ownership.
pub type iox2_port_factory_pub_sub_h = *mut iox2_port_factory_pub_sub_h_t;
/// The non-owning handle for `iox2_port_factory_pub_sub_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_port_factory_pub_sub_h_ref = *const iox2_port_factory_pub_sub_h;

impl AssertNonNullHandle for iox2_port_factory_pub_sub_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_port_factory_pub_sub_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_port_factory_pub_sub_h {
    type Target = *mut iox2_port_factory_pub_sub_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_port_factory_pub_sub_h_ref {
    type Target = *mut iox2_port_factory_pub_sub_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

/// The callback for [`iox2_port_factory_pub_sub_dynamic_config_list_subscribers()`]
///
/// # Arguments
///
/// * [`iox2_callback_context`] -> provided by the user and can be `NULL`
/// * [`iox2_subscriber_details_ptr`] -> a pointer to the details struct of the port
///
/// Returns a [`iox2_callback_progression_e`](crate::iox2_callback_progression_e)
pub type iox2_list_subscribers_callback = extern "C" fn(
    iox2_callback_context,
    iox2_subscriber_details_ptr,
) -> iox2_callback_progression_e;

/// The callback for [`iox2_port_factory_pub_sub_dynamic_config_list_publishers()`]
///
/// # Arguments
///
/// * [`iox2_callback_context`] -> provided by the user and can be `NULL`
/// * [`iox2_publisher_details_ptr`] -> a pointer to the details struct of the port
///
/// Returns a [`iox2_callback_progression_e`](crate::iox2_callback_progression_e)
pub type iox2_list_publishers_callback =
    extern "C" fn(iox2_callback_context, iox2_publisher_details_ptr) -> iox2_callback_progression_e;

// END type definition

// BEGIN C API

/// Instantiates a [`iox2_port_factory_publisher_builder_h`] to build a publisher.
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_pub_sub_h_ref`] obtained
///   by e.g. [`iox2_service_builder_pub_sub_open_or_create`](crate::iox2_service_builder_pub_sub_open_or_create).
/// * `publisher_builder_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_port_factory_publisher_builder_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
///
/// Returns the `iox2_port_factory_publisher_builder_h` handle for the publisher builder.
///
/// # Safety
///
/// * The `port_factory_handle` is still valid after the return of this function and can be use in another function call.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_pub_sub_publisher_builder(
    port_factory_handle: iox2_port_factory_pub_sub_h_ref,
    publisher_builder_struct_ptr: *mut iox2_port_factory_publisher_builder_t,
) -> iox2_port_factory_publisher_builder_h {
    port_factory_handle.assert_non_null();

    let mut publisher_builder_struct_ptr = publisher_builder_struct_ptr;
    fn no_op(_: *mut iox2_port_factory_publisher_builder_t) {}
    let mut deleter: fn(*mut iox2_port_factory_publisher_builder_t) = no_op;
    if publisher_builder_struct_ptr.is_null() {
        publisher_builder_struct_ptr = iox2_port_factory_publisher_builder_t::alloc();
        deleter = iox2_port_factory_publisher_builder_t::dealloc;
    }
    debug_assert!(!publisher_builder_struct_ptr.is_null());

    let port_factory = &mut *port_factory_handle.as_type();
    match port_factory.service_type {
        iox2_service_type_e::IPC => {
            let publisher_builder = port_factory.value.as_ref().ipc.publisher_builder();
            (*publisher_builder_struct_ptr).init(
                port_factory.service_type,
                PortFactoryPublisherBuilderUnion::new_ipc(publisher_builder),
                deleter,
            );
        }
        iox2_service_type_e::LOCAL => {
            let publisher_builder = port_factory.value.as_ref().local.publisher_builder();
            (*publisher_builder_struct_ptr).init(
                port_factory.service_type,
                PortFactoryPublisherBuilderUnion::new_local(publisher_builder),
                deleter,
            );
        }
    };

    (*publisher_builder_struct_ptr).as_handle()
}

/// Instantiates a [`iox2_port_factory_subscriber_builder_h`] to build a subscriber.
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_pub_sub_h_ref`] obtained
///   by e.g. [`iox2_service_builder_pub_sub_open_or_create`](crate::iox2_service_builder_pub_sub_open_or_create).
/// * `subscriber_builder_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_port_factory_subscriber_builder_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
///
/// Returns the [`iox2_port_factory_subscriber_builder_h`] handle for the subscriber builder.
///
/// # Safety
///
/// * The `port_factory_handle` is still valid after the return of this function and can be use in another function call.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_pub_sub_subscriber_builder(
    port_factory_handle: iox2_port_factory_pub_sub_h_ref,
    subscriber_builder_struct_ptr: *mut iox2_port_factory_subscriber_builder_t,
) -> iox2_port_factory_subscriber_builder_h {
    port_factory_handle.assert_non_null();

    let mut subscriber_builder_struct_ptr = subscriber_builder_struct_ptr;
    fn no_op(_: *mut iox2_port_factory_subscriber_builder_t) {}
    let mut deleter: fn(*mut iox2_port_factory_subscriber_builder_t) = no_op;
    if subscriber_builder_struct_ptr.is_null() {
        subscriber_builder_struct_ptr = iox2_port_factory_subscriber_builder_t::alloc();
        deleter = iox2_port_factory_subscriber_builder_t::dealloc;
    }
    debug_assert!(!subscriber_builder_struct_ptr.is_null());

    let port_factory = &mut *port_factory_handle.as_type();
    match port_factory.service_type {
        iox2_service_type_e::IPC => {
            let subscriber_builder = port_factory.value.as_ref().ipc.subscriber_builder();
            (*subscriber_builder_struct_ptr).init(
                port_factory.service_type,
                PortFactorySubscriberBuilderUnion::new_ipc(subscriber_builder),
                deleter,
            );
        }
        iox2_service_type_e::LOCAL => {
            let subscriber_builder = port_factory.value.as_ref().local.subscriber_builder();
            (*subscriber_builder_struct_ptr).init(
                port_factory.service_type,
                PortFactorySubscriberBuilderUnion::new_local(subscriber_builder),
                deleter,
            );
        }
    };

    (*subscriber_builder_struct_ptr).as_handle()
}

/// Returnes the services attributes.
///
/// # Safety
///
/// * The `port_factory_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The `port_factory_handle` must live longer than the returned `iox2_attribute_set_h_ref`.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_pub_sub_attributes(
    port_factory_handle: iox2_port_factory_pub_sub_h_ref,
) -> iox2_attribute_set_ptr {
    use iceoryx2::prelude::PortFactory;

    port_factory_handle.assert_non_null();

    let port_factory = &mut *port_factory_handle.as_type();
    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory.value.as_ref().ipc.attributes(),
        iox2_service_type_e::LOCAL => port_factory.value.as_ref().local.attributes(),
    }
}

/// Set the values in the provided [`iox2_static_config_publish_subscribe_t`] pointer.
///
/// # Safety
///
/// * The `_handle` must be valid and obtained by [`iox2_service_builder_pub_sub_open`](crate::iox2_service_builder_pub_sub_open) or
///   [`iox2_service_builder_pub_sub_open_or_create`](crate::iox2_service_builder_pub_sub_open_or_create)!
/// * The `static_config` must be a valid pointer and non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_pub_sub_static_config(
    port_factory_handle: iox2_port_factory_pub_sub_h_ref,
    static_config: *mut iox2_static_config_publish_subscribe_t,
) {
    port_factory_handle.assert_non_null();
    debug_assert!(!static_config.is_null());

    let port_factory = &mut *port_factory_handle.as_type();

    use iceoryx2::prelude::PortFactory;
    let config = match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory.value.as_ref().ipc.static_config(),
        iox2_service_type_e::LOCAL => port_factory.value.as_ref().local.static_config(),
    };

    *static_config = config.into();
}

/// Returns how many publisher ports are currently connected.
///
/// # Safety
///
/// * The `handle` must be valid and obtained by [`iox2_service_builder_pub_sub_open`](crate::iox2_service_builder_pub_sub_open) or
///   [`iox2_service_builder_pub_sub_open_or_create`](crate::iox2_service_builder_pub_sub_open_or_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_pub_sub_dynamic_config_number_of_publishers(
    handle: iox2_port_factory_pub_sub_h_ref,
) -> usize {
    handle.assert_non_null();

    let port_factory = &mut *handle.as_type();

    use iceoryx2::prelude::PortFactory;
    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory
            .value
            .as_ref()
            .ipc
            .dynamic_config()
            .number_of_publishers(),
        iox2_service_type_e::LOCAL => port_factory
            .value
            .as_ref()
            .local
            .dynamic_config()
            .number_of_publishers(),
    }
}

/// Calls the callback repeatedly with an [`iox2_node_state_e`](crate::api::iox2_node_state_e),
/// [`iox2_node_id_ptr`](crate::api::iox2_node_id_ptr),
/// [´iox2_node_name_ptr´](crate::api::iox2_node_name_ptr) and
/// [`iox2_config_ptr`](crate::api::iox2_config_ptr) for all
/// [`Node`](iceoryx2::node::Node)s that
/// have opened the service.
///
/// Returns IOX2_OK on success, an
/// [`iox2_node_list_failure_e`](crate::api::iox2_node_list_failure_e) otherwise.
///
/// # Safety
///
/// * The `handle` must be valid and obtained by [`iox2_service_builder_pub_sub_open`](crate::iox2_service_builder_pub_sub_open) or
///   [`iox2_service_builder_pub_sub_open_or_create`](crate::iox2_service_builder_pub_sub_open_or_create)!
/// * `callback` - A valid callback with [`iox2_node_list_callback`} signature
/// * `callback_ctx` - An optional callback context [`iox2_callback_context`} to e.g. store information across callback iterations
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_pub_sub_nodes(
    handle: iox2_port_factory_pub_sub_h_ref,
    callback: iox2_node_list_callback,
    callback_ctx: iox2_callback_context,
) -> c_int {
    use iceoryx2::prelude::PortFactory;

    handle.assert_non_null();

    let port_factory = &mut *handle.as_type();

    let list_result = match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory
            .value
            .as_ref()
            .ipc
            .nodes(|node_state| iox2_node_list_impl(&node_state, callback, callback_ctx)),
        iox2_service_type_e::LOCAL => port_factory
            .value
            .as_ref()
            .local
            .nodes(|node_state| iox2_node_list_impl(&node_state, callback, callback_ctx)),
    };

    match list_result {
        Ok(_) => IOX2_OK,
        Err(e) => e.into_c_int(),
    }
}

/// Returns the [`iox2_service_name_ptr`], an immutable pointer to the service name.
///
/// # Safety
///
/// * The `handle` must be valid and obtained by [`iox2_service_builder_pub_sub_open`](crate::iox2_service_builder_pub_sub_open) or
///   [`iox2_service_builder_pub_sub_open_or_create`](crate::iox2_service_builder_pub_sub_open_or_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_pub_sub_service_name(
    handle: iox2_port_factory_pub_sub_h_ref,
) -> iox2_service_name_ptr {
    use iceoryx2::prelude::PortFactory;

    handle.assert_non_null();

    let port_factory = &mut *handle.as_type();

    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory.value.as_ref().ipc.name(),
        iox2_service_type_e::LOCAL => port_factory.value.as_ref().local.name(),
    }
}

/// Stores the service id in the provided buffer
///
/// # Safety
///
/// * The `handle` must be valid and obtained by [`iox2_service_builder_pub_sub_open`](crate::iox2_service_builder_pub_sub_open) or
///   [`iox2_service_builder_pub_sub_open_or_create`](crate::iox2_service_builder_pub_sub_open_or_create)!
/// * `buffer` must be non-zero and point to a valid memory location
/// * `buffer_len` must define the actual size of the memory location `buffer` is pointing to
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_pub_sub_service_id(
    handle: iox2_port_factory_pub_sub_h_ref,
    buffer: *mut c_char,
    buffer_len: usize,
) {
    use iceoryx2::prelude::PortFactory;

    debug_assert!(!buffer.is_null());
    handle.assert_non_null();

    let port_factory = &mut *handle.as_type();
    let service_id = match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory.value.as_ref().ipc.service_id(),
        iox2_service_type_e::LOCAL => port_factory.value.as_ref().local.service_id(),
    };

    let len = buffer_len.min(service_id.as_str().len());
    core::ptr::copy_nonoverlapping(service_id.as_str().as_ptr(), buffer.cast(), len);
    buffer.add(len).write(0);
}

/// Returns how many subscriber ports are currently connected.
///
/// # Safety
///
/// * The `handle` must be valid and obtained by [`iox2_service_builder_pub_sub_open`](crate::iox2_service_builder_pub_sub_open) or
///   [`iox2_service_builder_pub_sub_open_or_create`](crate::iox2_service_builder_pub_sub_open_or_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_pub_sub_dynamic_config_number_of_subscribers(
    handle: iox2_port_factory_pub_sub_h_ref,
) -> usize {
    handle.assert_non_null();

    let port_factory = &mut *handle.as_type();

    use iceoryx2::prelude::PortFactory;
    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory
            .value
            .as_ref()
            .ipc
            .dynamic_config()
            .number_of_subscribers(),
        iox2_service_type_e::LOCAL => port_factory
            .value
            .as_ref()
            .local
            .dynamic_config()
            .number_of_subscribers(),
    }
}

/// Calls the callback repeatedly for every connected [`iox2_subscriber_h`](crate::iox2_subscriber_h)
/// and provides all communcation details with a [`iox2_subscriber_details_ptr`].
///
/// # Safety
///
/// * [`iox2_subscriber_details_ptr`] - Provides a view to the subscriber details. Data must not be
///   accessed outside of the callback.
/// * `callback` - A valid callback with [`iox2_list_subscribers_callback`] signature
/// * `callback_ctx` - An optional callback context [`iox2_callback_context`] to e.g. store
///   information across callback iterations. Must be either `NULL` or point to a valid memory
///   location
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_pub_sub_dynamic_config_list_subscribers(
    handle: iox2_port_factory_pub_sub_h_ref,
    callback: iox2_list_subscribers_callback,
    callback_ctx: iox2_callback_context,
) {
    handle.assert_non_null();
    use iceoryx2::prelude::PortFactory;

    let port_factory = &mut *handle.as_type();
    let callback_tr = |subscriber: &SubscriberDetails| callback(callback_ctx, subscriber).into();
    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory
            .value
            .as_ref()
            .ipc
            .dynamic_config()
            .list_subscribers(callback_tr),
        iox2_service_type_e::LOCAL => port_factory
            .value
            .as_ref()
            .local
            .dynamic_config()
            .list_subscribers(callback_tr),
    };
}

/// Calls the callback repeatedly for every connected [`iox2_publisher_h`](crate::iox2_publisher_h)
/// and provides all communcation details with a [`iox2_publisher_details_ptr`].
///
/// # Safety
///
/// * [`iox2_publisher_details_ptr`] - Provides a view to the publisher details. Data must not be
///   accessed outside of the callback.
/// * `callback` - A valid callback with [`iox2_list_publishers_callback`] signature
/// * `callback_ctx` - An optional callback context [`iox2_callback_context`] to e.g. store
///   information across callback iterations. Must be either `NULL` or point to a valid memory
///   location
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_pub_sub_dynamic_config_list_publishers(
    handle: iox2_port_factory_pub_sub_h_ref,
    callback: iox2_list_publishers_callback,
    callback_ctx: iox2_callback_context,
) {
    handle.assert_non_null();
    use iceoryx2::prelude::PortFactory;

    let port_factory = &mut *handle.as_type();
    let callback_tr = |publisher: &PublisherDetails| callback(callback_ctx, publisher).into();
    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory
            .value
            .as_ref()
            .ipc
            .dynamic_config()
            .list_publishers(callback_tr),
        iox2_service_type_e::LOCAL => port_factory
            .value
            .as_ref()
            .local
            .dynamic_config()
            .list_publishers(callback_tr),
    };
}

/// This function needs to be called to destroy the port factory!
///
/// # Arguments
///
/// * `port_factory_handle` - A valid [`iox2_port_factory_pub_sub_h`]
///
/// # Safety
///
/// * The `port_factory_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_port_factory_pub_sub_t`] can be re-used with a call to
///   [`iox2_service_builder_pub_sub_open_or_create`](crate::iox2_service_builder_pub_sub_open_or_create) or
///   [`iox2_service_builder_pub_sub_open`](crate::iox2_service_builder_pub_sub_open)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_pub_sub_drop(
    port_factory_handle: iox2_port_factory_pub_sub_h,
) {
    debug_assert!(!port_factory_handle.is_null());

    let port_factory = &mut *port_factory_handle.as_type();

    match port_factory.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut port_factory.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut port_factory.value.as_mut().local);
        }
    }
    (port_factory.deleter)(port_factory);
}

// END C API
