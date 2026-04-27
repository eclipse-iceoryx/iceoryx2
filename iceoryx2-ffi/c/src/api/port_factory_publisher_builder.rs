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
    AssertNonNullHandle, HandleToType, IOX2_OK, IntoCInt, PayloadFfi, PublisherUnion,
    UnsafeCallbackContextSendWorkaround, UserHeaderFfi, c_size_t, degradation_info_cast,
    iox2_callback_context, iox2_degradation_handler, iox2_publisher_h, iox2_publisher_t,
    iox2_service_type_e, iox2_unable_to_deliver_handler, unable_to_deliver_info_cast,
};

use iceoryx2::port::publisher::PublisherCreateError;
use iceoryx2::prelude::*;
use iceoryx2::service::port_factory::publisher::PortFactoryPublisher;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::CStrRepr;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::ffi::{c_char, c_int};
use core::mem::ManuallyDrop;

// BEGIN types definition

/// Describes generically an allocation strategy, meaning how the memory is increased when the
/// available memory is insufficient.
#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_allocation_strategy_e {
    /// Increases the memory so that it perfectly fits the new size requirements. This may lead
    /// to a lot of reallocations but has the benefit that no byte is wasted.
    BEST_FIT,
    /// Increases the memory by rounding the increased memory size up to the next power of two.
    /// Reduces reallocations a lot at the cost of increased memory usage.
    POWER_OF_TWO,
    /// The memory is not increased. This may lead to an out-of-memory error when allocating.
    STATIC,
}

impl From<iox2_allocation_strategy_e> for AllocationStrategy {
    fn from(value: iox2_allocation_strategy_e) -> Self {
        match value {
            iox2_allocation_strategy_e::STATIC => AllocationStrategy::Static,
            iox2_allocation_strategy_e::BEST_FIT => AllocationStrategy::BestFit,
            iox2_allocation_strategy_e::POWER_OF_TWO => AllocationStrategy::PowerOfTwo,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_unable_to_deliver_strategy_e {
    RETRY_UNTIL_DELIVERED,
    DISCARD_DATA,
}

impl From<iox2_unable_to_deliver_strategy_e> for UnableToDeliverStrategy {
    fn from(value: iox2_unable_to_deliver_strategy_e) -> Self {
        match value {
            iox2_unable_to_deliver_strategy_e::RETRY_UNTIL_DELIVERED => {
                UnableToDeliverStrategy::RetryUntilDelivered
            }
            iox2_unable_to_deliver_strategy_e::DISCARD_DATA => UnableToDeliverStrategy::DiscardData,
        }
    }
}

impl From<UnableToDeliverStrategy> for iox2_unable_to_deliver_strategy_e {
    fn from(value: UnableToDeliverStrategy) -> Self {
        match value {
            UnableToDeliverStrategy::RetryUntilDelivered => {
                iox2_unable_to_deliver_strategy_e::RETRY_UNTIL_DELIVERED
            }
            UnableToDeliverStrategy::DiscardData => iox2_unable_to_deliver_strategy_e::DISCARD_DATA,
        }
    }
}

impl IntoCInt for UnableToDeliverStrategy {
    fn into_c_int(self) -> c_int {
        Into::<iox2_unable_to_deliver_strategy_e>::into(self) as c_int
    }
}

#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_publisher_create_error_e {
    EXCEEDS_MAX_SUPPORTED_PUBLISHERS = IOX2_OK as isize + 1,
    UNABLE_TO_CREATE_DATA_SEGMENT,
    FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY,
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
            PublisherCreateError::FailedToDeployThreadsafetyPolicy => {
                iox2_publisher_create_error_e::FAILED_TO_DEPLOY_THREAD_SAFETY_POLICY
            }
        }) as c_int
    }
}

pub(super) union PortFactoryPublisherBuilderUnion {
    ipc: ManuallyDrop<PortFactoryPublisher<'static, crate::IpcService, PayloadFfi, UserHeaderFfi>>,
    local:
        ManuallyDrop<PortFactoryPublisher<'static, crate::LocalService, PayloadFfi, UserHeaderFfi>>,
}

impl PortFactoryPublisherBuilderUnion {
    pub(super) fn new_ipc(
        port_factory: PortFactoryPublisher<'static, crate::IpcService, PayloadFfi, UserHeaderFfi>,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(port_factory),
        }
    }
    pub(super) fn new_local(
        port_factory: PortFactoryPublisher<'static, crate::LocalService, PayloadFfi, UserHeaderFfi>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(port_factory),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<PortFactoryPublisherBuilderUnion>
pub struct iox2_port_factory_publisher_builder_storage_t {
    internal: [u8; 208], // magic number obtained with size_of::<Option<PortFactoryPublisherBuilderUnion>>()
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
/// The non-owning handle for `iox2_port_factory_publisher_builder_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_port_factory_publisher_builder_h_ref = *const iox2_port_factory_publisher_builder_h;

impl AssertNonNullHandle for iox2_port_factory_publisher_builder_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_port_factory_publisher_builder_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_port_factory_publisher_builder_h {
    type Target = *mut iox2_port_factory_publisher_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_port_factory_publisher_builder_h_ref {
    type Target = *mut iox2_port_factory_publisher_builder_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

/// The callback for [`iox2_port_factory_publisher_builder_override_samples_preallocation`]
///
/// # Arguments
///
/// * `number_of_preallocated_samples` - the worst case number of samples that need to be
///   preallocated so that iceoryx2 can guarantee that it never runs out of memory.
/// * `iox2_callback_context` -> provided by the user and can be `NULL`
///
/// Returns the override value of preallocated samples. The return value is clamped between `1`
/// and the worst case number of preallocated samples (`number_of_preallocated_sample`).
pub type iox2_preallocated_samples_override = extern "C" fn(usize, iox2_callback_context) -> usize;

// END type definition

// BEGIN C API

/// Returns a string literal describing the provided [`iox2_publisher_create_error_e`].
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
pub unsafe extern "C" fn iox2_publisher_create_error_string(
    error: iox2_publisher_create_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// Defines a callback to reduce the number of preallocated samples.
/// The input argument is the worst case number of preallocated samples required
/// to guarantee that the publisher never runs out of samples to loan
/// and send.
/// The return value is clamped between `1` and the worst case number of
/// preallocated samples.
///
/// # Important
///
/// If the user reduces the number of preallocated samples, iceoryx2 can
/// no longer guarantee, that the publisher can always loan a sample
/// to send.
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_publisher_builder_h_ref`]
///   obtained by [`iox2_port_factory_pub_sub_publisher_builder`](crate::iox2_port_factory_pub_sub_publisher_builder).
/// * callback - the override callback
/// * callback_ctx - a context pointer provided to the override callback as input argument
///
/// # Safety
///
/// * `port_factory_handle` must be a valid handle
///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_port_factory_publisher_builder_override_samples_preallocation(
    port_factory_handle: iox2_port_factory_publisher_builder_h_ref,
    callback: iox2_preallocated_samples_override,
    callback_ctx: iox2_callback_context,
) {
    port_factory_handle.assert_non_null();
    unsafe {
        let port_factory_struct = &mut *port_factory_handle.as_type();
        match port_factory_struct.service_type {
            iox2_service_type_e::IPC => {
                let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().ipc);

                port_factory_struct.set(PortFactoryPublisherBuilderUnion::new_ipc(
                    port_factory.override_sample_preallocation(move |v| callback(v, callback_ctx)),
                ));
            }
            iox2_service_type_e::LOCAL => {
                let port_factory =
                    ManuallyDrop::take(&mut port_factory_struct.value.as_mut().local);

                port_factory_struct.set(PortFactoryPublisherBuilderUnion::new_local(
                    port_factory.override_sample_preallocation(move |v| callback(v, callback_ctx)),
                ));
            }
        }
    }
}

/// Sets the [`iox2_allocation_strategy_e`] for the publisher
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_publisher_builder_h_ref`]
///   obtained by [`iox2_port_factory_pub_sub_publisher_builder`](crate::iox2_port_factory_pub_sub_publisher_builder).
/// * `value` - The value to set max slice length to
///
/// # Safety
///
/// * `port_factory_handle` must be valid handles
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_port_factory_publisher_builder_set_allocation_strategy(
    port_factory_handle: iox2_port_factory_publisher_builder_h_ref,
    value: iox2_allocation_strategy_e,
) {
    port_factory_handle.assert_non_null();
    unsafe {
        let port_factory_struct = &mut *port_factory_handle.as_type();
        match port_factory_struct.service_type {
            iox2_service_type_e::IPC => {
                let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().ipc);

                port_factory_struct.set(PortFactoryPublisherBuilderUnion::new_ipc(
                    port_factory.allocation_strategy(value.into()),
                ));
            }
            iox2_service_type_e::LOCAL => {
                let port_factory =
                    ManuallyDrop::take(&mut port_factory_struct.value.as_mut().local);

                port_factory_struct.set(PortFactoryPublisherBuilderUnion::new_local(
                    port_factory.allocation_strategy(value.into()),
                ));
            }
        }
    }
}

/// Sets the degradation handler for the publisher
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_publisher_builder_h_ref`]
///   obtained by [`iox2_port_factory_pub_sub_publisher_builder`](crate::iox2_port_factory_pub_sub_publisher_builder).
/// * `handler` is the [`iox2_degradation_handler`](crate::iox2_degradation_handler)
/// * `ctx` is an user defined [`iox2_callback_context`](crate::iox2_callback_context)
///
/// # Safety
///
/// * `port_factory_handle` must be valid handles
/// * `ctx` is stored for later use; if the publisher, including the send function,
///   is accessed from multiple threads, the `ctx` must be thread-safe
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_port_factory_publisher_builder_set_degradation_handler(
    port_factory_handle: iox2_port_factory_publisher_builder_h_ref,
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

                port_factory_struct.set(PortFactoryPublisherBuilderUnion::new_ipc(
                    port_factory.set_degradation_handler(move |cause, info| {
                        let ctx = ctx;
                        handler(cause.into(), degradation_info_cast(info), ctx.ctx).into()
                    }),
                ));
            }
            iox2_service_type_e::LOCAL => {
                let port_factory =
                    ManuallyDrop::take(&mut port_factory_struct.value.as_mut().local);

                port_factory_struct.set(PortFactoryPublisherBuilderUnion::new_local(
                    port_factory.set_degradation_handler(move |cause, info| {
                        let ctx = ctx;
                        handler(cause.into(), degradation_info_cast(info), ctx.ctx).into()
                    }),
                ));
            }
        }
    }
}

/// Sets the unable to deliver handler for the publisher to be able to execute custom code if a sample cannot be delivered
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_publisher_builder_h_ref`]
///   obtained by [`iox2_port_factory_pub_sub_publisher_builder`](crate::iox2_port_factory_pub_sub_publisher_builder).
/// * `handler` is the [`iox2_unable_to_deliver_handler`](crate::iox2_unable_to_deliver_handler)
/// * `ctx` is an user defined [`iox2_callback_context`](crate::iox2_callback_context)
///
/// # Safety
///
/// * `port_factory_handle` must be valid handles
/// * `ctx` is stored for later use; if the publisher, including the send function,
///   is accessed from multiple threads, the `ctx` must be thread-safe
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_port_factory_publisher_builder_set_unable_to_deliver_handler(
    port_factory_handle: iox2_port_factory_publisher_builder_h_ref,
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

                port_factory_struct.set(PortFactoryPublisherBuilderUnion::new_ipc(
                    port_factory.set_unable_to_deliver_handler(move |info| {
                        let ctx = ctx;
                        handler(unable_to_deliver_info_cast(info), ctx.ctx).into()
                    }),
                ));
            }
            iox2_service_type_e::LOCAL => {
                let port_factory =
                    ManuallyDrop::take(&mut port_factory_struct.value.as_mut().local);

                port_factory_struct.set(PortFactoryPublisherBuilderUnion::new_local(
                    port_factory.set_unable_to_deliver_handler(move |info| {
                        let ctx = ctx;
                        handler(unable_to_deliver_info_cast(info), ctx.ctx).into()
                    }),
                ));
            }
        }
    }
}

/// Sets the max slice length for the publisher
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_publisher_builder_h_ref`]
///   obtained by [`iox2_port_factory_pub_sub_publisher_builder`](crate::iox2_port_factory_pub_sub_publisher_builder).
/// * `value` - The value to set max slice length to
///
/// # Safety
///
/// * `port_factory_handle` must be valid handles
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_port_factory_publisher_builder_set_initial_max_slice_len(
    port_factory_handle: iox2_port_factory_publisher_builder_h_ref,
    value: c_size_t,
) {
    port_factory_handle.assert_non_null();
    unsafe {
        let port_factory_struct = &mut *port_factory_handle.as_type();
        match port_factory_struct.service_type {
            iox2_service_type_e::IPC => {
                let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().ipc);

                port_factory_struct.set(PortFactoryPublisherBuilderUnion::new_ipc(
                    port_factory.initial_max_slice_len(value),
                ));
            }
            iox2_service_type_e::LOCAL => {
                let port_factory =
                    ManuallyDrop::take(&mut port_factory_struct.value.as_mut().local);

                port_factory_struct.set(PortFactoryPublisherBuilderUnion::new_local(
                    port_factory.initial_max_slice_len(value),
                ));
            }
        }
    }
}

/// Sets the max loaned samples for the publisher
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_publisher_builder_h_ref`]
///   obtained by [`iox2_port_factory_pub_sub_publisher_builder`](crate::iox2_port_factory_pub_sub_publisher_builder).
/// * `value` - The value to set max loaned samples to
///
/// # Safety
///
/// * `port_factory_handle` must be valid handles
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_port_factory_publisher_builder_set_max_loaned_samples(
    port_factory_handle: iox2_port_factory_publisher_builder_h_ref,
    value: c_size_t,
) {
    port_factory_handle.assert_non_null();
    unsafe {
        let port_factory_struct = &mut *port_factory_handle.as_type();
        match port_factory_struct.service_type {
            iox2_service_type_e::IPC => {
                let port_factory = ManuallyDrop::take(&mut port_factory_struct.value.as_mut().ipc);

                port_factory_struct.set(PortFactoryPublisherBuilderUnion::new_ipc(
                    port_factory.max_loaned_samples(value),
                ));
            }
            iox2_service_type_e::LOCAL => {
                let port_factory =
                    ManuallyDrop::take(&mut port_factory_struct.value.as_mut().local);

                port_factory_struct.set(PortFactoryPublisherBuilderUnion::new_local(
                    port_factory.max_loaned_samples(value),
                ));
            }
        }
    }
}

// TODO [#210] add all the other setter methods

/// Sets the unable to deliver strategy for the publisher
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_publisher_builder_h_ref`]
///   obtained by [`iox2_port_factory_pub_sub_publisher_builder`](crate::iox2_port_factory_pub_sub_publisher_builder).
/// * `value` - The value to set the strategy to
///
/// # Safety
///
/// * `port_factory_handle` must be valid handles
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_port_factory_publisher_builder_unable_to_deliver_strategy(
    port_factory_handle: iox2_port_factory_publisher_builder_h_ref,
    value: iox2_unable_to_deliver_strategy_e,
) {
    port_factory_handle.assert_non_null();
    unsafe {
        let handle = &mut *port_factory_handle.as_type();
        match handle.service_type {
            iox2_service_type_e::IPC => {
                let builder = ManuallyDrop::take(&mut handle.value.as_mut().ipc);

                handle.set(PortFactoryPublisherBuilderUnion::new_ipc(
                    builder.unable_to_deliver_strategy(value.into()),
                ));
            }
            iox2_service_type_e::LOCAL => {
                let builder = ManuallyDrop::take(&mut handle.value.as_mut().local);

                handle.set(PortFactoryPublisherBuilderUnion::new_local(
                    builder.unable_to_deliver_strategy(value.into()),
                ));
            }
        }
    }
}

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
#[unsafe(no_mangle)]
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
    unsafe {
        let publisher_builder_struct = &mut *port_factory_handle.as_type();
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
                        deleter(publisher_struct_ptr);
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
                        deleter(publisher_struct_ptr);
                        return error.into_c_int();
                    }
                }
            }
        }

        *publisher_handle_ptr = (*publisher_struct_ptr).as_handle();
    }
    IOX2_OK
}

// END C API
