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
use std::rc::Rc;

use r2r_rcl::{
    RCL_RET_OK, RCL_RET_SUBSCRIPTION_TAKE_FAILED, rcl_get_zero_initialized_subscription,
    rcl_serialized_message_t, rcl_subscription_fini, rcl_subscription_get_default_options,
    rcl_subscription_get_publisher_count, rcl_subscription_init,
    rcl_subscription_set_on_new_message_callback, rcl_take_serialized_message, rmw_message_info_t,
};

use iceoryx2_bb_concurrency::cell::UnsafeCell;
use iceoryx2_log::{fail, origin};

use crate::qos::QosProfile;
use crate::rcl::node::RclNode;
use crate::rcl::{Gid, RclError, TopicName, qos};
use crate::rcutils::LoanAllocator;
use crate::typesupport::MessageTypeSupport;

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
pub enum CountError {
    Count,
}

impl core::fmt::Display for CountError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CountError::{self:?}")
    }
}

impl core::error::Error for CountError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TakeError {
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
    /// The originating DDS writer's ID.
    pub gid: Gid,
    pub source_timestamp_ns: i64,
    pub sequence_number: u64,
}

impl From<&rmw_message_info_t> for MessageInfo {
    fn from(info: &rmw_message_info_t) -> Self {
        Self {
            gid: Gid::from(&info.publisher_gid),
            source_timestamp_ns: info.source_timestamp,
            sequence_number: info.publication_sequence_number,
        }
    }
}

/// Builder for [`RclSubscription`].
#[derive(Debug)]
pub struct RclSubscriptionBuilder<'a> {
    node: Rc<RclNode>,
    topic: &'a TopicName,
    type_support: Rc<MessageTypeSupport>,
    qos: QosProfile,
}

impl<'a> RclSubscriptionBuilder<'a> {
    /// Begins building a subscription on `node` for the given topic and
    /// typesupport.
    pub fn new(
        node: Rc<RclNode>,
        topic: &'a TopicName,
        type_support: Rc<MessageTypeSupport>,
    ) -> Self {
        Self {
            node,
            topic,
            type_support,
            qos: QosProfile::default(),
        }
    }

    /// Sets the QoS profile of the subscription.
    pub fn qos(mut self, qos: QosProfile) -> Self {
        self.qos = qos;
        self
    }

    pub fn create(self) -> Result<RclSubscription, CreationError> {
        let origin = origin!("RclSubscriptionBuilder::create");

        unsafe {
            let subscription = Box::new(UnsafeCell::new(rcl_get_zero_initialized_subscription()));
            let mut options = rcl_subscription_get_default_options();
            qos::apply(&self.qos, &mut options.qos);
            let ret = rcl_subscription_init(
                subscription.get(),
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

            Ok(RclSubscription {
                subscription,
                callback: None,
                node: self.node,
                _type_support: self.type_support,
            })
        }
    }
}

/// Receives serialized messages from a ROS 2 topic.
pub struct RclSubscription {
    subscription: Box<UnsafeCell<r2r_rcl::rcl_subscription_t>>,
    /// The registered new-message callback. rcl holds a thin pointer to it as
    /// `user_data`, hence the box around the boxed closure.
    callback: Option<Box<NewMessageCallback>>,
    node: Rc<RclNode>,
    /// Keeps the typesupport library loaded while the endpoint uses it.
    _type_support: Rc<MessageTypeSupport>,
}

impl core::fmt::Debug for RclSubscription {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RclSubscription")
            .field("subscription", &self.subscription.get())
            .field("callback", &self.callback.is_some())
            .field("node", &self.node)
            .finish_non_exhaustive()
    }
}

impl RclSubscription {
    /// Registers `callback` to be invoked whenever new messages arrive on
    /// the subscription, with the number of new messages. It is invoked by
    /// the RMW from a middleware thread and must not panic; a previously
    /// registered callback is replaced.
    pub fn on_new_message(&mut self, callback: NewMessageCallback) -> Result<(), CallbackError> {
        let origin = origin!("RclSubscription::on_new_message");

        let callback = Box::new(callback);
        let user_data: *const NewMessageCallback = &*callback;

        let ret = unsafe {
            rcl_subscription_set_on_new_message_callback(
                self.subscription.get(),
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
        let origin = origin!("RclSubscription::take_into");

        let mut loan = LoanAllocator::new(loan);
        let mut message = rcl_serialized_message_t {
            buffer: core::ptr::null_mut(),
            buffer_length: 0,
            buffer_capacity: 0,
            allocator: loan.allocator(),
        };
        let mut message_info = rmw_message_info_t::default();

        let ret = unsafe {
            rcl_take_serialized_message(
                self.subscription.get(),
                &mut message,
                &mut message_info,
                core::ptr::null_mut(),
            )
        };
        if ret == RCL_RET_SUBSCRIPTION_TAKE_FAILED as i32 {
            return Ok(None);
        }
        if ret != RCL_RET_OK as i32 {
            if loan.declined() {
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

    /// The number of publishers matched with the subscription.
    pub fn publisher_count(&self) -> Result<usize, CountError> {
        let origin = origin!("RclSubscription::publisher_count");

        let mut count = 0;
        let ret =
            unsafe { rcl_subscription_get_publisher_count(self.subscription.get(), &mut count) };
        if ret != RCL_RET_OK as i32 {
            fail!(
                from origin,
                with CountError::Count,
                "Failed to count the matched publishers: {}",
                RclError::from(ret)
            );
        }
        Ok(count)
    }
}

impl Drop for RclSubscription {
    fn drop(&mut self) {
        unsafe {
            // Clear the callback before the closure is dropped: a middleware
            // thread must not invoke a freed closure.
            if self.callback.is_some() {
                let _ = rcl_subscription_set_on_new_message_callback(
                    self.subscription.get(),
                    None,
                    core::ptr::null(),
                );
            }

            let _ = rcl_subscription_fini(self.subscription.get(), self.node.handle());
        }
    }
}

/// The C entry point rcl invokes when messages arrive; recovers the
/// [`NewMessageCallback`] from the `user_data` pointer registered in
/// [`RclSubscription::on_new_message`] and calls it.
unsafe extern "C" fn on_new_message_trampoline(user_data: *const c_void, number_of_events: usize) {
    let callback = unsafe { &*(user_data as *const NewMessageCallback) };
    callback(number_of_events);
}
