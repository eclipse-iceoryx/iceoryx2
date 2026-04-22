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

use iceoryx2::port::{DegradationAction, DegradationCause, DegradationContext};
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::CStrRepr;

// BEGIN types definition

/// Defines the action that shall be take when an degradation is detected. This can happen when a
/// sample cannot be delivered, or when the system is corrupted and files are modified by
/// non-iceoryx2 instances. Is used as return value of the degradation callback to define a
/// custom behavior.
#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_degradation_action_e {
    /// Perform the default action
    DEFAULT,
    /// Ignore the degradation completely
    IGNORE,
    /// Performs whatever is necessary to discard the degradation
    DISCARD,
    /// Retries the action that caused the degradation
    RETRY,
    /// Blocks until the cause of the degradation disappeared
    BLOCK,
    /// Print out a warning as soon as the degradation is detected
    WARN,
    /// Returns a failure in the function the degradation was detected
    FAIL,
}

impl From<iox2_degradation_action_e> for DegradationAction {
    fn from(value: iox2_degradation_action_e) -> Self {
        match value {
            iox2_degradation_action_e::DEFAULT => DegradationAction::Default,
            iox2_degradation_action_e::IGNORE => DegradationAction::Ignore,
            iox2_degradation_action_e::DISCARD => DegradationAction::Discard,
            iox2_degradation_action_e::RETRY => DegradationAction::Retry,
            iox2_degradation_action_e::BLOCK => DegradationAction::Block,
            iox2_degradation_action_e::WARN => DegradationAction::Warn,
            iox2_degradation_action_e::FAIL => DegradationAction::Fail,
        }
    }
}

/// Defines the cause of a degradation and is a parameter of the degradation callback.
#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_degradation_cause_e {
    /// Connection could not be established
    FAILED_TO_ESTABLISH_CONNECTION,
    /// Connection is corrupted
    CONNECTION_CORRUPTED,
    /// Data could not be delivered
    UNABLE_TO_DELIVER_DATA,
    /// The 'iox2_degradation_action_e' used by the degradation callback was invalid for the given 'iox2_degradation_cause_e'.
    /// The function will return with an error after the invocation of the degradation callback.
    INVALID_DEGRADATION_ACTION,
}

impl From<iox2_degradation_cause_e> for DegradationCause {
    fn from(value: iox2_degradation_cause_e) -> Self {
        match value {
            iox2_degradation_cause_e::FAILED_TO_ESTABLISH_CONNECTION => {
                DegradationCause::FailedToEstablishConnection
            }
            iox2_degradation_cause_e::CONNECTION_CORRUPTED => DegradationCause::ConnectionCorrupted,
            iox2_degradation_cause_e::UNABLE_TO_DELIVER_DATA => {
                DegradationCause::UnableToDeliverData
            }
            iox2_degradation_cause_e::INVALID_DEGRADATION_ACTION => {
                DegradationCause::InvalidDegradationAction
            }
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
            DegradationCause::UnableToDeliverData => {
                iox2_degradation_cause_e::UNABLE_TO_DELIVER_DATA
            }
            DegradationCause::InvalidDegradationAction => {
                iox2_degradation_cause_e::INVALID_DEGRADATION_ACTION
            }
        }
    }
}

pub struct iox2_degradation_context_h;
pub type iox2_degradation_context_h_ref = *const iox2_degradation_context_h;

pub(crate) fn degradation_context_cast(
    context: &DegradationContext,
) -> iox2_degradation_context_h_ref {
    context as *const _ as iox2_degradation_context_h_ref
}

fn degradation_context_as_type<'a>(
    context: iox2_degradation_context_h_ref,
) -> &'a DegradationContext {
    debug_assert!(!context.is_null());
    unsafe { &*(context as *const _ as *const DegradationContext) }
}

/// Obtains the service id, which is involved in the degradation
///
/// # Arguments
///
/// * `context_handle` - Must be a valid [`iox2_degradation_context_h_ref`] provided as parameter by [`iox2_degradation_callback`]
///
/// # Safety
///
/// * `context_handle` must be a valid handle
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_degradation_context_service_id(
    context_handle: iox2_degradation_context_h_ref,
) -> u64 {
    // TODO: How to pass a u128 to the caller? Via in-parameter?
    degradation_context_as_type(context_handle).service_id as u64
}

/// Obtains the receiver port id, which is involved in the degradation
///
/// # Arguments
///
/// * `context_handle` - Must be a valid [`iox2_degradation_context_h_ref`] provided as parameter by [`iox2_degradation_callback`]
///
/// # Safety
///
/// * `context_handle` must be a valid handle
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_degradation_context_receiver_port_id(
    context_handle: iox2_degradation_context_h_ref,
) -> u64 {
    // TODO: How to pass a u128 to the caller? Via in-parameter?
    degradation_context_as_type(context_handle).receiver_port_id as u64
}

/// Obtains the sender port id, which is involved in the degradation
///
/// # Arguments
///
/// * `context_handle` - Must be a valid [`iox2_degradation_context_h_ref`] provided as parameter by [`iox2_degradation_callback`]
///
/// # Safety
///
/// * `context_handle` must be a valid handle
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_degradation_context_sender_port_id(
    context_handle: iox2_degradation_context_h_ref,
) -> u64 {
    // TODO: How to pass a u128 to the caller? Via in-parameter?
    degradation_context_as_type(context_handle).sender_port_id as u64
}

/// The degradation callback signature
///
/// # Arguments
///
/// * [`iox2_degradation_cause_e`] is the cause for the degradation
/// * [`iox2_degradation_context_h_ref`] is a handle to obtain some context information for the degradation;
///   to be used with the `iox2_degradation_context_*` functions
/// * [`iox2_callback_context`](crate::iox2_callback_context) is a user defined callback context
///   to provide additional information
///
/// # Returns
///
/// [`iox2_degradation_action_e`] the selected action to handle the degradation
///
/// Attention
///
/// It is recommended to use a switch statement for the [`iox2_degradation_cause_e`] with a `default` case
/// which returns `iox2_degradation_action_e_DEFAULT`
pub type iox2_degradation_callback = extern "C" fn(
    iox2_degradation_cause_e,
    iox2_degradation_context_h_ref,
    iox2_callback_context,
) -> iox2_degradation_action_e;

// END types definition
