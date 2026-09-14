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

mod descriptor;
mod messaging_pattern;
mod settings;
mod types;

pub use descriptor::ServiceDescriptor;
pub use messaging_pattern::{EventDescription, MessagingPattern, PublishSubscribeDescription};
pub use settings::{
    EventSettings, Identified, PatternSettings, PublishSubscribeSettings, ServiceSettings,
};
pub use types::{
    InvalidSampleLayout, InvalidTypeDescription, PublishSubscribeTypes, ServiceTypes,
    TypeDescription,
};

use iceoryx2::config::Config;
use iceoryx2::service::Service;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern as StaticPattern;

/// A service's [`StaticConfig`] as hash, settings and types.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ServiceDescription {
    hash: ServiceHash,
    settings: ServiceSettings,
    types: ServiceTypes,
}

impl ServiceDescription {
    /// The description of the publish-subscribe service `name` with these
    /// settings and types, hashed with the name hasher of `S`.
    pub fn compose_publish_subscribe<S: Service>(
        name: ServiceName,
        settings: PublishSubscribeSettings,
        types: PublishSubscribeTypes,
    ) -> Self {
        Self::hashed::<S>(
            ServiceSettings::new(name, PatternSettings::PublishSubscribe(settings)),
            ServiceTypes::PublishSubscribe(types),
        )
    }

    /// The description of the event service `name` with these settings,
    /// hashed with the name hasher of `S`.
    pub fn compose_event<S: Service>(name: ServiceName, settings: EventSettings) -> Self {
        Self::hashed::<S>(
            ServiceSettings::new(name, PatternSettings::Event(settings)),
            ServiceTypes::Event,
        )
    }

    /// The description of the service `name` with these types and the
    /// settings a service of their pattern receives from `config` when
    /// none are specified.
    pub fn with_default_settings<S: Service>(
        name: ServiceName,
        types: ServiceTypes,
        config: &Config,
    ) -> Self {
        let settings = match &types {
            ServiceTypes::PublishSubscribe(_) => {
                PatternSettings::PublishSubscribe(PublishSubscribeSettings::from_config(config))
            }
            ServiceTypes::Event => PatternSettings::Event(EventSettings::from_config(config)),
        };
        Self::hashed::<S>(ServiceSettings::new(name, settings), types)
    }

    fn hashed<S: Service>(settings: ServiceSettings, types: ServiceTypes) -> Self {
        Self {
            hash: ServiceHash::new::<S::ServiceNameHasher>(
                &settings.id(),
                settings.pattern.messaging_pattern(),
            ),
            settings,
            types,
        }
    }

    pub fn hash(&self) -> ServiceHash {
        self.hash
    }

    pub fn name(&self) -> ServiceName {
        self.settings.id()
    }

    /// The settings half.
    pub fn settings(&self) -> &ServiceSettings {
        &self.settings
    }

    /// The types half, what crosses.
    pub fn types(&self) -> &ServiceTypes {
        &self.types
    }

    /// The messaging pattern the description represents.
    pub fn messaging_pattern(&self) -> MessagingPattern<'_> {
        MessagingPattern::of(self)
    }
}

/// The [`StaticConfig`] has a messaging pattern the link does not carry.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct UnsupportedPattern;

impl core::fmt::Display for UnsupportedPattern {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UnsupportedPattern")
    }
}

impl core::error::Error for UnsupportedPattern {}

impl TryFrom<&StaticConfig> for ServiceDescription {
    type Error = UnsupportedPattern;

    fn try_from(static_config: &StaticConfig) -> Result<Self, Self::Error> {
        let (pattern, types) = match static_config.messaging_pattern() {
            StaticPattern::PublishSubscribe(config) => {
                let types = config.message_type_details();
                (
                    PatternSettings::PublishSubscribe(PublishSubscribeSettings {
                        max_subscribers: config.max_subscribers(),
                        max_publishers: config.max_publishers(),
                        max_nodes: config.max_nodes(),
                        history_size: config.history_size(),
                        subscriber_max_buffer_size: config.subscriber_max_buffer_size(),
                        subscriber_max_borrowed_samples: config.subscriber_max_borrowed_samples(),
                        safe_overflow: config.has_safe_overflow(),
                    }),
                    ServiceTypes::PublishSubscribe(PublishSubscribeTypes {
                        payload: (&types.payload).into(),
                        user_header: (&types.user_header).into(),
                    }),
                )
            }
            StaticPattern::Event(config) => (
                PatternSettings::Event(EventSettings {
                    max_notifiers: config.max_notifiers(),
                    max_listeners: config.max_listeners(),
                    max_nodes: config.max_nodes(),
                    event_id_max_value: config.event_id_max_value(),
                    deadline: config.deadline(),
                    notifier_created_event: config.notifier_created_event(),
                    notifier_dropped_event: config.notifier_dropped_event(),
                    notifier_dead_event: config.notifier_dead_event(),
                }),
                ServiceTypes::Event,
            ),
            _ => return Err(UnsupportedPattern),
        };

        Ok(Self {
            hash: *static_config.service_hash(),
            settings: ServiceSettings::new(*static_config.name(), pattern),
            types,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::node::NodeBuilder;
    use iceoryx2::port::event_id::EventId;
    use iceoryx2::service::local;
    use iceoryx2::service::messaging_pattern::MessagingPattern as Pattern;
    use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
    use iceoryx2::testing::{generate_isolated_config, generate_service_name};
    use iceoryx2_bb_testing::assert_that;

    /// The static config of the service `name` of `pattern` in the
    /// system of `config`.
    pub(super) fn static_config_of(
        name: &ServiceName,
        config: &Config,
        pattern: Pattern,
    ) -> StaticConfig {
        local::Service::details(name, config, pattern)
            .expect("details are readable")
            .expect("service exists")
            .static_details
    }

    #[test]
    fn publish_subscribe_static_config_is_described() {
        const MAX_SUBSCRIBERS: usize = 11;
        const MAX_PUBLISHERS: usize = 7;
        const MAX_NODES: usize = 19;
        const HISTORY_SIZE: usize = 9;
        const SUBSCRIBER_MAX_BUFFER_SIZE: usize = 13;
        const SUBSCRIBER_MAX_BORROWED_SAMPLES: usize = 3;
        const SAFE_OVERFLOW: bool = true;

        let config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let service_name = generate_service_name();

        let _service = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_subscribers(MAX_SUBSCRIBERS)
            .max_publishers(MAX_PUBLISHERS)
            .max_nodes(MAX_NODES)
            .history_size(HISTORY_SIZE)
            .subscriber_max_buffer_size(SUBSCRIBER_MAX_BUFFER_SIZE)
            .subscriber_max_borrowed_samples(SUBSCRIBER_MAX_BORROWED_SAMPLES)
            .enable_safe_overflow(SAFE_OVERFLOW)
            .create()
            .expect("service is created");

        let static_config = static_config_of(&service_name, &config, Pattern::PublishSubscribe);
        let sut = ServiceDescription::try_from(&static_config).expect("pattern is carried");

        assert_that!(sut.name(), eq service_name);
        assert_that!(sut.hash, eq * static_config.service_hash());
        assert_that!(
            sut.settings.pattern,
            eq PatternSettings::PublishSubscribe(PublishSubscribeSettings {
                max_subscribers: MAX_SUBSCRIBERS,
                max_publishers: MAX_PUBLISHERS,
                max_nodes: MAX_NODES,
                history_size: HISTORY_SIZE,
                subscriber_max_buffer_size: SUBSCRIBER_MAX_BUFFER_SIZE,
                subscriber_max_borrowed_samples: SUBSCRIBER_MAX_BORROWED_SAMPLES,
                safe_overflow: SAFE_OVERFLOW,
            })
        );
        assert_that!(
            sut.types,
            eq ServiceTypes::PublishSubscribe(PublishSubscribeTypes {
                payload: (&TypeDetail::new::<u64>(TypeVariant::FixedSize)).into(),
                user_header: (&TypeDetail::new::<()>(TypeVariant::FixedSize)).into(),
            })
        );
    }

    #[test]
    fn event_static_config_is_described() {
        const MAX_NOTIFIERS: usize = 5;
        const MAX_LISTENERS: usize = 6;
        const MAX_NODES: usize = 17;
        const EVENT_ID_MAX_VALUE: usize = 42;
        const NOTIFIER_CREATED_EVENT: EventId = EventId::new(40);

        let config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let service_name = generate_service_name();

        let _service = node
            .service_builder(&service_name)
            .event()
            .max_notifiers(MAX_NOTIFIERS)
            .max_listeners(MAX_LISTENERS)
            .max_nodes(MAX_NODES)
            .event_id_max_value(EVENT_ID_MAX_VALUE)
            .disable_deadline()
            .notifier_created_event(NOTIFIER_CREATED_EVENT)
            .disable_notifier_dropped_event()
            .disable_notifier_dead_event()
            .create()
            .expect("service is created");

        let static_config = static_config_of(&service_name, &config, Pattern::Event);
        let sut = ServiceDescription::try_from(&static_config).expect("pattern is carried");

        assert_that!(sut.name(), eq service_name);
        assert_that!(sut.hash, eq * static_config.service_hash());
        assert_that!(
            sut.settings.pattern,
            eq PatternSettings::Event(EventSettings {
                max_notifiers: MAX_NOTIFIERS,
                max_listeners: MAX_LISTENERS,
                max_nodes: MAX_NODES,
                event_id_max_value: EVENT_ID_MAX_VALUE,
                deadline: None,
                notifier_created_event: Some(NOTIFIER_CREATED_EVENT),
                notifier_dropped_event: None,
                notifier_dead_event: None,
            })
        );
        assert_that!(sut.types, eq ServiceTypes::Event);
    }

    #[test]
    fn compose_publish_subscribe_computes_the_same_hash_as_the_service() {
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

        let static_config = static_config_of(&service_name, &config, Pattern::PublishSubscribe);
        let described = ServiceDescription::try_from(&static_config).expect("pattern is carried");
        let PatternSettings::PublishSubscribe(settings) = described.settings.pattern else {
            panic!("a publish-subscribe service");
        };
        let ServiceTypes::PublishSubscribe(types) = described.types else {
            panic!("a publish-subscribe service");
        };
        let sut = ServiceDescription::compose_publish_subscribe::<local::Service>(
            service_name,
            settings,
            types,
        );

        assert_that!(sut.hash, eq * static_config.service_hash());
    }

    #[test]
    fn compose_event_hashes_by_its_pattern() {
        let config = Config::default();
        let name = generate_service_name();

        let sut = ServiceDescription::compose_event::<local::Service>(
            name,
            EventSettings::from_config(&config),
        );

        assert_that!(
            sut.hash,
            eq ServiceHash::new::<<local::Service as Service>::ServiceNameHasher>(&name, Pattern::Event)
        );
    }

    #[test]
    fn default_settings_follow_the_types_pattern() {
        let config = Config::default();
        let name = generate_service_name();

        let sut = ServiceDescription::with_default_settings::<local::Service>(
            name,
            ServiceTypes::Event,
            &config,
        );

        assert_that!(sut.settings.pattern, eq PatternSettings::Event(EventSettings::from_config(&config)));
        assert_that!(sut.types, eq ServiceTypes::Event);
    }
}
