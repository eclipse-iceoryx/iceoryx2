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
pub use messaging_pattern::{
    EventDescription, MessagingPattern, PublishSubscribeDescription, RequestResponseDescription,
};
pub use settings::{
    EventSettings, Identified, PatternSettings, PublishSubscribeSettings, RequestResponseSettings,
    ServiceSettings,
};
pub use types::{
    InvalidSampleLayout, InvalidTypeDescription, RequestResponseTypes, SampleTypes, Schema,
    ServiceTypes, TypeDescription, TypeIdentifier,
};

use alloc::vec::Vec;

use iceoryx2::config::Config;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2::service::static_config::message_type_details::MessageTypeDetails;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern as StaticPattern;
use iceoryx2::service::{
    __internal_payload_type_definition, __internal_request_type_definition,
    __internal_response_type_definition, Service, ServiceDetailsError,
};
use iceoryx2_log::{fail, origin};

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
        types: SampleTypes,
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

    /// The description of the request-response service `name` with these
    /// settings and types, hashed with the name hasher of `S`.
    pub fn compose_request_response<S: Service>(
        name: ServiceName,
        settings: RequestResponseSettings,
        types: RequestResponseTypes,
    ) -> Self {
        Self::hashed::<S>(
            ServiceSettings::new(name, PatternSettings::RequestResponse(settings)),
            ServiceTypes::RequestResponse(types),
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
            ServiceTypes::RequestResponse(_) => {
                PatternSettings::RequestResponse(RequestResponseSettings::from_config(config))
            }
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

    /// The service settings.
    pub fn settings(&self) -> &ServiceSettings {
        &self.settings
    }

    /// The service types.
    pub fn types(&self) -> &ServiceTypes {
        &self.types
    }

    /// The messaging pattern the description represents.
    pub fn messaging_pattern(&self) -> MessagingPattern<'_> {
        MessagingPattern::of(self)
    }
}

/// The [`StaticConfig`] has a messaging pattern the link does not carry.
/// Why a static config cannot be described.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DescriptionError {
    /// The service has a messaging pattern the link does not carry.
    UnsupportedPattern,
    /// The service's details could not be read from its static resources.
    ServiceDetails(ServiceDetailsError),
}

impl core::fmt::Display for DescriptionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DescriptionError::{self:?}")
    }
}

impl core::error::Error for DescriptionError {}

impl From<ServiceDetailsError> for DescriptionError {
    fn from(error: ServiceDetailsError) -> Self {
        Self::ServiceDetails(error)
    }
}

impl ServiceDescription {
    /// The description of the service of `static_config` in the system
    /// `node_config` configures.
    pub fn load<S: Service>(
        node_config: &Config,
        static_config: &StaticConfig,
    ) -> Result<Self, DescriptionError> {
        let origin = origin!("ServiceDescription::load");

        let (pattern, types) = match static_config.messaging_pattern() {
            StaticPattern::PublishSubscribe(config) => {
                let schema = fail!(
                    from origin,
                    when __internal_payload_type_definition::<S>(node_config, static_config),
                    to DescriptionError,
                    "Failed to read the type definition of {}", static_config.name()
                );
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
                    ServiceTypes::PublishSubscribe(sample_types(
                        config.message_type_details(),
                        schema,
                    )),
                )
            }
            StaticPattern::RequestResponse(config) => {
                let request_schema = fail!(
                    from origin,
                    when __internal_request_type_definition::<S>(node_config, static_config),
                    to DescriptionError,
                    "Failed to read the request type definition of {}", static_config.name()
                );
                let response_schema = fail!(
                    from origin,
                    when __internal_response_type_definition::<S>(node_config, static_config),
                    to DescriptionError,
                    "Failed to read the response type definition of {}", static_config.name()
                );
                (
                    PatternSettings::RequestResponse(RequestResponseSettings {
                        max_servers: config.max_servers(),
                        max_clients: config.max_clients(),
                        max_nodes: config.max_nodes(),
                        max_active_requests_per_client: config.max_active_requests_per_client(),
                        max_response_buffer_size: config.max_response_buffer_size(),
                        max_borrowed_responses_per_pending_response: config
                            .max_borrowed_responses_per_pending_response(),
                        max_loaned_requests: config.max_loaned_requests(),
                        safe_overflow_for_requests: config.has_safe_overflow_for_requests(),
                        safe_overflow_for_responses: config.has_safe_overflow_for_responses(),
                        fire_and_forget_requests: config.does_support_fire_and_forget_requests(),
                    }),
                    ServiceTypes::RequestResponse(RequestResponseTypes {
                        request: sample_types(
                            config.request_message_type_details(),
                            request_schema,
                        ),
                        response: sample_types(
                            config.response_message_type_details(),
                            response_schema,
                        ),
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
            _ => return Err(DescriptionError::UnsupportedPattern),
        };

        Ok(Self {
            hash: *static_config.service_hash(),
            settings: ServiceSettings::new(*static_config.name(), pattern),
            types,
        })
    }
}

fn sample_types(types: &MessageTypeDetails, schema: Option<Vec<u8>>) -> SampleTypes {
    let mut payload = TypeDescription::from(&types.payload);
    if let Some(schema) = schema {
        payload.identifier = TypeIdentifier::Flatbuffer(Schema::new(schema));
    }
    SampleTypes {
        payload,
        user_header: (&types.user_header).into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::node::NodeBuilder;
    use iceoryx2::port::event_id::EventId;
    use iceoryx2::service::local;
    use iceoryx2::service::marker::Flatbuffer;
    use iceoryx2::service::messaging_pattern::MessagingPattern as Pattern;
    use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
    use iceoryx2::testing::{generate_isolated_config, generate_service_name};
    use iceoryx2_bb_elementary::alignment::Alignment;
    use iceoryx2_bb_posix::testing::create_typed_file_with_content;
    use iceoryx2_bb_testing::assert_that;

    /// The static config of the service `name` of `pattern` in the
    /// system of `config`.
    pub(super) fn static_config_of<S: iceoryx2::service::Service>(
        name: &ServiceName,
        config: &Config,
        pattern: Pattern,
    ) -> StaticConfig {
        S::details(name, config, pattern)
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

        let static_config =
            static_config_of::<local::Service>(&service_name, &config, Pattern::PublishSubscribe);
        let sut = ServiceDescription::load::<local::Service>(&config, &static_config)
            .expect("pattern is carried");

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
            eq ServiceTypes::PublishSubscribe(SampleTypes {
                payload: (&TypeDetail::new::<u64>(TypeVariant::FixedSize)).into(),
                user_header: (&TypeDetail::new::<()>(TypeVariant::FixedSize)).into(),
            })
        );
    }

    #[test]
    fn publish_subscribe_with_custom_payload_alignment_is_described() {
        const PAYLOAD_ALIGNMENT: usize = 16;

        let config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let service_name = generate_service_name();

        let _service = node
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .payload_alignment(Alignment::new(PAYLOAD_ALIGNMENT).expect("power of two"))
            .create()
            .expect("service is created");

        let static_config =
            static_config_of::<local::Service>(&service_name, &config, Pattern::PublishSubscribe);
        let sut = ServiceDescription::load::<local::Service>(&config, &static_config)
            .expect("pattern is carried");

        let types = sut.types.publish_subscribe();
        assert_that!(types.payload.alignment, eq PAYLOAD_ALIGNMENT);
        assert_that!(types.type_details(), is_ok);
    }

    #[test]
    fn a_flatbuffer_payload_is_identified_by_its_schema() {
        const SCHEMA: &str = "the binary schema";

        let config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let service_name = generate_service_name();
        let schema_file = create_typed_file_with_content(SCHEMA, "bfbs");
        let _service = node
            .service_builder(&service_name)
            .publish_subscribe::<Flatbuffer<u64>>()
            .flatbuffer_schema_path(schema_file.path().expect("the file has a path"))
            .create()
            .expect("service is created");

        let static_config =
            static_config_of::<local::Service>(&service_name, &config, Pattern::PublishSubscribe);
        let sut = ServiceDescription::load::<local::Service>(&config, &static_config)
            .expect("pattern is carried");

        let ServiceTypes::PublishSubscribe(types) = sut.types else {
            panic!("a publish-subscribe service");
        };
        assert_that!(
            types.payload.identifier,
            eq TypeIdentifier::Flatbuffer(Schema::new(SCHEMA.as_bytes().to_vec()))
        );
    }

    #[test]
    fn flatbuffer_requests_and_responses_are_identified_by_their_schemas() {
        const REQUEST_SCHEMA: &str = "the request schema";
        const RESPONSE_SCHEMA: &str = "the response schema";

        let config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let service_name = generate_service_name();
        let request_schema = create_typed_file_with_content(REQUEST_SCHEMA, "bfbs");
        let response_schema = create_typed_file_with_content(RESPONSE_SCHEMA, "bfbs");
        let _service = node
            .service_builder(&service_name)
            .request_response::<Flatbuffer<u64>, Flatbuffer<u32>>()
            .request_flatbuffer_schema_path(request_schema.path().expect("the file has a path"))
            .response_flatbuffer_schema_path(response_schema.path().expect("the file has a path"))
            .create()
            .expect("service is created");

        let static_config =
            static_config_of::<local::Service>(&service_name, &config, Pattern::RequestResponse);
        let sut = ServiceDescription::load::<local::Service>(&config, &static_config)
            .expect("pattern is carried");

        let ServiceTypes::RequestResponse(types) = sut.types else {
            panic!("a request-response service");
        };
        assert_that!(
            types.request.payload.identifier,
            eq TypeIdentifier::Flatbuffer(Schema::new(REQUEST_SCHEMA.as_bytes().to_vec()))
        );
        assert_that!(
            types.response.payload.identifier,
            eq TypeIdentifier::Flatbuffer(Schema::new(RESPONSE_SCHEMA.as_bytes().to_vec()))
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

        let static_config =
            static_config_of::<local::Service>(&service_name, &config, Pattern::Event);
        let sut = ServiceDescription::load::<local::Service>(&config, &static_config)
            .expect("pattern is carried");

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

        let static_config =
            static_config_of::<local::Service>(&service_name, &config, Pattern::PublishSubscribe);
        let described = ServiceDescription::load::<local::Service>(&config, &static_config)
            .expect("pattern is carried");
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
    fn request_response_static_config_is_described() {
        const MAX_SERVERS: usize = 3;
        const MAX_CLIENTS: usize = 5;
        const MAX_NODES: usize = 17;
        const MAX_ACTIVE_REQUESTS_PER_CLIENT: usize = 7;
        const MAX_RESPONSE_BUFFER_SIZE: usize = 11;
        const MAX_BORROWED_RESPONSES_PER_PENDING_RESPONSE: usize = 4;
        const MAX_LOANED_REQUESTS: usize = 6;
        const SAFE_OVERFLOW_FOR_REQUESTS: bool = false;
        const SAFE_OVERFLOW_FOR_RESPONSES: bool = true;
        const FIRE_AND_FORGET_REQUESTS: bool = true;

        let config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let service_name = generate_service_name();

        let _service = node
            .service_builder(&service_name)
            .request_response::<u64, u32>()
            .max_servers(MAX_SERVERS)
            .max_clients(MAX_CLIENTS)
            .max_nodes(MAX_NODES)
            .max_active_requests_per_client(MAX_ACTIVE_REQUESTS_PER_CLIENT)
            .max_response_buffer_size(MAX_RESPONSE_BUFFER_SIZE)
            .max_borrowed_responses_per_pending_response(
                MAX_BORROWED_RESPONSES_PER_PENDING_RESPONSE,
            )
            .max_loaned_requests(MAX_LOANED_REQUESTS)
            .enable_safe_overflow_for_requests(SAFE_OVERFLOW_FOR_REQUESTS)
            .enable_safe_overflow_for_responses(SAFE_OVERFLOW_FOR_RESPONSES)
            .enable_fire_and_forget_requests(FIRE_AND_FORGET_REQUESTS)
            .create()
            .expect("service is created");

        let static_config =
            static_config_of::<local::Service>(&service_name, &config, Pattern::RequestResponse);
        let sut = ServiceDescription::load::<local::Service>(&config, &static_config)
            .expect("pattern is carried");

        assert_that!(sut.name(), eq service_name);
        assert_that!(sut.hash, eq * static_config.service_hash());
        assert_that!(
            sut.settings.pattern,
            eq PatternSettings::RequestResponse(RequestResponseSettings {
                max_servers: MAX_SERVERS,
                max_clients: MAX_CLIENTS,
                max_nodes: MAX_NODES,
                max_active_requests_per_client: MAX_ACTIVE_REQUESTS_PER_CLIENT,
                max_response_buffer_size: MAX_RESPONSE_BUFFER_SIZE,
                max_borrowed_responses_per_pending_response:
                    MAX_BORROWED_RESPONSES_PER_PENDING_RESPONSE,
                max_loaned_requests: MAX_LOANED_REQUESTS,
                safe_overflow_for_requests: SAFE_OVERFLOW_FOR_REQUESTS,
                safe_overflow_for_responses: SAFE_OVERFLOW_FOR_RESPONSES,
                fire_and_forget_requests: FIRE_AND_FORGET_REQUESTS,
            })
        );
        let no_user_header = TypeDescription::from(&TypeDetail::new::<()>(TypeVariant::FixedSize));
        assert_that!(
            sut.types,
            eq ServiceTypes::RequestResponse(RequestResponseTypes {
                request: SampleTypes {
                    payload: (&TypeDetail::new::<u64>(TypeVariant::FixedSize)).into(),
                    user_header: no_user_header.clone(),
                },
                response: SampleTypes {
                    payload: (&TypeDetail::new::<u32>(TypeVariant::FixedSize)).into(),
                    user_header: no_user_header,
                },
            })
        );
    }

    #[test]
    fn compose_request_response_computes_the_same_hash_as_the_service() {
        let config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let service_name = generate_service_name();
        let _service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create()
            .expect("service is created");

        let static_config =
            static_config_of::<local::Service>(&service_name, &config, Pattern::RequestResponse);
        let described = ServiceDescription::load::<local::Service>(&config, &static_config)
            .expect("pattern is carried");
        let PatternSettings::RequestResponse(settings) = described.settings.pattern else {
            panic!("a request-response service");
        };
        let ServiceTypes::RequestResponse(types) = described.types else {
            panic!("a request-response service");
        };
        let sut = ServiceDescription::compose_request_response::<local::Service>(
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
