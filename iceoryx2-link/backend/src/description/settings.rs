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

use core::fmt::Display;
use core::time::Duration;

use iceoryx2::config::Config;
use iceoryx2::port::event_id::EventId;
use iceoryx2::service::messaging_pattern::MessagingPattern;
use iceoryx2::service::service_name::ServiceName;
use serde::{Deserialize, Serialize};

/// Something with an identity.
pub trait Identified {
    type Id: Ord + Clone + Display;

    fn id(&self) -> Self::Id;
}

/// The settings of a service's pattern, identified by the service's name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSettings {
    name: ServiceName,
    pub pattern: PatternSettings,
}

impl ServiceSettings {
    pub fn new(name: ServiceName, pattern: PatternSettings) -> Self {
        Self { name, pattern }
    }
}

impl Identified for ServiceSettings {
    type Id = ServiceName;

    fn id(&self) -> ServiceName {
        self.name
    }
}

/// The settings of a service's pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternSettings {
    PublishSubscribe(PublishSubscribeSettings),
    Event(EventSettings),
}

impl PatternSettings {
    /// The pattern these settings are of.
    pub fn messaging_pattern(&self) -> MessagingPattern {
        match self {
            Self::PublishSubscribe(_) => MessagingPattern::PublishSubscribe,
            Self::Event(_) => MessagingPattern::Event,
        }
    }
}

/// The settings a publish-subscribe service is created with.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PublishSubscribeSettings {
    pub max_subscribers: usize,
    pub max_publishers: usize,
    pub max_nodes: usize,
    pub history_size: usize,
    pub subscriber_max_buffer_size: usize,
    pub subscriber_max_borrowed_samples: usize,
    pub safe_overflow: bool,
}

impl PublishSubscribeSettings {
    /// The settings a publish-subscribe service created with `config`
    /// receives when none are specified.
    pub fn from_config(config: &Config) -> Self {
        let defaults = &config.defaults.publish_subscribe;
        Self {
            max_subscribers: defaults.max_subscribers,
            max_publishers: defaults.max_publishers,
            max_nodes: defaults.max_nodes,
            history_size: defaults.publisher_history_size,
            subscriber_max_buffer_size: defaults.subscriber_max_buffer_size,
            subscriber_max_borrowed_samples: defaults.subscriber_max_borrowed_samples,
            safe_overflow: defaults.enable_safe_overflow,
        }
    }
}

/// The settings an event service is created with.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EventSettings {
    pub max_notifiers: usize,
    pub max_listeners: usize,
    pub max_nodes: usize,
    pub event_id_max_value: usize,
    pub deadline: Option<Duration>,
    pub notifier_created_event: Option<EventId>,
    pub notifier_dropped_event: Option<EventId>,
    pub notifier_dead_event: Option<EventId>,
}

impl EventSettings {
    /// The settings an event service created with `config` receives when
    /// none are specified.
    pub fn from_config(config: &Config) -> Self {
        let defaults = &config.defaults.event;
        Self {
            max_notifiers: defaults.max_notifiers,
            max_listeners: defaults.max_listeners,
            max_nodes: defaults.max_nodes,
            event_id_max_value: defaults.event_id_max_value,
            deadline: defaults.deadline,
            notifier_created_event: defaults.notifier_created_event.map(EventId::new),
            notifier_dropped_event: defaults.notifier_dropped_event.map(EventId::new),
            notifier_dead_event: defaults.notifier_dead_event.map(EventId::new),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::node::NodeBuilder;
    use iceoryx2::service::local;
    use iceoryx2::service::messaging_pattern::MessagingPattern;
    use iceoryx2::testing::{generate_isolated_config, generate_service_name};
    use iceoryx2_bb_testing::assert_that;

    use crate::description::ServiceDescription;
    use crate::description::tests::static_config_of;

    #[test]
    fn settings_from_config_match_a_service_created_without_settings() {
        let config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let service_name = generate_service_name();
        let _service = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .expect("service is created");

        let static_config =
            static_config_of(&service_name, &config, MessagingPattern::PublishSubscribe);
        let described = ServiceDescription::try_from(&static_config).expect("pattern is carried");
        let PatternSettings::PublishSubscribe(settings) = described.settings.pattern else {
            panic!("a publish-subscribe service");
        };

        assert_that!(settings, eq PublishSubscribeSettings::from_config(&config));
    }

    #[test]
    fn event_settings_from_config_match_a_service_created_without_settings() {
        let config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let service_name = generate_service_name();
        let _service = node
            .service_builder(&service_name)
            .event()
            .create()
            .expect("service is created");

        let static_config = static_config_of(&service_name, &config, MessagingPattern::Event);
        let described = ServiceDescription::try_from(&static_config).expect("pattern is carried");
        let PatternSettings::Event(settings) = described.settings.pattern else {
            panic!("an event service");
        };

        assert_that!(settings, eq EventSettings::from_config(&config));
    }
}
