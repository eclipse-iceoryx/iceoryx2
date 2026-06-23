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

use core::cell::UnsafeCell;
use std::rc::Rc;

use r2r_rcl::{
    RCL_RET_OK, rcl_get_zero_initialized_publisher, rcl_publish_serialized_message,
    rcl_publisher_fini, rcl_publisher_get_default_options, rcl_publisher_init, rcl_ret_t,
    rcl_serialized_message_t, rcutils_get_default_allocator,
};

use iceoryx2_log::fail;

use crate::rcl::node::NodeInner;
use crate::rcl::{RclError, TopicName};
use crate::typesupport::TypeSupport;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    PublisherInit(RclError),
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PublishError {
    Publish(RclError),
}

impl core::fmt::Display for PublishError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PublishError::{self:?}")
    }
}

impl core::error::Error for PublishError {}

/// Builder for [`Publisher`].
#[derive(Debug)]
pub struct Builder {
    node: Rc<NodeInner>,
    topic: TopicName,
    type_support: TypeSupport,
}

impl Builder {
    pub(crate) fn new(node: Rc<NodeInner>, topic: TopicName, type_support: TypeSupport) -> Self {
        Self {
            node,
            topic,
            type_support,
        }
    }

    pub fn create(self) -> Result<Publisher, CreationError> {
        let origin = "Publisher::Builder::create";

        unsafe {
            let publisher = Box::new(UnsafeCell::new(rcl_get_zero_initialized_publisher()));
            let options = rcl_publisher_get_default_options();

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
                    with CreationError::PublisherInit(ret.into()),
                    "Failed to initialize publisher"
                );
            }

            Ok(Publisher {
                node: self.node,
                publisher,
                _type_support: self.type_support,
            })
        }
    }
}

/// Publishes pre-serialized messages on a ROS 2 topic.
pub struct Publisher {
    node: Rc<NodeInner>,
    publisher: Box<UnsafeCell<r2r_rcl::rcl_publisher_t>>,
    /// Keeps the typesupport library loaded while the endpoint uses it.
    _type_support: TypeSupport,
}

impl core::fmt::Debug for Publisher {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Publisher")
            .field("publisher", &self.publisher.get())
            .field("node", &self.node)
            .field("_type_support", &self._type_support)
            .finish()
    }
}

impl Publisher {
    /// Publishes the payload as-is; it must be a serialized message of the
    /// publisher's type.
    pub fn publish(&self, payload: &[u8]) -> Result<(), PublishError> {
        let origin = "Publisher::publish";

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
                with PublishError::Publish(ret.into()),
                "Failed to publish serialized message"
            );
        }

        Ok(())
    }
}

impl Drop for Publisher {
    fn drop(&mut self) {
        unsafe {
            let _ = rcl_publisher_fini(self.publisher.get(), self.node.handle());
        }
    }
}
