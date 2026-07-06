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

use alloc::string::String;
use core::time::Duration;

use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;

use serde::{Deserialize, Serialize};

/// Description of a service for creation of both local iceoryx endpoints
/// and remote backend endpoints.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ServiceDescription {
    /// The service's identity: keys bridges, discovery state and backend
    /// wire entries. Computed from (name, pattern) via
    /// [`ServiceHash::new()`] with the [`Service`]'s name hasher.
    pub service_hash: ServiceHash,
    pub name: ServiceName,
    pub pattern: PatternDescription,
}

/// Description of a services messaging pattern.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum PatternDescription {
    PublishSubscribe(PublishSubscribeDescription),
    Event(EventDescription),
}

impl core::fmt::Display for PatternDescription {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PatternDescription::PublishSubscribe(_) => write!(f, "PublishSubscribe"),
            PatternDescription::Event(_) => write!(f, "Event"),
        }
    }
}

/// Settings for a messaging pattern, either known or absent.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum PatternSettings<T> {
    Value(T),
    UnknownApplyDefaults,
}

/// Description of a publish-subscribe service.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PublishSubscribeDescription {
    pub user_header: TypeDescription,
    pub payload: TypeDescription,
    pub settings: PatternSettings<PublishSubscribeSettings>,
}

/// Description of an event service.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct EventDescription {
    pub settings: PatternSettings<EventSettings>,
}

/// Description of a services type(s).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TypeDescription {
    pub variant: TypeVariant,
    pub type_name: String,
    pub size: usize,
    pub alignment: usize,
}

/// Settings for publish-subscribe services.
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

/// Settings for event services.
///
/// `None` in the optional fields means the origin service has the feature
/// disabled, not that it is unknown.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct EventSettings {
    pub max_notifiers: usize,
    pub max_listeners: usize,
    pub max_nodes: usize,
    pub event_id_max_value: usize,
    pub deadline: Option<Duration>,
    pub notifier_created_event: Option<usize>,
    pub notifier_dropped_event: Option<usize>,
    pub notifier_dead_event: Option<usize>,
}

/// A [`StaticConfig`] whose messaging pattern the tunnel does not support.
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
        let pattern = match static_config.messaging_pattern() {
            MessagingPattern::PublishSubscribe(config) => {
                let types = config.message_type_details();
                PatternDescription::PublishSubscribe(PublishSubscribeDescription {
                    payload: (&types.payload).into(),
                    user_header: (&types.user_header).into(),
                    settings: PatternSettings::Value(PublishSubscribeSettings {
                        max_subscribers: config.max_subscribers(),
                        max_publishers: config.max_publishers(),
                        max_nodes: config.max_nodes(),
                        history_size: config.history_size(),
                        subscriber_max_buffer_size: config.subscriber_max_buffer_size(),
                        subscriber_max_borrowed_samples: config.subscriber_max_borrowed_samples(),
                        safe_overflow: config.has_safe_overflow(),
                    }),
                })
            }
            MessagingPattern::Event(config) => PatternDescription::Event(EventDescription {
                settings: PatternSettings::Value(EventSettings {
                    max_notifiers: config.max_notifiers(),
                    max_listeners: config.max_listeners(),
                    max_nodes: config.max_nodes(),
                    event_id_max_value: config.event_id_max_value(),
                    deadline: config.deadline(),
                    notifier_created_event: config.notifier_created_event().map(|id| id.as_value()),
                    notifier_dropped_event: config.notifier_dropped_event().map(|id| id.as_value()),
                    notifier_dead_event: config.notifier_dead_event().map(|id| id.as_value()),
                }),
            }),
            _ => return Err(UnsupportedPattern),
        };

        Ok(Self {
            service_hash: *static_config.service_hash(),
            name: *static_config.name(),
            pattern,
        })
    }
}

impl From<&TypeDetail> for TypeDescription {
    fn from(detail: &TypeDetail) -> Self {
        Self {
            variant: detail.variant(),
            type_name: String::from_utf8_lossy(detail.type_name()).into_owned(),
            size: detail.size(),
            alignment: detail.alignment(),
        }
    }
}

/// A [`TypeDescription`] that cannot be represented as a core
/// [`TypeDetail`].
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum InvalidTypeDescription {
    TypeNameTooLong,
}

impl core::fmt::Display for InvalidTypeDescription {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "InvalidTypeDescription::{self:?}")
    }
}

impl core::error::Error for InvalidTypeDescription {}

impl TryFrom<&TypeDescription> for TypeDetail {
    type Error = InvalidTypeDescription;

    fn try_from(description: &TypeDescription) -> Result<Self, Self::Error> {
        TypeDetail::__internal_new_from_parts(
            description.variant,
            &description.type_name,
            description.size,
            description.alignment,
        )
        .map_err(|_| InvalidTypeDescription::TypeNameTooLong)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::constants::MAX_TYPE_NAME_LENGTH;
    use iceoryx2::prelude::*;
    use iceoryx2::service::Service as _;
    use iceoryx2::service::local::Service;
    use iceoryx2::service::messaging_pattern::MessagingPattern;
    use iceoryx2::testing::*;
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn round_trips_type_detail() {
        let detail = TypeDetail::new::<u64>(TypeVariant::FixedSize);

        let description = TypeDescription::from(&detail);
        assert_that!(description.variant, eq TypeVariant::FixedSize);
        assert_that!(description.type_name, eq "u64");
        assert_that!(description.size, eq core::mem::size_of::<u64>());
        assert_that!(description.alignment, eq core::mem::align_of::<u64>());

        let round_tripped = TypeDetail::try_from(&description).unwrap();
        assert_that!(round_tripped, eq detail);
    }

    #[test]
    fn rejects_overlong_type_name() {
        let description = TypeDescription {
            variant: TypeVariant::FixedSize,
            type_name: "x".repeat(MAX_TYPE_NAME_LENGTH + 1),
            size: 1,
            alignment: 1,
        };

        let result = TypeDetail::try_from(&description);
        assert_that!(result, eq Err(InvalidTypeDescription::TypeNameTooLong));
    }

    #[test]
    fn maps_publish_subscribe_static_config() {
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
            .create::<Service>()
            .unwrap();
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
            .unwrap();

        let static_config =
            Service::details(&service_name, &config, MessagingPattern::PublishSubscribe)
                .unwrap()
                .unwrap()
                .static_details;
        let sut = ServiceDescription::try_from(&static_config).unwrap();

        assert_that!(sut.name, eq service_name);
        assert_that!(sut.service_hash, eq * static_config.service_hash());
        assert_that!(
            sut.pattern,
            eq PatternDescription::PublishSubscribe(PublishSubscribeDescription {
                user_header: (&TypeDetail::new::<()>(TypeVariant::FixedSize)).into(),
                payload: (&TypeDetail::new::<u64>(TypeVariant::FixedSize)).into(),
                settings: PatternSettings::Value(PublishSubscribeSettings {
                    max_subscribers: MAX_SUBSCRIBERS,
                    max_publishers: MAX_PUBLISHERS,
                    max_nodes: MAX_NODES,
                    history_size: HISTORY_SIZE,
                    subscriber_max_buffer_size: SUBSCRIBER_MAX_BUFFER_SIZE,
                    subscriber_max_borrowed_samples: SUBSCRIBER_MAX_BORROWED_SAMPLES,
                    safe_overflow: SAFE_OVERFLOW,
                }),
            })
        );
    }

    #[test]
    fn maps_event_static_config() {
        const MAX_NOTIFIERS: usize = 7;
        const MAX_LISTENERS: usize = 11;
        const MAX_NODES: usize = 19;
        const EVENT_ID_MAX_VALUE: usize = 37;
        const DEADLINE: Duration = Duration::from_millis(100);
        const NOTIFIER_CREATED_EVENT: usize = 21;
        const NOTIFIER_DROPPED_EVENT: usize = 22;
        const NOTIFIER_DEAD_EVENT: usize = 23;

        let config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&config)
            .create::<Service>()
            .unwrap();
        let service_name = generate_service_name();

        let _service = node
            .service_builder(&service_name)
            .event()
            .max_notifiers(MAX_NOTIFIERS)
            .max_listeners(MAX_LISTENERS)
            .max_nodes(MAX_NODES)
            .event_id_max_value(EVENT_ID_MAX_VALUE)
            .deadline(DEADLINE)
            .notifier_created_event(EventId::new(NOTIFIER_CREATED_EVENT))
            .notifier_dropped_event(EventId::new(NOTIFIER_DROPPED_EVENT))
            .notifier_dead_event(EventId::new(NOTIFIER_DEAD_EVENT))
            .create()
            .unwrap();

        let static_config = Service::details(&service_name, &config, MessagingPattern::Event)
            .unwrap()
            .unwrap()
            .static_details;
        let sut = ServiceDescription::try_from(&static_config).unwrap();

        assert_that!(sut.name, eq service_name);
        assert_that!(sut.service_hash, eq * static_config.service_hash());
        assert_that!(
            sut.pattern,
            eq PatternDescription::Event(EventDescription {
                settings: PatternSettings::Value(EventSettings {
                    max_notifiers: MAX_NOTIFIERS,
                    max_listeners: MAX_LISTENERS,
                    max_nodes: MAX_NODES,
                    event_id_max_value: EVENT_ID_MAX_VALUE,
                    deadline: Some(DEADLINE),
                    notifier_created_event: Some(NOTIFIER_CREATED_EVENT),
                    notifier_dropped_event: Some(NOTIFIER_DROPPED_EVENT),
                    notifier_dead_event: Some(NOTIFIER_DEAD_EVENT),
                }),
            })
        );
    }

    #[test]
    fn maps_disabled_event_features_to_none() {
        let config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&config)
            .create::<Service>()
            .unwrap();
        let service_name = generate_service_name();

        let _service = node
            .service_builder(&service_name)
            .event()
            .disable_deadline()
            .disable_notifier_created_event()
            .disable_notifier_dropped_event()
            .disable_notifier_dead_event()
            .create()
            .unwrap();

        let static_config = Service::details(&service_name, &config, MessagingPattern::Event)
            .unwrap()
            .unwrap()
            .static_details;
        let sut = ServiceDescription::try_from(&static_config).unwrap();

        let PatternDescription::Event(description) = sut.pattern else {
            panic!("expected an event pattern description");
        };
        let PatternSettings::Value(settings) = description.settings else {
            panic!("expected provided event settings");
        };
        assert_that!(settings.deadline, eq None);
        assert_that!(settings.notifier_created_event, eq None);
        assert_that!(settings.notifier_dropped_event, eq None);
        assert_that!(settings.notifier_dead_event, eq None);
    }

    #[test]
    fn rejects_unsupported_messaging_pattern() {
        let config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&config)
            .create::<Service>()
            .unwrap();
        let service_name = generate_service_name();

        let _service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create()
            .unwrap();

        let static_config =
            Service::details(&service_name, &config, MessagingPattern::RequestResponse)
                .unwrap()
                .unwrap()
                .static_details;
        let result = ServiceDescription::try_from(&static_config);

        assert_that!(result, eq Err(UnsupportedPattern));
    }
}
