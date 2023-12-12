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

//! Defines the messaging pattern used in a [`Service`](crate::service::Service)-based
//! communication.
//!
//! ## Messaging Patterns
//!
//! ### Publish-Subscribe
//!
//! See the
//! [Wikipedia Article: Publish-subscribe pattern](https://en.wikipedia.org/wiki/Publish%E2%80%93subscribe_pattern).
//! It uses uni-directional communication where `n`
//! [`Publisher`](crate::port::publisher::Publisher)s continuously send data to `m`
//! [`Subscriber`](crate::port::subscriber::Subscriber)s.
//!
//! ### Event
//!
//! Enable processes to notify and wakeup other processes by sending events that are uniquely
//! identified by a [`crate::port::event_id::EventId`]. Hereby, `n`
//! [`Notifier`](crate::port::notifier::Notifier)s can notify `m`
//! [`Listener`](crate::port::listener::Listener)s.
//!
//! **Note:** This does **not** send or receive POSIX signals nor is it based on them.
use crate::service::static_config::event;
use crate::service::static_config::publish_subscribe;
use serde::{Deserialize, Serialize};

/// Contains the static config of the corresponding messaging pattern.
#[non_exhaustive]
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(tag = "messaging_pattern")]
pub enum MessagingPattern {
    PublishSubscribe(publish_subscribe::StaticConfig),
    Event(event::StaticConfig),
}

impl MessagingPattern {
    pub(crate) fn is_same_pattern(&self, rhs: &MessagingPattern) -> bool {
        match self {
            MessagingPattern::PublishSubscribe(_) => {
                matches!(rhs, MessagingPattern::PublishSubscribe(_))
            }
            MessagingPattern::Event(_) => {
                matches!(rhs, MessagingPattern::Event(_))
            }
        }
    }

    pub(crate) fn required_amount_of_samples_per_data_segment(
        &self,
        publisher_max_loaned_samples: usize,
    ) -> usize {
        match self {
            MessagingPattern::PublishSubscribe(v) => {
                v.max_subscribers
                    * (v.subscriber_max_buffer_size + v.subscriber_max_borrowed_samples)
                    + v.history_size
                    + publisher_max_loaned_samples
                    + 1
            }
            _ => 0,
        }
    }
}
