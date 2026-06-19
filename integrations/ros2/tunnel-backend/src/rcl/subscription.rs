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
    rcl_subscription_init, rcl_subscription_set_on_new_message_callback,
    rcl_take_serialized_message, rcutils_allocator_t, rmw_message_info_t,
};

use iceoryx2_log::fail;

use crate::rcl::{NodeHandle, RclError};
use crate::typesupport::TypeSupportHandle;

/// Invoked by the RMW (from a middleware thread) when new messages arrive.
pub type NewMessageCallback = Box<dyn Fn(usize) + Send>;
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    InvalidTopic,
    SubscriptionInit,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CallbackError {
    Set,
}

impl core::fmt::Display for CallbackError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CallbackError::{self:?}")
    }
}

impl core::error::Error for CallbackError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TakeError {
    /// The loan closure declined to provide a destination buffer.
    LoanDeclined,
    Take,
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
pub struct Subscription {
    subscription: *mut r2r_rcl::rcl_subscription_t,
    /// TODO: Is there a better way than double Box?
    /// The registered new-message callback. Boxed twice: the inner box is
    /// the object the trampoline's thin `user_data` pointer refers to, and
    /// must stay at a stable heap address while registered.
    callback: Option<Box<NewMessageCallback>>,
    node: NodeHandle,
    /// Keeps the typesupport library loaded while the endpoint uses it.
    _type_support: TypeSupportHandle,
}

impl core::fmt::Debug for Subscription {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Subscription")
            .field("subscription", &self.subscription)
            .field("callback", &self.callback.is_some())
            .field("node", &self.node)
            .finish_non_exhaustive()
    }
}

impl Subscription {
    pub fn create(
        node: &NodeHandle,
        topic: &str,
        type_support: TypeSupportHandle,
    ) -> Result<Self, CreationError> {
        let origin = "Subscription::create";

        let topic = fail!(from origin,
            when CString::new(topic),
            with CreationError::InvalidTopic,
            "Invalid topic name '{}'",
            topic
        );

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
                fail!(from origin,
                    with CreationError::SubscriptionInit,
                    "Failed to initialize subscription: {}",
                    RclError::from(ret)
                );
            }

            Ok(Self {
                subscription: Box::into_raw(subscription),
                callback: None,
                node: node.clone(),
                _type_support: type_support,
            })
        }
    }

    /// Registers `callback` to be invoked whenever new messages arrive on
    /// the subscription, with the number of new messages. It is invoked by
    /// the RMW from a middleware thread and must not panic; a previously
    /// registered callback is replaced.
    pub fn on_new_message(&mut self, callback: NewMessageCallback) -> Result<(), CallbackError> {
        let origin = "Subscription::on_new_message";

        // The trampoline's `user_data` refers to the heap-stable inner box.
        let callback = Box::new(callback);
        let user_data: *const NewMessageCallback = &*callback;

        let ret = unsafe {
            rcl_subscription_set_on_new_message_callback(
                self.subscription,
                Some(new_message_trampoline),
                user_data.cast(),
            )
        };
        if ret != RCL_RET_OK as i32 {
            fail!(from origin,
                with CallbackError::Set,
                "Failed to set on-new-message callback: {}",
                RclError::from(ret)
            );
        }

        self.callback = Some(callback);
        Ok(())
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
    /// info, or [`None`] when the queue is empty.
    pub fn take_into<F>(&self, loan: F) -> Result<Option<(usize, MessageInfo)>, TakeError>
    where
        F: FnOnce(usize) -> Option<*mut u8>,
    {
        let origin = "Subscription::take_into";

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
                fail!(from origin,
                    with TakeError::LoanDeclined,
                    "Failed to take serialized message into loaned buffer"
                );
            }
            fail!(from origin,
                with TakeError::Take,
                "Failed to take serialized message: {}",
                RclError::from(ret)
            );
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
            // Clear the callback before the closure is dropped: a middleware
            // thread must not invoke a freed closure.
            if self.callback.is_some() {
                let _ = rcl_subscription_set_on_new_message_callback(
                    self.subscription,
                    None,
                    core::ptr::null(),
                );
            }

            let mut subscription = Box::from_raw(self.subscription);
            let _ = rcl_subscription_fini(subscription.as_mut(), self.node.handle());
        }
    }
}

/// Bounces the RMW's C callback to the registered [`NewMessageCallback`].
unsafe extern "C" fn new_message_trampoline(user_data: *const c_void, number_of_events: usize) {
    let callback = unsafe { &*(user_data as *const NewMessageCallback) };
    callback(number_of_events);
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
