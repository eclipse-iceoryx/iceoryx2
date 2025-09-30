// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use iceoryx2::node::Node;
use iceoryx2::port::listener::Listener;
use iceoryx2::port::listener::ListenerCreateError;
use iceoryx2::port::notifier::Notifier;
use iceoryx2::port::notifier::NotifierCreateError;
use iceoryx2::port::publisher::Publisher;
use iceoryx2::port::publisher::PublisherCreateError;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::port::subscriber::SubscriberCreateError;
use iceoryx2::prelude::AllocationStrategy;
use iceoryx2::service::builder::event::EventOpenOrCreateError;
use iceoryx2::service::builder::publish_subscribe::PublishSubscribeOpenOrCreateError;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::port_factory::event::PortFactory as EventService;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory as PublishSubscribeService;
use iceoryx2::service::static_config::StaticConfig as ServiceConfig;
use iceoryx2_bb_log::fail;

/// Creates an iceoryx2 publish-subscribe service matching the provided service configuration.
pub(crate) fn create_publish_subscribe_service<ServiceType: iceoryx2::service::Service>(
    node: &Node<ServiceType>,
    service_config: &ServiceConfig,
) -> Result<
    PublishSubscribeService<ServiceType, [CustomPayloadMarker], CustomHeaderMarker>,
    PublishSubscribeOpenOrCreateError,
> {
    let publish_subscribe_config = service_config.publish_subscribe();
    let service = unsafe {
        fail!(
            from "create_publish_subscribe_service()",
            when node.service_builder(service_config.name())
                    .publish_subscribe::<[CustomPayloadMarker]>()
                    .user_header::<CustomHeaderMarker>()
                    .__internal_set_user_header_type_details(
                        &publish_subscribe_config.message_type_details().user_header,
                    )
                    .__internal_set_payload_type_details(
                        &publish_subscribe_config.message_type_details().payload,
                    )
                    .enable_safe_overflow(publish_subscribe_config.has_safe_overflow())
                    .history_size(publish_subscribe_config.history_size())
                    .max_nodes(publish_subscribe_config.max_nodes())
                    .max_publishers(publish_subscribe_config.max_publishers())
                    .max_subscribers(publish_subscribe_config.max_subscribers())
                    .subscriber_max_buffer_size(publish_subscribe_config.subscriber_max_buffer_size())
                    .subscriber_max_borrowed_samples(
                        publish_subscribe_config.subscriber_max_borrowed_samples(),
                    )
                    .open_or_create(),
            "failed to open or create publish-subscribe service"
        )
    };

    Ok(service)
}

/// Creates an iceoryx event service matching the provided service configuration.
pub(crate) fn create_event_service<ServiceType: iceoryx2::service::Service>(
    node: &Node<ServiceType>,
    service_config: &ServiceConfig,
) -> Result<EventService<ServiceType>, EventOpenOrCreateError> {
    let event_config = service_config.event();
    let service = fail!(
        from "create_event_service()",
        when node
            .service_builder(service_config.name())
            .event()
            .max_nodes(event_config.max_nodes())
            .max_listeners(event_config.max_listeners())
            .max_notifiers(event_config.max_notifiers())
            .event_id_max_value(event_config.event_id_max_value())
            .open_or_create(),
        "failed to open or create event service"
    );

    Ok(service)
}

/// Creates an iceoryx publisher to the provided service.
pub(crate) fn create_publisher<ServiceType: iceoryx2::service::Service>(
    service: &PublishSubscribeService<ServiceType, [CustomPayloadMarker], CustomHeaderMarker>,
) -> Result<Publisher<ServiceType, [CustomPayloadMarker], CustomHeaderMarker>, PublisherCreateError>
{
    let publisher = fail!(
        from "create_publisher()",
        when service
            .publisher_builder()
            .allocation_strategy(AllocationStrategy::PowerOfTwo)
            .create(),
        "failed to create publisher"
    );
    Ok(publisher)
}

/// Creates an iceoryx subscriber to the provided service.
pub(crate) fn create_subscriber<ServiceType: iceoryx2::service::Service>(
    service: &PublishSubscribeService<ServiceType, [CustomPayloadMarker], CustomHeaderMarker>,
) -> Result<Subscriber<ServiceType, [CustomPayloadMarker], CustomHeaderMarker>, SubscriberCreateError>
{
    let subscriber = fail!(
        from "create_subscriber()",
        when service.subscriber_builder().create(),
        "failed to create subscriber"
    );
    Ok(subscriber)
}

/// Creates an iceoryx notifier to the provided service.
pub(crate) fn create_notifier<ServiceType: iceoryx2::service::Service>(
    service: &EventService<ServiceType>,
) -> Result<Notifier<ServiceType>, NotifierCreateError> {
    let notifier = fail!(
        from "create_notifier()",
        when service.notifier_builder().create(),
        "failed to create notifier"
    );
    Ok(notifier)
}

/// Creates an iceoryx listener for the provided service.
pub(crate) fn create_listener<ServiceType: iceoryx2::service::Service>(
    service: &EventService<ServiceType>,
) -> Result<Listener<ServiceType>, ListenerCreateError> {
    let listener = fail!(
        from "create_listener()",
        when service.listener_builder().create(),
        "failed to create listener"
    );
    Ok(listener)
}
