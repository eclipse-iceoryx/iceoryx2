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

mod connection;
pub mod keys;
mod tunnel;

pub(crate) use connection::*;
pub use tunnel::*;

use iceoryx2::node::Node as IceoryxNode;
use iceoryx2::port::listener::Listener as IceoryxListener;
use iceoryx2::port::listener::ListenerCreateError;
use iceoryx2::port::notifier::Notifier as IceoryxNotifier;
use iceoryx2::port::notifier::NotifierCreateError;
use iceoryx2::port::publisher::Publisher as IceoryxPublisher;
use iceoryx2::port::publisher::PublisherCreateError;
use iceoryx2::port::subscriber::Subscriber as IceoryxSubscriber;
use iceoryx2::port::subscriber::SubscriberCreateError;
use iceoryx2::prelude::AllocationStrategy;
use iceoryx2::service::builder::event::EventOpenOrCreateError;
use iceoryx2::service::builder::publish_subscribe::PublishSubscribeOpenOrCreateError;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::port_factory::event::PortFactory as IceoryxEventService;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory as IceoryxPublishSubscribeService;
use iceoryx2::service::static_config::StaticConfig as IceoryxServiceConfig;
use iceoryx2_bb_log::error;
use iceoryx2_bb_log::info;

use zenoh::handlers::FifoChannel;
use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Publisher as ZenohPublisher;
use zenoh::pubsub::Subscriber as ZenohSubscriber;
use zenoh::sample::Locality;
use zenoh::sample::Sample;
use zenoh::Session as ZenohSession;
use zenoh::Wait;

/// Creates an iceoryx2 publish-subscribe service matching the provided service configuration.
pub(crate) fn iox_create_publish_subscribe_service<Service: iceoryx2::service::Service>(
    iox_node: &IceoryxNode<Service>,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<
    IceoryxPublishSubscribeService<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    PublishSubscribeOpenOrCreateError,
> {
    let iox_publish_subscribe_config = iox_service_config.publish_subscribe();
    let iox_service = unsafe {
        iox_node
            .service_builder(iox_service_config.name())
            .publish_subscribe::<[CustomPayloadMarker]>()
            .user_header::<CustomHeaderMarker>()
            .__internal_set_user_header_type_details(
                &iox_publish_subscribe_config
                    .message_type_details()
                    .user_header,
            )
            .__internal_set_payload_type_details(
                &iox_publish_subscribe_config.message_type_details().payload,
            )
            .enable_safe_overflow(iox_publish_subscribe_config.has_safe_overflow())
            .history_size(iox_publish_subscribe_config.history_size())
            .max_publishers(iox_publish_subscribe_config.max_publishers())
            .max_subscribers(iox_publish_subscribe_config.max_subscribers())
            .subscriber_max_buffer_size(iox_publish_subscribe_config.subscriber_max_buffer_size())
            .subscriber_max_buffer_size(
                iox_publish_subscribe_config.subscriber_max_borrowed_samples(),
            )
            .open_or_create()?
    };

    Ok(iox_service)
}

/// Creates an iceoryx2 event service matching the provided service configuration.
pub(crate) fn iox_create_event_service<Service: iceoryx2::service::Service>(
    iox_node: &IceoryxNode<Service>,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<IceoryxEventService<Service>, EventOpenOrCreateError> {
    // TODO(correctness): Use properties of existing event services to prevent clashing
    let iox_service = iox_node
        .service_builder(iox_service_config.name())
        .event()
        .open_or_create()?;

    Ok(iox_service)
}

/// Creates an iceoryx2 publisher to the provided service.
pub(crate) fn iox_create_publisher<Service: iceoryx2::service::Service>(
    iox_publish_subscribe_service: &IceoryxPublishSubscribeService<
        Service,
        [CustomPayloadMarker],
        CustomHeaderMarker,
    >,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<
    IceoryxPublisher<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    PublisherCreateError,
> {
    let iox_publisher = iox_publish_subscribe_service
        .publisher_builder()
        .allocation_strategy(AllocationStrategy::PowerOfTwo)
        .create()?;

    info!(
        "CREATED(iceoryx2): Publisher {} [{}]",
        iox_service_config.service_id().as_str(),
        iox_service_config.name()
    );

    Ok(iox_publisher)
}

/// Creates an iceoryx2 subscriber to the provided service.
pub(crate) fn iox_create_subscriber<Service: iceoryx2::service::Service>(
    iox_publish_subscribe_service: &IceoryxPublishSubscribeService<
        Service,
        [CustomPayloadMarker],
        CustomHeaderMarker,
    >,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<
    IceoryxSubscriber<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    SubscriberCreateError,
> {
    let iox_subscriber = iox_publish_subscribe_service
        .subscriber_builder()
        .create()?;

    info!(
        "CREATED(iceoryx2): Subscriber {} [{}]",
        iox_service_config.service_id().as_str(),
        iox_service_config.name()
    );

    Ok(iox_subscriber)
}

/// Creates an iceoryx2 notifier to the provided service.
pub(crate) fn iox_create_notifier<Service: iceoryx2::service::Service>(
    iox_event_service: &IceoryxEventService<Service>,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<IceoryxNotifier<Service>, NotifierCreateError> {
    let iox_notifier = iox_event_service.notifier_builder().create()?;

    info!(
        "CREATED(iceoryx2): Notifier {} [{}]",
        iox_service_config.service_id().as_str(),
        iox_service_config.name()
    );

    Ok(iox_notifier)
}

/// Creates an iceoryx2 listener for the provided service.
pub(crate) fn iox_create_listener<Service: iceoryx2::service::Service>(
    iox_event_service: &IceoryxEventService<Service>,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<IceoryxListener<Service>, ListenerCreateError> {
    let iox_listener = iox_event_service.listener_builder().create()?;

    info!(
        "CREATED(iceoryx2): Listener {} [{}]",
        iox_service_config.service_id().as_str(),
        iox_service_config.name()
    );

    Ok(iox_listener)
}

/// Creates a Zenoh publisher to send payloads from iceoryx2 services to remote hosts.
pub(crate) fn z_create_publisher<'a>(
    z_session: &ZenohSession,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<ZenohPublisher<'a>, zenoh::Error> {
    let z_key = keys::publish_subscribe(iox_service_config.service_id());
    let z_publisher = z_session
        .declare_publisher(z_key.clone())
        .allowed_destination(Locality::Remote)
        .wait()?;
    info!(
        "CREATED(zenoh): Publisher {} [{}]",
        z_key,
        iox_service_config.name()
    );

    Ok(z_publisher)
}

/// Creates a Zenoh subscriber to receive payloads from remote hosts for a particular iceoryx2 service.
pub(crate) fn z_create_subscriber(
    z_session: &ZenohSession,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<ZenohSubscriber<FifoChannelHandler<Sample>>, zenoh::Error> {
    let z_key = keys::publish_subscribe(iox_service_config.service_id());
    let z_subscriber = z_session
        .declare_subscriber(z_key.clone())
        .with(FifoChannel::new(10))
        .allowed_origin(Locality::Remote)
        .wait()?;
    info!(
        "CREATED(zenoh): Subscriber {} [{}]",
        z_key,
        iox_service_config.name()
    );

    Ok(z_subscriber)
}

// TODO(correctness): Optimize for notifications
/// Creates a Zenoh publisher to send notifications from iceoryx2 services to remote hosts.
pub(crate) fn z_create_notifier<'a>(
    z_session: &ZenohSession,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<ZenohPublisher<'a>, zenoh::Error> {
    let z_key = keys::event(iox_service_config.service_id());
    let z_notifier = z_session
        .declare_publisher(z_key.clone())
        .allowed_destination(Locality::Remote)
        .wait()?;
    info!(
        "CREATED(zenoh): Notifier {} [{}]",
        z_key,
        iox_service_config.name()
    );

    Ok(z_notifier)
}

// TODO(correctness): Optimize for notifications
/// Creates a Zenoh listener to receive notifications from remote hosts for a particular iceoryx2 service.
pub(crate) fn z_create_listener(
    z_session: &ZenohSession,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<ZenohSubscriber<FifoChannelHandler<Sample>>, zenoh::Error> {
    let z_key = keys::event(iox_service_config.service_id());
    let z_listener = z_session
        .declare_subscriber(z_key.clone())
        .with(FifoChannel::new(10))
        .allowed_origin(Locality::Remote)
        .wait()?;
    info!(
        "CREATED(zenoh): Listener {} [{}]",
        z_key,
        iox_service_config.name()
    );

    Ok(z_listener)
}

/// Announces an iceoryx2 service over Zenoh to make it discoverable by remote hosts.
pub(crate) fn z_announce_service(
    z_session: &ZenohSession,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<(), zenoh::Error> {
    let z_key = keys::service_details(iox_service_config.service_id());
    let iox_service_config_serialized = serde_json::to_string(&iox_service_config)?;

    match iox_service_config.messaging_pattern() {
        iceoryx2::service::static_config::messaging_pattern::MessagingPattern::PublishSubscribe(
            _,
        ) => {
            info!(
                "ANNOUNCING(zenoh): PublishSubscribe {} [{}]",
                z_key,
                iox_service_config.name()
            );
        }
        iceoryx2::service::static_config::messaging_pattern::MessagingPattern::Event(_) => {
            info!(
                "ANNOUNCING(zenoh): Event {} [{}]",
                z_key,
                iox_service_config.name()
            );
        }
        _ => {
            // Not Supported. Nothing to do.
        }
    }

    z_session
        .declare_queryable(z_key.clone())
        .callback(move |query| {
            if let Err(e) = query
                .reply(z_key.clone(), iox_service_config_serialized.clone())
                .wait()
            {
                error!("Failed to reply to query {}: {}", z_key, e);
            }
        })
        .allowed_origin(Locality::Remote)
        .background()
        .wait()?;

    Ok(())
}
