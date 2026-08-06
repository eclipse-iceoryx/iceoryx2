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

use std::rc::Rc;

use r2r_rcl::{
    RCL_RET_OK, rcl_get_zero_initialized_publisher, rcl_publish_serialized_message,
    rcl_publisher_fini, rcl_publisher_get_default_options, rcl_publisher_init, rcl_ret_t,
    rcl_serialized_message_t, rcutils_get_default_allocator,
};

use iceoryx2_bb_concurrency::cell::UnsafeCell;
use iceoryx2_log::fail;

use crate::qos::QosProfile;
use crate::rcl::node::RclNode;
use crate::rcl::{RclError, TopicName, qos};
use crate::typesupport::TypeSupport;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    PublisherInit,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PublishError {
    Publish,
}

impl core::fmt::Display for PublishError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PublishError::{self:?}")
    }
}

impl core::error::Error for PublishError {}

/// Builder for [`RclPublisher`].
#[derive(Debug)]
pub struct RclPublisherBuilder<'a> {
    node: Rc<RclNode>,
    topic: &'a TopicName,
    type_support: Rc<TypeSupport>,
    qos: QosProfile,
}

impl<'a> RclPublisherBuilder<'a> {
    /// Begins building a publisher on `node` for the given topic and
    /// typesupport.
    pub fn new(node: Rc<RclNode>, topic: &'a TopicName, type_support: Rc<TypeSupport>) -> Self {
        Self {
            node,
            topic,
            type_support,
            qos: QosProfile::default(),
        }
    }

    /// Sets the QoS profile of the publisher.
    pub fn qos(mut self, qos: QosProfile) -> Self {
        self.qos = qos;
        self
    }

    pub fn create(self) -> Result<RclPublisher, CreationError> {
        let origin = "RclPublisherBuilder::create";

        unsafe {
            let publisher = Box::new(UnsafeCell::new(rcl_get_zero_initialized_publisher()));
            let mut options = rcl_publisher_get_default_options();
            qos::apply(&self.qos, &mut options.qos);

            let ret = rcl_publisher_init(
                publisher.get(),
                self.node.handle(),
                self.type_support.handle(),
                self.topic.as_c_str().as_ptr(),
                &options,
            );
            if ret != RCL_RET_OK as rcl_ret_t {
                fail!(
                    from origin,
                    with CreationError::PublisherInit,
                    "Failed to initialize publisher: {}",
                    RclError::from(ret)
                );
            }

            Ok(RclPublisher {
                node: self.node,
                publisher,
                _type_support: self.type_support,
            })
        }
    }
}

/// Publishes pre-serialized messages on a ROS 2 topic.
pub struct RclPublisher {
    node: Rc<RclNode>,
    publisher: Box<UnsafeCell<r2r_rcl::rcl_publisher_t>>,
    /// Keeps the typesupport library loaded while the endpoint uses it.
    _type_support: Rc<TypeSupport>,
}

impl core::fmt::Debug for RclPublisher {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RclPublisher")
            .field("publisher", &self.publisher.get())
            .field("node", &self.node)
            .field("_type_support", &self._type_support)
            .finish()
    }
}

impl RclPublisher {
    /// Publishes the payload as-is; it must be a serialized message of the
    /// publisher's type.
    pub fn publish(&self, payload: &[u8]) -> Result<(), PublishError> {
        let origin = "RclPublisher::publish";

        let message = rcl_serialized_message_t {
            buffer: payload.as_ptr() as *mut u8,
            buffer_length: payload.len(),
            buffer_capacity: payload.len(),
            allocator: unsafe { rcutils_get_default_allocator() },
        };

        let ret = unsafe {
            rcl_publish_serialized_message(self.publisher.get(), &message, core::ptr::null_mut())
        };
        if ret != RCL_RET_OK as rcl_ret_t {
            fail!(
                from origin,
                with PublishError::Publish,
                "Failed to publish serialized message: {}",
                RclError::from(ret)
            );
        }

        Ok(())
    }
}

impl Drop for RclPublisher {
    fn drop(&mut self) {
        unsafe {
            let _ = rcl_publisher_fini(self.publisher.get(), self.node.handle());
        }
    }
}
