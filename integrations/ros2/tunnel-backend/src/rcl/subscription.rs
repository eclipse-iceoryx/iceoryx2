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

use std::ffi::c_void;
use std::pin::Pin;
use std::rc::Rc;

use r2r_rcl::{
    RCL_RET_OK, RCL_RET_SUBSCRIPTION_TAKE_FAILED, RMW_GID_STORAGE_SIZE,
    rcl_get_zero_initialized_subscription, rcl_serialized_message_t, rcl_subscription_fini,
    rcl_subscription_get_default_options, rcl_subscription_init,
    rcl_subscription_set_on_new_message_callback, rcl_take_serialized_message, rcutils_allocator_t,
    rmw_message_info_t,
};

use iceoryx2_log::fail;

use crate::rcl::node::Node;
use crate::rcl::{RclError, TopicName};
use crate::typesupport::TypeSupport;

/// A callback invoked with the number of newly-arrived messages. The RMW
/// calls it from a middleware thread.
pub type NewMessageCallback = Box<dyn Fn(usize) + Send>;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
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
    pub gid: [u8; RMW_GID_STORAGE_SIZE as usize],
    pub source_timestamp_ns: i64,
    pub sequence_number: u64,
}

impl From<&rmw_message_info_t> for MessageInfo {
    fn from(info: &rmw_message_info_t) -> Self {
        let mut gid = [0u8; RMW_GID_STORAGE_SIZE as usize];
        gid.copy_from_slice(&info.publisher_gid.data);
        Self {
            gid,
            source_timestamp_ns: info.source_timestamp,
            sequence_number: info.publication_sequence_number,
        }
    }
}

/// Builder for [`Subscription`].
#[derive(Debug)]
pub struct Builder<'a> {
    node: Rc<Node>,
    topic: &'a TopicName,
    type_support: Rc<TypeSupport>,
}

impl<'a> Builder<'a> {
    fn new(node: Rc<Node>, topic: &'a TopicName, type_support: Rc<TypeSupport>) -> Self {
        Self {
            node,
            topic,
            type_support,
        }
    }

    pub fn create(self) -> Result<Subscription, CreationError> {
        let origin = "Subscription::Builder::create";

        unsafe {
            let mut subscription = Box::new(rcl_get_zero_initialized_subscription());
            let mut options = rcl_subscription_get_default_options();
            // Prevent loopback.
            options.rmw_subscription_options.ignore_local_publications = true;
            let ret = rcl_subscription_init(
                subscription.as_mut(),
                self.node.handle(),
                self.type_support.handle(),
                self.topic.as_c_str().as_ptr(),
                &options,
            );
            if ret != RCL_RET_OK as i32 {
                fail!(from origin,
                    with CreationError::SubscriptionInit,
                    "Failed to initialize subscription: {}",
                    RclError::from(ret)
                );
            }

            Ok(Subscription {
                subscription: Box::into_raw(subscription),
                callback: None,
                node: self.node,
                _type_support: self.type_support,
            })
        }
    }
}

/// Receives serialized messages from a ROS 2 topic.
pub struct Subscription {
    subscription: *mut r2r_rcl::rcl_subscription_t,
    /// The new-message callback while one is registered, kept alive and pinned
    /// so the `user_data` pointer rcl holds stays valid until it is cleared.
    callback: Option<Pin<Box<NewMessageCallback>>>,
    node: Rc<Node>,
    /// Keeps the typesupport library loaded while the endpoint uses it.
    _type_support: Rc<TypeSupport>,
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
    /// Begins building a subscription on `node` for the given topic and
    /// typesupport.
    #[allow(clippy::new_ret_no_self)]
    pub fn new<'a>(
        node: Rc<Node>,
        topic: &'a TopicName,
        type_support: Rc<TypeSupport>,
    ) -> Builder<'a> {
        Builder::new(node, topic, type_support)
    }

    /// Registers `callback` to be invoked whenever new messages arrive on
    /// the subscription, with the number of new messages. It is invoked by
    /// the RMW from a middleware thread and must not panic; a previously
    /// registered callback is replaced.
    pub fn on_new_message(&mut self, callback: NewMessageCallback) -> Result<(), CallbackError> {
        let origin = "Subscription::on_new_message";

        // Pin to a stable heap address, then pass rcl a thin pointer to the
        // boxed closure as `user_data`.
        let callback = Box::pin(callback);
        let user_data: *const NewMessageCallback = &*callback;

        let ret = unsafe {
            rcl_subscription_set_on_new_message_callback(
                self.subscription,
                Some(on_new_message_trampoline),
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

/// The C entry point rcl invokes when messages arrive; recovers the
/// [`NewMessageCallback`] from the `user_data` pointer registered in
/// [`Subscription::on_new_message`] and calls it.
unsafe extern "C" fn on_new_message_trampoline(user_data: *const c_void, number_of_events: usize) {
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
