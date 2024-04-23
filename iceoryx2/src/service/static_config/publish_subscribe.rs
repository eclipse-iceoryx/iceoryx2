// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

//! # Example
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let service_name = ServiceName::new("My/Funk/ServiceName")?;
//! let pubsub = zero_copy::Service::new(&service_name)
//!     .publish_subscribe()
//!     .typed::<u64>()
//!     .open_or_create()?;
//!
//! println!("type name:                        {:?}", pubsub.static_config().type_name());
//! println!("max publishers:                   {:?}", pubsub.static_config().max_supported_publishers());
//! println!("max subscribers:                  {:?}", pubsub.static_config().max_supported_subscribers());
//! println!("subscriber buffer size:           {:?}", pubsub.static_config().subscriber_max_buffer_size());
//! println!("history size:                     {:?}", pubsub.static_config().history_size());
//! println!("subscriber max borrowed samples:  {:?}", pubsub.static_config().subscriber_max_borrowed_samples());
//! println!("safe overflow:                    {:?}", pubsub.static_config().has_safe_overflow());
//!
//! # Ok(())
//! # }
//! ```

use std::alloc::Layout;

use crate::{config, message::Message};
use serde::{Deserialize, Serialize};

/// The static configuration of an
/// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe)
/// based service. Contains all parameters that do not change during the lifetime of a
/// [`Service`](crate::service::Service).
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct StaticConfig {
    pub(crate) max_subscribers: usize,
    pub(crate) max_publishers: usize,
    pub(crate) history_size: usize,
    pub(crate) subscriber_max_buffer_size: usize,
    pub(crate) subscriber_max_borrowed_samples: usize,
    pub(crate) enable_safe_overflow: bool,
    pub(crate) type_details: TypeDetails,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Typed {
    pub type_name: String,
    pub msg_size: usize,
    pub msg_alignment: usize,
    pub type_size: usize,
    pub type_alignment: usize,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Sliced {
    pub type_name: String,
    pub msg_size: usize,
    pub msg_alignment: usize,
    pub type_size: usize,
    pub type_alignment: usize,
    pub max_elements: usize,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TypeDetails {
    Typed { typed: Typed },
    Sliced { sliced: Sliced },
}

impl TypeDetails {
    pub fn from_type<MessageType, Header>() -> Self {
        Self::Typed {
            typed: Typed {
                type_name: core::any::type_name::<MessageType>().to_string(),
                msg_size: core::mem::size_of::<Message<Header, MessageType>>(),
                msg_alignment: core::mem::align_of::<Message<Header, MessageType>>(),
                type_size: core::mem::size_of::<MessageType>(),
                type_alignment: core::mem::align_of::<MessageType>(),
            },
        }
    }

    pub fn from_slice<MessageType, Header>(max_elements: usize) -> Self {
        Self::Sliced {
            sliced: Sliced {
                type_name: core::any::type_name::<MessageType>().to_string(),
                msg_size: core::mem::size_of::<Message<Header, MessageType>>()
                    + core::mem::size_of::<MessageType>() * max_elements,
                msg_alignment: core::mem::align_of::<Message<Header, MessageType>>(),
                type_size: core::mem::size_of::<MessageType>() * max_elements,
                type_alignment: core::mem::align_of::<MessageType>(),
                max_elements,
            },
        }
    }

    pub fn msg_layout(&self) -> Layout {
        match self {
            Self::Typed { typed: d } => unsafe {
                Layout::from_size_align_unchecked(d.msg_size, d.msg_alignment)
            },
            Self::Sliced { sliced: d } => unsafe {
                Layout::from_size_align_unchecked(d.msg_size, d.msg_alignment)
            },
        }
    }

    pub fn type_layout(&self) -> Layout {
        match self {
            Self::Typed { typed: d } => unsafe {
                Layout::from_size_align_unchecked(d.type_size, d.type_alignment)
            },
            Self::Sliced { sliced: d } => unsafe {
                Layout::from_size_align_unchecked(d.type_size, d.type_alignment)
            },
        }
    }

    pub fn is_compatible(&self, rhs: &Self) -> bool {
        match self {
            TypeDetails::Typed { typed: lhs } => {
                if let TypeDetails::Typed { typed: rhs } = rhs {
                    lhs == rhs
                } else {
                    false
                }
            }
            TypeDetails::Sliced { sliced: lhs } => {
                if let TypeDetails::Sliced { sliced: rhs } = rhs {
                    // everything must be equal except max_elements, this can be detected at
                    // runtime
                    lhs.type_name == rhs.type_name
                        && lhs.type_size == rhs.type_size
                        && lhs.type_alignment == rhs.type_alignment
                        && lhs.msg_size == rhs.msg_size
                        && lhs.msg_alignment == rhs.msg_alignment
                } else {
                    false
                }
            }
        }
    }
}

impl StaticConfig {
    pub(crate) fn new(config: &config::Config) -> Self {
        Self {
            max_subscribers: config.defaults.publish_subscribe.max_subscribers,
            max_publishers: config.defaults.publish_subscribe.max_publishers,
            history_size: config.defaults.publish_subscribe.publisher_history_size,
            subscriber_max_buffer_size: config
                .defaults
                .publish_subscribe
                .subscriber_max_buffer_size,
            subscriber_max_borrowed_samples: config
                .defaults
                .publish_subscribe
                .subscriber_max_borrowed_samples,
            enable_safe_overflow: config.defaults.publish_subscribe.enable_safe_overflow,
            type_details: TypeDetails::Typed {
                typed: Typed {
                    type_name: String::new(),
                    type_size: 0,
                    type_alignment: 0,
                    msg_size: 0,
                    msg_alignment: 0,
                },
            },
        }
    }

    /// Returns the maximum supported amount of [`crate::port::publisher::Publisher`] ports
    pub fn max_supported_publishers(&self) -> usize {
        self.max_publishers
    }

    /// Returns the maximum supported amount of [`crate::port::subscriber::Subscriber`] ports
    pub fn max_supported_subscribers(&self) -> usize {
        self.max_subscribers
    }

    /// Returns the maximum history size that can be requested on connect.
    pub fn history_size(&self) -> usize {
        self.history_size
    }

    /// Returns the maximum supported buffer size for [`crate::port::subscriber::Subscriber`] port
    pub fn subscriber_max_buffer_size(&self) -> usize {
        self.subscriber_max_buffer_size
    }

    /// Returns how many [`crate::sample::Sample`] a [`crate::port::subscriber::Subscriber`] port
    /// can borrow in parallel at most.
    pub fn subscriber_max_borrowed_samples(&self) -> usize {
        self.subscriber_max_borrowed_samples
    }

    /// Returns true if the [`crate::service::Service`] safely overflows, otherwise false. Safe
    /// overflow means that the [`crate::port::publisher::Publisher`] will recycle the oldest
    /// [`crate::sample::Sample`] from the [`crate::port::subscriber::Subscriber`] when its buffer
    /// is full.
    pub fn has_safe_overflow(&self) -> bool {
        self.enable_safe_overflow
    }

    /// Returns the type details of the [`crate::service::Service`].
    pub fn type_details(&self) -> &TypeDetails {
        &self.type_details
    }
}
