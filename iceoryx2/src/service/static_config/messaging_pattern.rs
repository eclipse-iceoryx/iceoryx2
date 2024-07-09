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

//! Stores the [`Service`](crate::service::Service) messaging pattern specific static configuration.
use std::fmt::Display;

use crate::service::static_config::event;
use crate::service::static_config::publish_subscribe;
use serde::{Deserialize, Serialize};

/// Contains the static config of the corresponding
/// [`service::MessagingPattern`](crate::service::messaging_pattern::MessagingPattern).
#[non_exhaustive]
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(tag = "messaging_pattern")]
pub enum MessagingPattern {
    /// Stores the static config of the
    /// [`service::MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe)
    PublishSubscribe(publish_subscribe::StaticConfig),

    /// Stores the static config of the
    /// [`service::MessagingPattern::Event`](crate::service::messaging_pattern::MessagingPattern::Event)
    Event(event::StaticConfig),
}

impl Display for MessagingPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessagingPattern::Event(_) => write!(f, "Event"),
            MessagingPattern::PublishSubscribe(_) => write!(f, "PublishSubscribe"),
        }
    }
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
            }
            _ => 0,
        }
    }
}
