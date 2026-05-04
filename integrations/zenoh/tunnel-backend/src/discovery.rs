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

use std::collections::BTreeMap;

use iceoryx2::service::{service_hash::ServiceHash, static_config::StaticConfig};
use iceoryx2_bb_concurrency::cell::RefCell;
use iceoryx2_log::{error, fail, warn};
use iceoryx2_services_common::DiscoveryEvent;

use zenoh::{
    Session, Wait,
    handlers::FifoChannelHandler,
    liveliness::LivelinessToken,
    pubsub::Subscriber,
    query::Queryable,
    sample::{Locality, Sample, SampleKind},
};

use crate::keys;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    UnableToSubscribeToRemoteDiscoveryUpdates,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    SubscriberReceive,
    DiscoveryQuery,
    DiscoveryProcessing,
}

impl core::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DiscoveryError::{self:?}")
    }
}

impl core::error::Error for DiscoveryError {}

#[derive(Debug)]
struct AnnouncedService {
    // Indicates that the service is available locally. Dropping triggers a
    // liveliness `Delete` for remote subscribers.
    _token: LivelinessToken,
    // Responds to remote peers who query for the static config of the service.
    _queryable: Queryable<()>,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum AnnouncementError {
    Serialization,
    LivelinessTokenDeclaration,
    QueryableDeclaration,
}

impl core::fmt::Display for AnnouncementError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AnnouncementError::{self:?}")
    }
}

impl core::error::Error for AnnouncementError {}

#[derive(Debug)]
pub struct Discovery {
    session: Session,
    // Subscribes to liveliness changes for service announcements.
    subscriber: Subscriber<FifoChannelHandler<Sample>>,
    // Keeps track of services that have been announced locally.
    announced: RefCell<BTreeMap<ServiceHash, AnnouncedService>>,
}

impl Discovery {
    pub fn create(session: &Session) -> Result<Self, CreationError> {
        let origin = "Discovery::create()";

        let subscriber = fail!(
            from origin,
            when session
                    .liveliness()
                    .declare_subscriber(keys::service_discovery())
                    .history(true)
                    .wait(),
            with CreationError::UnableToSubscribeToRemoteDiscoveryUpdates,
            "Failed to create subscriber for remote discovery updates"
        );

        Ok(Self {
            session: session.clone(),
            subscriber,
            announced: RefCell::new(BTreeMap::new()),
        })
    }
}

impl iceoryx2_services_tunnel_backend::traits::Discovery for Discovery {
    type DiscoveryError = DiscoveryError;
    type AnnouncementError = AnnouncementError;

    fn announce(&self, discovery_event: DiscoveryEvent) -> Result<(), Self::AnnouncementError> {
        match discovery_event {
            DiscoveryEvent::Added(static_config) => {
                let service_hash = *static_config.service_hash();
                let mut announced = self.announced.borrow_mut();
                if announced.contains_key(&service_hash) {
                    return Ok(());
                }

                let key = keys::service_details(&service_hash);

                let serialized = fail!(
                    from self,
                    when serde_json::to_string(&static_config),
                    with AnnouncementError::Serialization,
                    "Failed to serialize service config"
                );

                let token = fail!(
                    from self,
                    when self.session
                        .liveliness()
                        .declare_token(key.clone())
                        .wait(),
                    with AnnouncementError::LivelinessTokenDeclaration,
                    "Failed to declare liveliness token for service"
                );

                let reply_key = key.clone();
                let queryable = fail!(
                    from self,
                    when self.session
                        .declare_queryable(key.clone())
                        .callback(move |query| {
                            let _ = query
                                .reply(reply_key.clone(), serialized.clone())
                                .wait()
                                .inspect_err(|e| {
                                    error!("Failed to reply with service details for {}: {}", reply_key, e);
                                });
                        })
                        .allowed_origin(Locality::Remote)
                        .wait(),
                    with AnnouncementError::QueryableDeclaration,
                    "Failed to declare queryable for service"
                );

                announced.insert(
                    service_hash,
                    AnnouncedService {
                        _token: token,
                        _queryable: queryable,
                    },
                );
            }
            DiscoveryEvent::Removed(service_hash) => {
                self.announced.borrow_mut().remove(&service_hash);
            }
        }

        Ok(())
    }

    fn discover<E: core::error::Error, F: FnMut(&DiscoveryEvent) -> Result<(), E>>(
        &self,
        mut process_discovery: F,
    ) -> Result<(), DiscoveryError> {
        loop {
            let sample = match fail!(
                from self,
                when self.subscriber.try_recv(),
                with DiscoveryError::SubscriberReceive,
                "Failed to receive liveliness sample"
            ) {
                Some(sample) => sample,
                None => return Ok(()),
            };

            let key_str: &str = sample.key_expr().as_ref();
            let service_hash = match parse_service_hash(key_str) {
                Some(h) => h,
                None => {
                    warn!(
                        "Skipping liveliness sample with unparseable key: {}",
                        key_str
                    );
                    continue;
                }
            };

            match sample.kind() {
                SampleKind::Put => {
                    if let Some(static_config) = self.fetch_static_config(&service_hash)? {
                        fail!(
                            from self,
                            when process_discovery(&DiscoveryEvent::Added(static_config)),
                            with DiscoveryError::DiscoveryProcessing,
                            "Failed to process Added discovery event for {}", service_hash.as_str()
                        );
                    }
                }
                SampleKind::Delete => {
                    fail!(
                        from self,
                        when process_discovery(&DiscoveryEvent::Removed(service_hash)),
                        with DiscoveryError::DiscoveryProcessing,
                        "Failed to process Removed discovery event for {}", service_hash.as_str()
                    );
                }
            }
        }
    }
}

impl Discovery {
    fn fetch_static_config(
        &self,
        service_hash: &ServiceHash,
    ) -> Result<Option<StaticConfig>, DiscoveryError> {
        let key = keys::service_details(service_hash);

        let replies = fail!(
            from self,
            when self.session
                .get(key.clone())
                .allowed_destination(Locality::Remote)
                .wait(),
            with DiscoveryError::DiscoveryQuery,
            "Failed to query for static config of {}", key
        );

        for reply in replies {
            match reply.result() {
                Ok(reply_sample) => {
                    match serde_json::from_slice::<StaticConfig>(
                        &reply_sample.payload().to_bytes(),
                    ) {
                        Ok(static_config) => return Ok(Some(static_config)),
                        Err(e) => warn!("Skipping unparseable reply for {}: {}", key, e),
                    }
                }
                Err(e) => warn!("Erroneous reply for {}: {:?}", key, e),
            }
        }

        warn!(
            "No usable reply received for service {} after liveliness Put",
            service_hash.as_str()
        );
        Ok(None)
    }
}

fn parse_service_hash(key: &str) -> Option<ServiceHash> {
    let suffix = key.rsplit('/').next()?;
    ServiceHash::try_from(suffix).ok()
}
