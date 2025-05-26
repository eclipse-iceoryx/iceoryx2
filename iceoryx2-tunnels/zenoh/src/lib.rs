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

mod data_stream;
pub mod keys;
mod tunnel;

pub(crate) use data_stream::*;
pub use tunnel::*;

use iceoryx2::node::Node as IceoryxNode;
use iceoryx2::port::publisher::Publisher as IceoryxPublisher;
use iceoryx2::port::publisher::PublisherCreateError;
use iceoryx2::port::subscriber::Subscriber as IceoryxSubscriber;
use iceoryx2::port::subscriber::SubscriberCreateError;
use iceoryx2::prelude::*;
use iceoryx2::service::builder::publish_subscribe::PublishSubscribeOpenOrCreateError;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory;
use iceoryx2::service::static_config::StaticConfig;
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

pub(crate) fn iox_create_service<Service: iceoryx2::service::Service>(
    iox_node: &IceoryxNode<Service>,
    iox_service_config: &StaticConfig,
) -> Result<
    PortFactory<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    PublishSubscribeOpenOrCreateError,
> {
    let iox_service = unsafe {
        iox_node
            .service_builder(iox_service_config.name())
            .publish_subscribe::<[CustomPayloadMarker]>()
            .user_header::<CustomHeaderMarker>()
            .__internal_set_user_header_type_details(
                &iox_service_config
                    .publish_subscribe()
                    .message_type_details()
                    .user_header,
            )
            .__internal_set_payload_type_details(
                &iox_service_config
                    .publish_subscribe()
                    .message_type_details()
                    .payload,
            )
            .open_or_create()?
    };

    Ok(iox_service)
}

pub(crate) fn iox_create_publisher<Service: iceoryx2::service::Service>(
    iox_service: &PortFactory<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    iox_service_config: &StaticConfig,
) -> Result<
    IceoryxPublisher<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    PublisherCreateError,
> {
    let iox_publisher = iox_service
        .publisher_builder()
        .allocation_strategy(AllocationStrategy::PowerOfTwo)
        .create()?;

    info!(
        "NEW PUBLISHER (iceoryx2): {} [{}]",
        iox_service_config.service_id().as_str(),
        iox_service_config.name()
    );

    Ok(iox_publisher)
}

pub(crate) fn iox_create_subscriber<Service: iceoryx2::service::Service>(
    iox_service: &PortFactory<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    iox_service_config: &StaticConfig,
) -> Result<
    IceoryxSubscriber<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    SubscriberCreateError,
> {
    let iox_subscriber = iox_service.subscriber_builder().create()?;

    info!(
        "NEW SUBSCRIBER (iceoryx2): {} [{}]",
        iox_service_config.service_id().as_str(),
        iox_service_config.name()
    );

    Ok(iox_subscriber)
}

pub(crate) fn z_create_publisher<'a>(
    z_session: &ZenohSession,
    iox_service_config: &StaticConfig,
) -> Result<ZenohPublisher<'a>, zenoh::Error> {
    let z_key = keys::data_stream(iox_service_config.service_id());
    info!("NEW PUBLISHER (zenoh): {}", z_key.clone());
    let z_publisher = z_session.declare_publisher(z_key).wait()?;

    Ok(z_publisher)
}

pub(crate) fn z_create_subscriber(
    z_session: &ZenohSession,
    iox_service_config: &StaticConfig,
) -> Result<ZenohSubscriber<FifoChannelHandler<Sample>>, zenoh::Error> {
    let z_key = keys::data_stream(iox_service_config.service_id());
    let z_subscriber = z_session
        .declare_subscriber(z_key)
        .with(FifoChannel::new(10))
        .allowed_origin(Locality::Remote)
        .wait()?;

    Ok(z_subscriber)
}

pub(crate) fn z_announce_service(
    z_session: &ZenohSession,
    iox_service_config: &StaticConfig,
) -> Result<(), zenoh::Error> {
    let z_key = keys::service(iox_service_config.service_id());
    match serde_json::to_string(&iox_service_config) {
        Ok(iox_static_details_json) => {
            z_session
                .declare_queryable(z_key.clone())
                .callback(move |query| {
                    if let Err(e) = query
                        .reply(query.key_expr().clone(), &iox_static_details_json)
                        .wait()
                    {
                        error!("Failed to reply to query for service info: {}", e);
                    }
                })
                .background()
                .wait()?;

            info!("ANNOUNCING (zenoh): {}", z_key);
        }
        Err(e) => {
            error!("Failed to serialize static details to JSON: {}", e);
        }
    }
    Ok(())
}

pub(crate) fn z_query_services(
    z_session: &ZenohSession,
) -> Result<Vec<StaticConfig>, zenoh::Error> {
    let mut iox_remote_static_details = Vec::new();

    let replies = z_session
        .get(keys::all_services())
        .allowed_destination(Locality::Remote)
        .wait()?;

    while let Ok(reply) = replies.try_recv() {
        match reply {
            // Case: Reply contains a sample (actual data from a service)
            Some(sample) => match sample.result() {
                // Case: Sample contains valid data that can be processed
                Ok(sample) => {
                    match serde_json::from_slice::<StaticConfig>(&sample.payload().to_bytes()) {
                        Ok(iox_static_details) => {
                            if !iox_remote_static_details
                                .iter()
                                .any(|details: &StaticConfig| {
                                    details.service_id() == iox_static_details.service_id()
                                })
                            {
                                iox_remote_static_details.push(iox_static_details.clone());
                            }
                        }
                        Err(e) => {
                            error!("Failed to deserialize static details: {}", e);
                        }
                    }
                }
                // Case: Sample contains an error (e.g., malformed data)
                Err(e) => {
                    error!("Invalid sample: {}", e);
                }
            },
            // Case: Reply exists but contains no sample (empty response)
            None => { /* Nothing to do */ }
        }
    }

    Ok(iox_remote_static_details)
}
