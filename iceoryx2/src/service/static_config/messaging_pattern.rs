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
    /// checks whether the 2 MessagingPatterns are the same regardless the values inside them.
    pub(crate) fn is_same_pattern(&self, rhs: &MessagingPattern) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(rhs)
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

#[cfg(test)]
mod tests {
    use iceoryx2_bb_testing::assert_that;

    use super::*;
    use crate::service::config;
    use crate::service::static_config::event;
    use crate::service::static_config::publish_subscribe;

    #[test]
    fn test_is_same_pattern() {
        let cfg = config::Config::default();
        let p1 = MessagingPattern::PublishSubscribe(publish_subscribe::StaticConfig::new(&cfg));
        let p2 = MessagingPattern::PublishSubscribe(publish_subscribe::StaticConfig::new(&cfg));
        assert_that!(p1.is_same_pattern(&p2), eq true);
        assert_that!(p2.is_same_pattern(&p1), eq true);

        let mut new_defaults = config::Defaults {
            publish_subscribe: cfg.defaults.publish_subscribe.clone(),
            event: cfg.defaults.event.clone(),
        };
        new_defaults.event.event_id_max_value -= 1;
        new_defaults.publish_subscribe.max_nodes -= 1;

        let cfg2 = config::Config {
            defaults: new_defaults,
            global: cfg.global.clone(),
        };

        // ensure the cfg and cfg2 are not equal
        assert_that!(cfg, ne cfg2);
        let p3 = MessagingPattern::PublishSubscribe(publish_subscribe::StaticConfig::new(&cfg2));
        assert_that!(p1.is_same_pattern(&p3), eq true);
        assert_that!(p3.is_same_pattern(&p1), eq true);

        let e1 = MessagingPattern::Event(event::StaticConfig::new(&cfg));
        let e2 = MessagingPattern::Event(event::StaticConfig::new(&cfg));
        assert_that!(e1.is_same_pattern(&e2), eq true);
        assert_that!(e2.is_same_pattern(&e1), eq true);

        let e3 = MessagingPattern::Event(event::StaticConfig::new(&cfg2));
        assert_that!(e1.is_same_pattern(&e3), eq true);
        assert_that!(e2.is_same_pattern(&e3), eq true);

        assert_that!(p1.is_same_pattern(&e1), eq false);
        assert_that!(p3.is_same_pattern(&e3), eq false);
    }

    #[test]
    fn test_required_amount_of_samples_per_data_segment() {
        let cfg = config::Config::default();
        let p1 = MessagingPattern::PublishSubscribe(publish_subscribe::StaticConfig::new(&cfg));
        let sut = p1.required_amount_of_samples_per_data_segment(0);
        assert_that!(sut, eq 33);
        let sut = p1.required_amount_of_samples_per_data_segment(1);
        assert_that!(sut, eq 34);

        let e1 = MessagingPattern::Event(event::StaticConfig::new(&cfg));
        let sut = e1.required_amount_of_samples_per_data_segment(1);
        assert_that!(sut, eq 0);
    }
}
