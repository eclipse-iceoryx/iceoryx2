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

use crate::api::{iox2_buffer_16_align_4_t, iox2_callback_context};

use iceoryx2::port::{UnableToDeliverAction, UnableToDeliverInfo};
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::CStrRepr;

// BEGIN types definition

/// Defines the action that shall be take when an a data cannot be delivered.
#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_unable_to_deliver_action_e {
    /// Use an action which is derived from the `UnableToDeliverStrategy`
    FOLLOW_UNABLE_TO_DELIVERY_STRATEGY,
    /// Retry to send and invoke the handler again, if sending does not succeed
    RETRY,
    /// Discard the data for the receiver which cause the incident and continue
    /// to deliver the data to the remaining receivers
    DISCARD_DATA,
    /// Discard the data for the receiver which caused the incident, continue
    /// to deliver the data to the remaining receivers;
    /// return with an error if the data was not delivered to all receivers
    DISCARD_DATA_AND_FAIL,
}

impl From<iox2_unable_to_deliver_action_e> for UnableToDeliverAction {
    fn from(value: iox2_unable_to_deliver_action_e) -> Self {
        match value {
            iox2_unable_to_deliver_action_e::FOLLOW_UNABLE_TO_DELIVERY_STRATEGY => {
                UnableToDeliverAction::FollowUnableToDeliveryStrategy
            }
            iox2_unable_to_deliver_action_e::RETRY => UnableToDeliverAction::Retry,
            iox2_unable_to_deliver_action_e::DISCARD_DATA => UnableToDeliverAction::DiscardData,
            iox2_unable_to_deliver_action_e::DISCARD_DATA_AND_FAIL => {
                UnableToDeliverAction::DiscardDataAndFail
            }
        }
    }
}

pub struct iox2_unable_to_deliver_info_h;
pub type iox2_unable_to_deliver_info_h_ref = *const iox2_unable_to_deliver_info_h;

pub(crate) fn unable_to_deliver_info_cast(
    info: &UnableToDeliverInfo,
) -> iox2_unable_to_deliver_info_h_ref {
    info as *const _ as iox2_unable_to_deliver_info_h_ref
}

fn unable_to_deliver_info_as_type<'a>(
    info: iox2_unable_to_deliver_info_h_ref,
) -> &'a UnableToDeliverInfo {
    debug_assert!(!info.is_null());
    unsafe { &*(info as *const _ as *const UnableToDeliverInfo) }
}

/// Obtains the service id, which is involved in the delivery incident
///
/// # Arguments
///
/// * `info_handle` - Must be a valid [`iox2_unable_to_deliver_info_h_ref`] provided as parameter by [`iox2_unable_to_deliver_handler`]
/// * `service_id` - Must be a pointer to a [`iox2_buffer_16_align_4_t`](crate::api::iox2_buffer_16_align_4_t)
///
/// # Safety
///
/// * `info_handle` must be a valid handle
/// * `service_id` must not be a null pointer
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_unable_to_deliver_info_service_id(
    info_handle: iox2_unable_to_deliver_info_h_ref,
    service_id: *mut iox2_buffer_16_align_4_t,
) {
    unsafe {
        iox2_buffer_16_align_4_t::write(
            service_id,
            unable_to_deliver_info_as_type(info_handle)
                .service_id
                .to_be_bytes(),
        );
    }
}

/// Obtains the receiver port id, which is involved in the delivery incident
///
/// # Arguments
///
/// * `info_handle` - Must be a valid [`iox2_unable_to_deliver_info_h_ref`] provided as parameter by [`iox2_unable_to_deliver_handler`]
/// * `service_id` - Must be a pointer to a [`iox2_buffer_16_align_4_t`](crate::api::iox2_buffer_16_align_4_t)
///
/// # Safety
///
/// * `info_handle` must be a valid handle
/// * `service_id` must not be a null pointer
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_unable_to_deliver_info_receiver_port_id(
    info_handle: iox2_unable_to_deliver_info_h_ref,
    receiver_port_id: *mut iox2_buffer_16_align_4_t,
) {
    unsafe {
        iox2_buffer_16_align_4_t::write(
            receiver_port_id,
            unable_to_deliver_info_as_type(info_handle)
                .receiver_port_id
                .to_be_bytes(),
        );
    }
}

/// Obtains the sender port id, which is involved in the delivery incident
///
/// # Arguments
///
/// * `info_handle` - Must be a valid [`iox2_unable_to_deliver_info_h_ref`] provided as parameter by [`iox2_unable_to_deliver_handler`]
/// * `service_id` - Must be a pointer to a [`iox2_buffer_16_align_4_t`](crate::api::iox2_buffer_16_align_4_t)
///
/// # Safety
///
/// * `info_handle` must be a valid handle
/// * `service_id` must not be a null pointer
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_unable_to_deliver_info_sender_port_id(
    info_handle: iox2_unable_to_deliver_info_h_ref,
    sender_port_id: *mut iox2_buffer_16_align_4_t,
) {
    unsafe {
        iox2_buffer_16_align_4_t::write(
            sender_port_id,
            unable_to_deliver_info_as_type(info_handle)
                .sender_port_id
                .to_be_bytes(),
        );
    }
}

/// Obtains the number of retries for the running delivery to the receiver port
///
/// # Arguments
///
/// * `info_handle` - Must be a valid [`iox2_unable_to_deliver_info_h_ref`] provided as parameter by [`iox2_unable_to_deliver_handler`]
///
/// # Safety
///
/// * `info_handle` must be a valid handle
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_unable_to_deliver_info_retries(
    info_handle: iox2_unable_to_deliver_info_h_ref,
) -> u64 {
    unable_to_deliver_info_as_type(info_handle).retries
}

/// Obtains the elapsed time since the initial retry
///
/// # Arguments
///
/// * `info_handle` - Must be a valid [`iox2_unable_to_deliver_info_h_ref`] provided as parameter by [`iox2_unable_to_deliver_handler`]
/// * `seconds` - in parameter for the seconds part of the elapsed time
/// * `nanoseconds` - in parameter for the nanoseconds part of the elapsed time
///
/// # Safety
///
/// * `info_handle` must be a valid handle
/// * `seconds` must not be a null pointer
/// * `nanoseconds` must not be a null pointer
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_unable_to_deliver_info_elapsed_time(
    info_handle: iox2_unable_to_deliver_info_h_ref,
    seconds: *mut u64,
    nanoseconds: *mut u32,
) {
    assert!(!seconds.is_null());
    assert!(!nanoseconds.is_null());

    let elapsed = unable_to_deliver_info_as_type(info_handle).elapsed_time;

    unsafe {
        *seconds = elapsed.as_secs();
        *nanoseconds = elapsed.subsec_nanos();
    }
}

/// The unable to deliver handler signature
///
/// # Arguments
///
/// * [`iox2_unable_to_deliver_info_h_ref`] is a handle to obtain some information for the incident;
///   to be used with the `iox2_unable_to_deliver_info_*` functions
/// * [`iox2_callback_context`](crate::iox2_callback_context) is a user defined callback context
///   to provide additional information
///
/// # Returns
///
/// [`iox2_unable_to_deliver_action_e`] the selected action to handle the incident
///
/// # Safety
///
/// * `iox2_callback_context` is stored for later use; if the port, including the send and receive functions,
///   is accessed from multiple threads, the `ctx` must be thread-safe
pub type iox2_unable_to_deliver_handler = extern "C" fn(
    iox2_unable_to_deliver_info_h_ref,
    iox2_callback_context,
) -> iox2_unable_to_deliver_action_e;

// END types definition
