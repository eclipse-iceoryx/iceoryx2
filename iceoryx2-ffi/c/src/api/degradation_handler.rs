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

use crate::api::iox2_callback_context;

use crate::api::iox2_buffer_16_align_4_t;
use iceoryx2::port::{DegradationAction, DegradationCause, DegradationInfo};
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::CStrRepr;

// BEGIN types definition

/// Defines the action that shall be take when an degradation is detected. This can happen when a
/// sample cannot be delivered, or when the system is corrupted and files are modified by
/// non-iceoryx2 instances. Is used as return value of the degradation handler to define a
/// custom behavior.
#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_degradation_action_e {
    /// Ignore the degradation completely
    IGNORE,
    /// Print out a warning as soon as the degradation is detected
    WARN,
    /// Returns a failure in the function the degradation was detected
    DEGRADE_AND_FAIL,
}

impl From<iox2_degradation_action_e> for DegradationAction {
    fn from(value: iox2_degradation_action_e) -> Self {
        match value {
            iox2_degradation_action_e::IGNORE => DegradationAction::Ignore,
            iox2_degradation_action_e::WARN => DegradationAction::Warn,
            iox2_degradation_action_e::DEGRADE_AND_FAIL => DegradationAction::DegradeAndFail,
        }
    }
}

/// Defines the cause of a degradation and is a parameter of the degradation handler.
#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_degradation_cause_e {
    /// Connection could not be established
    FAILED_TO_ESTABLISH_CONNECTION,
    /// Connection is corrupted
    CONNECTION_CORRUPTED,
}

impl From<iox2_degradation_cause_e> for DegradationCause {
    fn from(value: iox2_degradation_cause_e) -> Self {
        match value {
            iox2_degradation_cause_e::FAILED_TO_ESTABLISH_CONNECTION => {
                DegradationCause::FailedToEstablishConnection
            }
            iox2_degradation_cause_e::CONNECTION_CORRUPTED => DegradationCause::ConnectionCorrupted,
        }
    }
}

impl From<DegradationCause> for iox2_degradation_cause_e {
    fn from(value: DegradationCause) -> Self {
        match value {
            DegradationCause::FailedToEstablishConnection => {
                iox2_degradation_cause_e::FAILED_TO_ESTABLISH_CONNECTION
            }
            DegradationCause::ConnectionCorrupted => iox2_degradation_cause_e::CONNECTION_CORRUPTED,
        }
    }
}

pub struct iox2_degradation_info_h;
pub type iox2_degradation_info_h_ref = *const iox2_degradation_info_h;

pub(crate) fn degradation_info_cast(info: &DegradationInfo) -> iox2_degradation_info_h_ref {
    info as *const _ as iox2_degradation_info_h_ref
}

fn degradation_info_as_type<'a>(info: iox2_degradation_info_h_ref) -> &'a DegradationInfo {
    debug_assert!(!info.is_null());
    unsafe { &*(info as *const _ as *const DegradationInfo) }
}

/// Obtains the service id, which is involved in the degradation
///
/// # Arguments
///
/// * `info_handle` - Must be a valid [`iox2_degradation_info_h_ref`] provided as parameter by [`iox2_degradation_handler`]
/// * `service_id` - Must be a pointer to a [`iox2_buffer_16_align_4_t`](crate::api::iox2_buffer_16_align_4_t)
///
/// # Safety
///
/// * `info_handle` must be a valid handle
/// * `service_id` must not be a null pointer
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_degradation_info_service_id(
    info_handle: iox2_degradation_info_h_ref,
    service_id: *mut iox2_buffer_16_align_4_t,
) {
    unsafe {
        iox2_buffer_16_align_4_t::write(
            service_id,
            degradation_info_as_type(info_handle)
                .service_id
                .to_be_bytes(),
        );
    }
}

/// Obtains the receiver port id, which is involved in the degradation
///
/// # Arguments
///
/// * `info_handle` - Must be a valid [`iox2_degradation_info_h_ref`] provided as parameter by [`iox2_degradation_handler`]
/// * `receiver_port_id` - Must be a pointer to a [`iox2_buffer_16_align_4_t`](crate::api::iox2_buffer_16_align_4_t)
///
/// # Safety
///
/// * `info_handle` must be a valid handle
/// * `receiver_port_id` must not be a null pointer
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_degradation_info_receiver_port_id(
    info_handle: iox2_degradation_info_h_ref,
    receiver_port_id: *mut iox2_buffer_16_align_4_t,
) {
    unsafe {
        iox2_buffer_16_align_4_t::write(
            receiver_port_id,
            degradation_info_as_type(info_handle)
                .receiver_port_id
                .to_be_bytes(),
        );
    }
}

/// Obtains the sender port id, which is involved in the degradation
///
/// # Arguments
///
/// * `info_handle` - Must be a valid [`iox2_degradation_info_h_ref`] provided as parameter by [`iox2_degradation_handler`]
/// * `sender_port_id` - Must be a pointer to a [`iox2_buffer_16_align_4_t`](crate::api::iox2_buffer_16_align_4_t)
///
/// # Safety
///
/// * `info_handle` must be a valid handle
/// * `sender_port_id` must not be a null pointer
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_degradation_info_sender_port_id(
    info_handle: iox2_degradation_info_h_ref,
    sender_port_id: *mut iox2_buffer_16_align_4_t,
) {
    unsafe {
        iox2_buffer_16_align_4_t::write(
            sender_port_id,
            degradation_info_as_type(info_handle)
                .sender_port_id
                .to_be_bytes(),
        );
    }
}

/// The degradation handler signature
///
/// # Arguments
///
/// * [`iox2_degradation_cause_e`] is the cause for the degradation
/// * [`iox2_degradation_info_h_ref`] is a handle to obtain some information for the degradation;
///   to be used with the `iox2_degradation_info_*` functions
/// * [`iox2_callback_context`](crate::iox2_callback_context) is a user defined handler context
///   to provide additional information
///
/// # Returns
///
/// [`iox2_degradation_action_e`] the selected action to handle the degradation
///
/// # Safety
///
/// * `iox2_callback_context` is stored for later use; if the port, including the send and receive functions,
///   is accessed from multiple threads, the `ctx` must be thread-safe
pub type iox2_degradation_handler = extern "C" fn(
    iox2_degradation_cause_e,
    iox2_degradation_info_h_ref,
    iox2_callback_context,
) -> iox2_degradation_action_e;

// END types definition
