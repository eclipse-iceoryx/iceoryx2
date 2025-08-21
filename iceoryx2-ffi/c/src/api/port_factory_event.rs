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
    iox2_port_factory_listener_builder_h, iox2_port_factory_listener_builder_t,
    iox2_port_factory_notifier_builder_h, iox2_port_factory_notifier_builder_t,
    iox2_service_name_ptr, iox2_service_type_e, AssertNonNullHandle, HandleToType, IntoCInt,
    PortFactoryListenerBuilderUnion, PortFactoryNotifierBuilderUnion,
};
use crate::{iox2_node_list_impl, IOX2_OK};

use iceoryx2::service::dynamic_config::event::{ListenerDetails, NotifierDetails};
use iceoryx2::service::port_factory::{event::PortFactory as PortFactoryEvent, PortFactory};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::ffi::{c_char, c_int};
use core::mem::ManuallyDrop;

use super::{
    iox2_attribute_set_ptr, iox2_callback_context, iox2_callback_progression_e,
    iox2_listener_details_ptr, iox2_node_list_callback, iox2_notifier_details_ptr,
    iox2_static_config_event_t,
};

// BEGIN types definition

pub(super) union PortFactoryEventUnion {
    ipc: ManuallyDrop<PortFactoryEvent<crate::IpcService>>,
    local: ManuallyDrop<PortFactoryEvent<crate::LocalService>>,
}

impl PortFactoryEventUnion {
    pub(super) fn new_ipc(port_factory: PortFactoryEvent<crate::IpcService>) -> Self {
        Self {
            ipc: ManuallyDrop::new(port_factory),
        }
    }
    pub(super) fn new_local(port_factory: PortFactoryEvent<crate::LocalService>) -> Self {
        Self {
            local: ManuallyDrop::new(port_factory),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<PortFactoryEventUnion>
pub struct iox2_port_factory_event_storage_t {
    internal: [u8; 1656], // magic number obtained with size_of::<Option<PortFactoryEventUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PortFactoryEventUnion)]
pub struct iox2_port_factory_event_t {
    service_type: iox2_service_type_e,
    value: iox2_port_factory_event_storage_t,
    deleter: fn(*mut iox2_port_factory_event_t),
}

impl iox2_port_factory_event_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PortFactoryEventUnion,
        deleter: fn(*mut iox2_port_factory_event_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_port_factory_event_h_t;
/// The owning handle for `iox2_port_factory_event_t`. Passing the handle to an function transfers the ownership.
pub type iox2_port_factory_event_h = *mut iox2_port_factory_event_h_t;
/// The non-owning handle for `iox2_port_factory_event_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_port_factory_event_h_ref = *const iox2_port_factory_event_h;

impl AssertNonNullHandle for iox2_port_factory_event_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_port_factory_event_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_port_factory_event_h {
    type Target = *mut iox2_port_factory_event_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_port_factory_event_h_ref {
    type Target = *mut iox2_port_factory_event_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

/// The callback for [`iox2_port_factory_event_dynamic_config_list_notifiers()`]
///
/// # Arguments
///
/// * [`iox2_callback_context`] -> provided by the user and can be `NULL`
/// * [`iox2_notifier_details_ptr`] -> a pointer to the details struct of the port
///
/// Returns a [`iox2_callback_progression_e`](crate::iox2_callback_progression_e)
pub type iox2_list_notifiers_callback =
    extern "C" fn(iox2_callback_context, iox2_notifier_details_ptr) -> iox2_callback_progression_e;

/// The callback for [`iox2_port_factory_event_dynamic_config_list_listeners()`]
///
/// # Arguments
///
/// * [`iox2_callback_context`] -> provided by the user and can be `NULL`
/// * [`iox2_listener_details_ptr`] -> a pointer to the details struct of the port
///
/// Returns a [`iox2_callback_progression_e`](crate::iox2_callback_progression_e)
pub type iox2_list_listeners_callback =
    extern "C" fn(iox2_callback_context, iox2_listener_details_ptr) -> iox2_callback_progression_e;
// END type definition

// BEGIN C API

/// Returns the [`iox2_service_name_ptr`], an immutable pointer to the service name.
///
/// # Safety
///
/// * The `_handle` must be valid and obtained by [`iox2_service_builder_event_open`](crate::iox2_service_builder_event_open) or
///   [`iox2_service_builder_event_open_or_create`](crate::iox2_service_builder_event_open_or_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_event_service_name(
    port_factory_handle: iox2_port_factory_event_h_ref,
) -> iox2_service_name_ptr {
    port_factory_handle.assert_non_null();

    let port_factory = &mut *port_factory_handle.as_type();

    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory.value.as_ref().ipc.name(),
        iox2_service_type_e::LOCAL => port_factory.value.as_ref().local.name(),
    }
}

/// Set the values in the provided [`iox2_static_config_event_t`] pointer.
///
/// # Safety
///
/// * The `_handle` must be valid and obtained by [`iox2_service_builder_event_open`](crate::iox2_service_builder_event_open) or
///   [`iox2_service_builder_event_open_or_create`](crate::iox2_service_builder_event_open_or_create)!
/// * The `static_config` must be a valid pointer and non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_event_static_config(
    port_factory_handle: iox2_port_factory_event_h_ref,
    static_config: *mut iox2_static_config_event_t,
) {
    port_factory_handle.assert_non_null();
    debug_assert!(!static_config.is_null());

    let port_factory = &mut *port_factory_handle.as_type();

    let config = match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory.value.as_ref().ipc.static_config(),
        iox2_service_type_e::LOCAL => port_factory.value.as_ref().local.static_config(),
    };

    *static_config = config.into();
}

// TODO [#210] add all the other setter methods

/// Instantiates a [`iox2_port_factory_notifier_builder_h`] to build a notifier.
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_event_h_ref`] obtained
///   by e.g. [`iox2_service_builder_event_open_or_create`](crate::iox2_service_builder_event_open_or_create)
/// * `notifier_builder_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_port_factory_notifier_builder_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
///
/// Returns the `iox2_port_factory_notifier_builder_h` handle for the notifier builder.
///
/// # Safety
///
/// * The `port_factory_handle` is still valid after the return of this function and can be use in another function call.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_event_notifier_builder(
    port_factory_handle: iox2_port_factory_event_h_ref,
    notifier_builder_struct_ptr: *mut iox2_port_factory_notifier_builder_t,
) -> iox2_port_factory_notifier_builder_h {
    port_factory_handle.assert_non_null();

    let mut notifier_builder_struct_ptr = notifier_builder_struct_ptr;
    fn no_op(_: *mut iox2_port_factory_notifier_builder_t) {}
    let mut deleter: fn(*mut iox2_port_factory_notifier_builder_t) = no_op;
    if notifier_builder_struct_ptr.is_null() {
        notifier_builder_struct_ptr = iox2_port_factory_notifier_builder_t::alloc();
        deleter = iox2_port_factory_notifier_builder_t::dealloc;
    }
    debug_assert!(!notifier_builder_struct_ptr.is_null());

    let port_factory = &mut *port_factory_handle.as_type();
    match port_factory.service_type {
        iox2_service_type_e::IPC => {
            let notifier_builder = port_factory.value.as_ref().ipc.notifier_builder();
            (*notifier_builder_struct_ptr).init(
                port_factory.service_type,
                PortFactoryNotifierBuilderUnion::new_ipc(notifier_builder),
                deleter,
            );
        }
        iox2_service_type_e::LOCAL => {
            let notifier_builder = port_factory.value.as_ref().local.notifier_builder();
            (*notifier_builder_struct_ptr).init(
                port_factory.service_type,
                PortFactoryNotifierBuilderUnion::new_local(notifier_builder),
                deleter,
            );
        }
    };

    (*notifier_builder_struct_ptr).as_handle()
}

/// Instantiates a [`iox2_port_factory_listener_builder_h`] to build a listener.
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_event_h_ref`] obtained
///   by e.g. [`iox2_service_builder_event_open_or_create`](crate::iox2_service_builder_event_open_or_create)
/// * `listener_builder_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_port_factory_listener_builder_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
///
/// Returns the [`iox2_port_factory_listener_builder_h`] handle for the listener builder.
///
/// # Safety
///
/// * The `port_factory_handle` is still valid after the return of this function and can be use in another function call.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_event_listener_builder(
    port_factory_handle: iox2_port_factory_event_h_ref,
    listener_builder_struct_ptr: *mut iox2_port_factory_listener_builder_t,
) -> iox2_port_factory_listener_builder_h {
    port_factory_handle.assert_non_null();

    let mut listener_builder_struct_ptr = listener_builder_struct_ptr;
    fn no_op(_: *mut iox2_port_factory_listener_builder_t) {}
    let mut deleter: fn(*mut iox2_port_factory_listener_builder_t) = no_op;
    if listener_builder_struct_ptr.is_null() {
        listener_builder_struct_ptr = iox2_port_factory_listener_builder_t::alloc();
        deleter = iox2_port_factory_listener_builder_t::dealloc;
    }
    debug_assert!(!listener_builder_struct_ptr.is_null());

    let port_factory = &mut *port_factory_handle.as_type();
    match port_factory.service_type {
        iox2_service_type_e::IPC => {
            let listener_builder = port_factory.value.as_ref().ipc.listener_builder();
            (*listener_builder_struct_ptr).init(
                port_factory.service_type,
                PortFactoryListenerBuilderUnion::new_ipc(listener_builder),
                deleter,
            );
        }
        iox2_service_type_e::LOCAL => {
            let listener_builder = port_factory.value.as_ref().local.listener_builder();
            (*listener_builder_struct_ptr).init(
                port_factory.service_type,
                PortFactoryListenerBuilderUnion::new_local(listener_builder),
                deleter,
            );
        }
    };

    (*listener_builder_struct_ptr).as_handle()
}

/// Returnes the services attributes.
///
/// # Safety
///
/// * The `port_factory_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The `port_factory_handle` must live longer than the returned `iox2_attribute_set_h_ref`.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_event_attributes(
    port_factory_handle: iox2_port_factory_event_h_ref,
) -> iox2_attribute_set_ptr {
    use iceoryx2::prelude::PortFactory;

    port_factory_handle.assert_non_null();

    let port_factory = &mut *port_factory_handle.as_type();
    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory.value.as_ref().ipc.attributes(),
        iox2_service_type_e::LOCAL => port_factory.value.as_ref().local.attributes(),
    }
}

/// Returns how many listener ports are currently connected.
///
/// # Safety
///
/// * The `handle` must be valid and obtained by [`iox2_service_builder_event_open`](crate::iox2_service_builder_event_open) or
///   [`iox2_service_builder_event_open_or_create`](crate::iox2_service_builder_event_open_or_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_event_dynamic_config_number_of_listeners(
    handle: iox2_port_factory_event_h_ref,
) -> usize {
    use iceoryx2::prelude::PortFactory;

    handle.assert_non_null();

    let port_factory = &mut *handle.as_type();
    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory
            .value
            .as_ref()
            .ipc
            .dynamic_config()
            .number_of_listeners(),
        iox2_service_type_e::LOCAL => port_factory
            .value
            .as_ref()
            .local
            .dynamic_config()
            .number_of_listeners(),
    }
}

/// Returns how many notifier ports are currently connected.
///
/// # Safety
///
/// * The `handle` must be valid and obtained by [`iox2_service_builder_event_open`](crate::iox2_service_builder_event_open) or
///   [`iox2_service_builder_event_open_or_create`](crate::iox2_service_builder_event_open_or_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_event_dynamic_config_number_of_notifiers(
    handle: iox2_port_factory_event_h_ref,
) -> usize {
    use iceoryx2::prelude::PortFactory;

    handle.assert_non_null();

    let port_factory = &mut *handle.as_type();
    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory
            .value
            .as_ref()
            .ipc
            .dynamic_config()
            .number_of_notifiers(),
        iox2_service_type_e::LOCAL => port_factory
            .value
            .as_ref()
            .local
            .dynamic_config()
            .number_of_notifiers(),
    }
}

/// Stores the service id in the provided buffer
///
/// # Safety
///
/// * The `handle` must be valid and obtained by [`iox2_service_builder_event_open`](crate::iox2_service_builder_event_open) or
///   [`iox2_service_builder_event_open_or_create`](crate::iox2_service_builder_event_open_or_create)!
/// * `buffer` must be non-zero and point to a valid memory location
/// * `buffer_len` must define the actual size of the memory location `buffer` is pointing to
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_event_service_id(
    handle: iox2_port_factory_event_h_ref,
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

/// Calls the callback repeatedly with an [`iox2_node_state_e`](crate::api::iox2_node_state_e),
/// [`iox2_node_id_ptr`](crate::api::iox2_node_id_ptr),
/// [´iox2_node_name_ptr´](crate::api::iox2_node_name_ptr) and
/// [`iox2_config_ptr`](crate::api::iox2_config_ptr) for all [`Node`](iceoryx2::node::Node)s that
/// have opened the service.
///
/// Returns IOX2_OK on success, an
/// [`iox2_node_list_failure_e`](crate::api::iox2_node_list_failure_e) otherwise.
///
/// # Safety
///
/// * The `handle` must be valid and obtained by [`iox2_service_builder_pub_sub_open`](crate::iox2_service_builder_pub_sub_open) or
///   [`iox2_service_builder_pub_sub_open_or_create`](crate::iox2_service_builder_pub_sub_open_or_create)!
/// * `callback` - A valid callback with [`iox2_node_list_callback`] signature
/// * `callback_ctx` - An optional callback context [`iox2_callback_context`] to e.g. store information across callback iterations
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_event_nodes(
    handle: iox2_port_factory_event_h_ref,
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

/// Calls the callback repeatedly for every connected [`iox2_listener_h`](crate::iox2_listener_h)
/// and provides all communcation details with a [`iox2_listener_details_ptr`].
///
/// # Safety
///
/// * [`iox2_listener_details_ptr`] - Provides a view to the listener details. Data must not be
///   accessed outside of the callback.
/// * `callback` - A valid callback with [`iox2_list_listeners_callback`] signature
/// * `callback_ctx` - An optional callback context [`iox2_callback_context`] to e.g. store
///   information across callback iterations. Must be either `NULL` or point to a valid memory
///   location
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_event_dynamic_config_list_listeners(
    handle: iox2_port_factory_event_h_ref,
    callback: iox2_list_listeners_callback,
    callback_ctx: iox2_callback_context,
) {
    handle.assert_non_null();
    use iceoryx2::prelude::PortFactory;

    let port_factory = &mut *handle.as_type();
    let callback_tr = |listener: &ListenerDetails| callback(callback_ctx, listener).into();
    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory
            .value
            .as_ref()
            .ipc
            .dynamic_config()
            .list_listeners(callback_tr),
        iox2_service_type_e::LOCAL => port_factory
            .value
            .as_ref()
            .local
            .dynamic_config()
            .list_listeners(callback_tr),
    };
}

/// Calls the callback repeatedly for every connected [`iox2_notifier_h`](crate::iox2_notifier_h)
/// and provides all communcation details with a [`iox2_notifier_details_ptr`].
///
/// # Safety
///
/// * [`iox2_notifier_details_ptr`] - Provides a view to the notifier details. Data must not be
///   accessed outside of the callback.
/// * `callback` - A valid callback with [`iox2_list_notifiers_callback`] signature
/// * `callback_ctx` - An optional callback context [`iox2_callback_context`] to e.g. store
///   information across callback iterations. Must be either `NULL` or point to a valid memory
///   location
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_event_dynamic_config_list_notifiers(
    handle: iox2_port_factory_event_h_ref,
    callback: iox2_list_notifiers_callback,
    callback_ctx: iox2_callback_context,
) {
    handle.assert_non_null();
    use iceoryx2::prelude::PortFactory;

    let port_factory = &mut *handle.as_type();
    let callback_tr = |notifier: &NotifierDetails| callback(callback_ctx, notifier).into();
    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory
            .value
            .as_ref()
            .ipc
            .dynamic_config()
            .list_notifiers(callback_tr),
        iox2_service_type_e::LOCAL => port_factory
            .value
            .as_ref()
            .local
            .dynamic_config()
            .list_notifiers(callback_tr),
    };
}

/// This function needs to be called to destroy the port factory!
///
/// # Arguments
///
/// * `port_factory_handle` - A valid [`iox2_port_factory_event_h`]
///
/// # Safety
///
/// * The `port_factory_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_port_factory_event_t`] can be re-used with a call to
///   [`iox2_service_builder_event_open_or_create`](crate::iox2_service_builder_event_open_or_create) or
///   [`iox2_service_builder_event_open`](crate::iox2_service_builder_event_open)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_event_drop(
    port_factory_handle: iox2_port_factory_event_h,
) {
    port_factory_handle.assert_non_null();

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
