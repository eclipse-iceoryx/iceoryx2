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

use std::ffi::CString;
use std::ffi::c_void;

use r2r_rcl::{
    RCL_RET_OK, RCL_RET_SUBSCRIPTION_TAKE_FAILED, rcl_get_zero_initialized_subscription,
    rcl_serialized_message_t, rcl_subscription_fini, rcl_subscription_get_default_options,
    rcl_subscription_init, rcl_take_serialized_message, rcutils_allocator_t, rmw_message_info_t,
};

use crate::rcl::NodeHandle;
use crate::typesupport::TypeSupportHandle;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    InvalidTopic,
    SubscriptionInit(i32),
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TakeError {
    /// The loan closure declined to provide a destination buffer.
    LoanDeclined,
    Take(i32),
}

impl core::fmt::Display for TakeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TakeError::{self:?}")
    }
}

impl core::error::Error for TakeError {}

/// Per-message metadata accompanying a take.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct MessageInfo {
    /// The originating DDS writer's GUID.
    pub gid: [u8; 16],
    pub source_timestamp_ns: i64,
    pub sequence_number: u64,
}

impl From<&rmw_message_info_t> for MessageInfo {
    fn from(info: &rmw_message_info_t) -> Self {
        let mut gid = [0u8; 16];
        gid.copy_from_slice(&info.publisher_gid.data[..16]);
        Self {
            gid,
            source_timestamp_ns: info.source_timestamp,
            sequence_number: info.publication_sequence_number,
        }
    }
}

/// Receives serialized messages from a ROS 2 topic.
#[derive(Debug)]
pub struct Subscription {
    subscription: *mut r2r_rcl::rcl_subscription_t,
    node: NodeHandle,
    /// Keeps the typesupport library loaded while the endpoint uses it.
    _type_support: TypeSupportHandle,
}

impl Subscription {
    pub fn create(
        node: &NodeHandle,
        topic: &str,
        type_support: TypeSupportHandle,
    ) -> Result<Self, CreationError> {
        let topic = CString::new(topic).map_err(|_| CreationError::InvalidTopic)?;

        unsafe {
            let mut subscription = Box::new(rcl_get_zero_initialized_subscription());
            let mut options = rcl_subscription_get_default_options();
            // Prevent loopback.
            options.rmw_subscription_options.ignore_local_publications = true;
            let ret = rcl_subscription_init(
                subscription.as_mut(),
                node.handle(),
                type_support.handle(),
                topic.as_ptr(),
                &options,
            );
            if ret != RCL_RET_OK as i32 {
                return Err(CreationError::SubscriptionInit(ret));
            }

            Ok(Self {
                subscription: Box::into_raw(subscription),
                node: node.clone(),
                _type_support: type_support,
            })
        }
    }

    /// Takes the next message from the subscription's queue directly into a
    /// caller-provided buffer, i.e. loaned iceoryx2 payload memory.
    ///
    /// The rmw learns the message size mid-take and requests the destination
    /// buffer through the serialized message's allocator; that request is
    /// routed to `loan`, which must return a buffer of exactly the
    /// requested size (or [`None`] to abort the take). The serialized bytes
    /// are written straight into it.
    ///
    /// `loan` must not panic (it is invoked from C). It is called
    /// at most once per take; the caller owns the provided buffer
    /// throughout. Returns the number of bytes written and the message
    /// info, or [`None`] when the queue is empty (callers loop until then).
    pub fn take_into<F>(&self, loan: F) -> Result<Option<(usize, MessageInfo)>, TakeError>
    where
        F: FnOnce(usize) -> Option<*mut u8>,
    {
        let mut loan: Option<F> = Some(loan);
        let mut message = rcl_serialized_message_t {
            buffer: core::ptr::null_mut(),
            buffer_length: 0,
            buffer_capacity: 0,
            allocator: loan_allocator::<F>(&mut loan),
        };
        let mut message_info = rmw_message_info_t::default();

        let ret = unsafe {
            rcl_take_serialized_message(
                self.subscription,
                &mut message,
                &mut message_info,
                core::ptr::null_mut(),
            )
        };
        if ret == RCL_RET_SUBSCRIPTION_TAKE_FAILED as i32 {
            return Ok(None);
        }
        if ret != RCL_RET_OK as i32 {
            // A consumed loan closure means the buffer request was declined,
            // which surfaces as an allocation error.
            if loan.is_none() {
                return Err(TakeError::LoanDeclined);
            }
            return Err(TakeError::Take(ret));
        }

        Ok(Some((
            message.buffer_length,
            MessageInfo::from(&message_info),
        )))
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        unsafe {
            let mut subscription = Box::from_raw(self.subscription);
            let _ = rcl_subscription_fini(subscription.as_mut(), self.node.handle());
        }
    }
}

/// An `rcutils` allocator that satisfies the single grow-from-empty request
/// of a take by handing out the loaned buffer.
fn loan_allocator<F: FnOnce(usize) -> Option<*mut u8>>(
    loan: &mut Option<F>,
) -> rcutils_allocator_t {
    unsafe extern "C" fn allocate<F: FnOnce(usize) -> Option<*mut u8>>(
        size: usize,
        state: *mut c_void,
    ) -> *mut c_void {
        // Recover the provided loan function and call it to get the buffer
        // with the correct size in iceoryx2 shared memory.
        let loan = unsafe { &mut *(state as *mut Option<F>) };
        match loan.take().and_then(|loan| loan(size)) {
            Some(buffer) => buffer as *mut c_void,
            None => core::ptr::null_mut(),
        }
    }

    unsafe extern "C" fn reallocate<F: FnOnce(usize) -> Option<*mut u8>>(
        pointer: *mut c_void,
        size: usize,
        state: *mut c_void,
    ) -> *mut c_void {
        if !pointer.is_null() {
            // Growing an already-loaned buffer is not possible.
            return core::ptr::null_mut();
        }
        unsafe { allocate::<F>(size, state) }
    }

    unsafe extern "C" fn deallocate(_pointer: *mut c_void, _state: *mut c_void) {
        // The loaned buffer stays owned by the lender.
    }

    unsafe extern "C" fn zero_allocate(
        _number_of_elements: usize,
        _size_of_element: usize,
        _state: *mut c_void,
    ) -> *mut c_void {
        core::ptr::null_mut()
    }

    rcutils_allocator_t {
        allocate: Some(allocate::<F>),
        deallocate: Some(deallocate),
        reallocate: Some(reallocate::<F>),
        zero_allocate: Some(zero_allocate),
        state: loan as *mut Option<F> as *mut c_void,
    }
}
