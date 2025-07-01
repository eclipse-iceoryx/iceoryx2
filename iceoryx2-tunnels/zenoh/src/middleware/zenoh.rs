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

use crate::keys;

use iceoryx2::service::static_config::StaticConfig as ServiceConfig;
use iceoryx2_bb_log::error;

use zenoh::handlers::FifoChannel;
use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Publisher;
use zenoh::pubsub::Subscriber;
use zenoh::qos::Reliability;
use zenoh::sample::Locality;
use zenoh::sample::Sample;
use zenoh::Session;
use zenoh::Wait;

/// Creates a Zenoh publisher to send payloads from iceoryx2 services to remote hosts.
pub(crate) fn create_publisher<'a>(
    session: &Session,
    service_config: &ServiceConfig,
) -> Result<Publisher<'a>, zenoh::Error> {
    let key = keys::publish_subscribe(service_config.service_id());

    let z_publisher = session
        .declare_publisher(key)
        .allowed_destination(Locality::Remote)
        .reliability(Reliability::Reliable)
        .wait()?;

    Ok(z_publisher)
}

/// Creates a Zenoh subscriber to receive payloads from remote hosts for a particular iceoryx2 service.
pub(crate) fn create_subscriber(
    session: &Session,
    service_config: &ServiceConfig,
) -> Result<Subscriber<FifoChannelHandler<Sample>>, zenoh::Error> {
    let key = keys::publish_subscribe(service_config.service_id());

    // TODO(correctness): Make handler type and properties configurable
    let subscriber = session
        .declare_subscriber(key)
        .with(FifoChannel::new(10))
        .allowed_origin(Locality::Remote)
        .wait()?;

    Ok(subscriber)
}

// TODO(correctness): Optimize for notifications
/// Creates a Zenoh notifier to send notifications from iceoryx2 services to remote hosts.
pub(crate) fn create_notifier<'a>(
    session: &Session,
    service_config: &ServiceConfig,
) -> Result<Publisher<'a>, zenoh::Error> {
    let key = keys::event(service_config.service_id());

    let notifier = session
        .declare_publisher(key.clone())
        .allowed_destination(Locality::Remote)
        .reliability(Reliability::Reliable)
        .wait()?;

    Ok(notifier)
}

// TODO(correctness): Optimize for notifications
/// Creates a Zenoh listener to receive notifications from remote hosts for a particular iceoryx2 service.
pub(crate) fn create_listener(
    session: &Session,
    service_config: &ServiceConfig,
) -> Result<Subscriber<FifoChannelHandler<Sample>>, zenoh::Error> {
    let key = keys::event(service_config.service_id());

    // TODO(correctness): Make handler type and properties configurable
    let listener = session
        .declare_subscriber(key.clone())
        .with(FifoChannel::new(10))
        .allowed_origin(Locality::Remote)
        .wait()?;

    Ok(listener)
}

/// Announces an iceoryx service over Zenoh to make it discoverable by remote hosts.
pub(crate) fn announce_service(
    session: &Session,
    service_config: &ServiceConfig,
) -> Result<(), zenoh::Error> {
    let key = keys::service_details(service_config.service_id());
    let service_config_serialized = serde_json::to_string(&service_config)?;

    // Notify all current hosts.
    session
        .put(key.clone(), service_config_serialized.clone())
        .allowed_destination(Locality::Remote)
        .wait()?;

    // Set up a queryable to respond to future hosts.
    session
        .declare_queryable(key.clone())
        .callback(move |query| {
            let _ = query
                .reply(key.clone(), service_config_serialized.clone())
                .wait()
                .inspect_err(|e| {
                    error!("Failed to announce service {}: {}", key, e);
                });
        })
        .allowed_origin(Locality::Remote)
        .background()
        .wait()?;

    Ok(())
}
