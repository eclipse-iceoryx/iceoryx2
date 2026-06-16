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
use std::ffi::CString;
use std::rc::Rc;

use r2r_rcl::{
    RCL_RET_OK, rcl_get_zero_initialized_publisher, rcl_publish_serialized_message,
    rcl_publisher_fini, rcl_publisher_get_default_options, rcl_publisher_init,
    rcl_serialized_message_t, rcutils_get_default_allocator,
};

use crate::rcl::{Node, RclError};
use crate::typesupport::TypeSupport;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    InvalidTopic,
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

/// Publishes pre-serialized messages on a ROS 2 topic.
pub struct Publisher {
    publisher: Box<UnsafeCell<r2r_rcl::rcl_publisher_t>>,
    node: Rc<Node>,
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
    pub fn create(
        node: &Rc<Node>,
        topic: &str,
        type_support: TypeSupport,
    ) -> Result<Self, CreationError> {
        let topic = CString::new(topic).map_err(|_| CreationError::InvalidTopic)?;

        unsafe {
            let publisher = Box::new(UnsafeCell::new(rcl_get_zero_initialized_publisher()));
            let options = rcl_publisher_get_default_options();
            let ret = rcl_publisher_init(
                publisher.get(),
                node.handle(),
                type_support.handle(),
                topic.as_ptr(),
                &options,
            );
            if ret != RCL_RET_OK as i32 {
                return Err(CreationError::PublisherInit(ret.into()));
            }

            Ok(Self {
                publisher,
                node: node.clone(),
                _type_support: type_support,
            })
        }
    }

    /// Publishes the payload as-is; it must be a serialized message of the
    /// publisher's type.
    pub fn publish(&self, payload: &[u8]) -> Result<(), PublishError> {
        let message = rcl_serialized_message_t {
            buffer: payload.as_ptr() as *mut u8,
            buffer_length: payload.len(),
            buffer_capacity: payload.len(),
            allocator: unsafe { rcutils_get_default_allocator() },
        };

        let ret = unsafe {
            rcl_publish_serialized_message(self.publisher.get(), &message, core::ptr::null_mut())
        };
        if ret != RCL_RET_OK as i32 {
            return Err(PublishError::Publish(ret.into()));
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
