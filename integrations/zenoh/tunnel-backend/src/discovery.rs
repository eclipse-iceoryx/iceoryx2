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
    query::{Queryable, Reply},
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
    // Cache for replies to requests for remote service details.
    // Replies are filled asynchronously by Zenoh but only processed on
    // subsequent discover calls. Enables non-blocking implementation.
    pending: RefCell<BTreeMap<ServiceHash, FifoChannelHandler<Reply>>>,
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
            pending: RefCell::new(BTreeMap::new()),
        })
    }
}

impl iceoryx2_services_tunnel_backend::traits::Discovery for Discovery {
    type DiscoveryError = DiscoveryError;
    type AnnouncementError = AnnouncementError;

    fn announce(&self, discovery_event: DiscoveryEvent) -> Result<(), Self::AnnouncementError> {
        match discovery_event {
            DiscoveryEvent::Added(static_config) => self.announce_added(static_config),
            DiscoveryEvent::Removed(service_hash) => self.announce_removed(service_hash),
        }
    }

    fn discover<E: core::error::Error, F: FnMut(&DiscoveryEvent) -> Result<(), E>>(
        &self,
        mut process_discovery: F,
    ) -> Result<(), DiscoveryError> {
        // Reaction to a new service being detected on the network: request
        // its details. The reply will be handled by `process_service_details`.
        // Subject to network latency.
        let on_service_added =
            |service_hash: &ServiceHash| self.request_service_details(service_hash);

        // Removes a service from the tunnel. Called on a liveliness Delete;
        // also cancels any in-flight details request.
        let on_service_removed = |service_hash: &ServiceHash| {
            self.pending.borrow_mut().remove(service_hash);

            fail!(
                from self,
                when process_discovery(&DiscoveryEvent::Removed(*service_hash)),
                with DiscoveryError::DiscoveryProcessing,
                "Failed to process Removed discovery event for {}", service_hash.as_str()
            );
            Ok(())
        };

        self.process_liveliness_changes(on_service_added, on_service_removed)?;

        // Adds a service to the tunnel. Called once the reply with the
        // service details of a discovered remote service has been received.
        let on_service_details = |static_config: StaticConfig| {
            let service_hash = *static_config.service_hash();
            fail!(
                from self,
                when process_discovery(&DiscoveryEvent::Added(static_config)),
                with DiscoveryError::DiscoveryProcessing,
                "Failed to process Added discovery event for {}", service_hash.as_str()
            );
            Ok(())
        };

        self.process_service_details(on_service_details)?;

        Ok(())
    }
}

impl Discovery {
    /// Makes a service available to remote peers by declaring a queryable for
    /// its details and a liveliness token at its key.
    ///
    /// No-op if the service has already been announced.
    fn announce_added(&self, static_config: StaticConfig) -> Result<(), AnnouncementError> {
        let service_hash = *static_config.service_hash();

        if self.announced.borrow().contains_key(&service_hash) {
            return Ok(());
        }

        let key = keys::service_details(&service_hash);
        let serialized = fail!(
            from self,
            when serde_json::to_string(&static_config),
            with AnnouncementError::Serialization,
            "Failed to serialize service config"
        );

        // Declare the queryable **before** the liveliness token. Peers
        // receive the token's Put as soon as it is declared.
        let queryable = self.declare_queryable(&key, serialized)?;
        let token = self.declare_liveliness_token(&key)?;

        self.announced.borrow_mut().insert(
            service_hash,
            AnnouncedService {
                _token: token,
                _queryable: queryable,
            },
        );
        Ok(())
    }

    /// Withdraws a service announcement by dropping its queryable and
    /// liveliness token, propagating a liveliness Delete to remote peers.
    fn announce_removed(&self, service_hash: ServiceHash) -> Result<(), AnnouncementError> {
        self.announced.borrow_mut().remove(&service_hash);
        Ok(())
    }

    /// Declares a queryable that responds to remote peers' `get` requests for
    /// a service's StaticConfig with the pre-serialised JSON payload.
    fn declare_queryable(
        &self,
        key: &str,
        serialized: String,
    ) -> Result<Queryable<()>, AnnouncementError> {
        let reply_key = key.to_string();
        let queryable = fail!(
            from self,
            when self.session
                .declare_queryable(key)
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
        Ok(queryable)
    }

    /// Declares the liveliness token that signals this service's presence to
    /// remote subscribers. Dropping the returned token propagates a Delete.
    fn declare_liveliness_token(&self, key: &str) -> Result<LivelinessToken, AnnouncementError> {
        let token = fail!(
            from self,
            when self.session
                .liveliness()
                .declare_token(key)
                .wait(),
            with AnnouncementError::LivelinessTokenDeclaration,
            "Failed to declare liveliness token for service"
        );
        Ok(token)
    }

    /// Drains the liveliness subscriber non-blocking and dispatches each
    /// sample to one of the provided callbacks based on its kind. `Put`
    /// samples (a service appeared on the network) go to `on_service_added`;
    /// `Delete` samples (a service vanished) go to `on_service_removed`.
    fn process_liveliness_changes<OnServiceAdded, OnServiceRemoved>(
        &self,
        mut on_service_added: OnServiceAdded,
        mut on_service_removed: OnServiceRemoved,
    ) -> Result<(), DiscoveryError>
    where
        OnServiceAdded: FnMut(&ServiceHash) -> Result<(), DiscoveryError>,
        OnServiceRemoved: FnMut(&ServiceHash) -> Result<(), DiscoveryError>,
    {
        loop {
            let sample = match fail!(
                from self,
                when self.subscriber.try_recv(),
                with DiscoveryError::SubscriberReceive,
                "Failed to receive liveliness sample"
            ) {
                Some(sample) => sample,
                None => break,
            };

            let key: &str = sample.key_expr().as_ref();
            let service_hash = match parse_service_hash(key) {
                Some(h) => h,
                None => {
                    warn!("Skipping liveliness sample with unparseable key: {}", key);
                    continue;
                }
            };

            match sample.kind() {
                SampleKind::Put => on_service_added(&service_hash)?,
                SampleKind::Delete => on_service_removed(&service_hash)?,
            }
        }
        Ok(())
    }

    /// Issues a Zenoh `get` for the service's StaticConfig and queues the
    /// reply handler under `pending`. Returns immediately; replies are
    /// processed by [`Discovery::process_service_details`] on subsequent calls.
    fn request_service_details(&self, service_hash: &ServiceHash) -> Result<(), DiscoveryError> {
        let key = keys::service_details(service_hash);
        let handler = fail!(
            from self,
            when self.session
                .get(key.clone())
                .allowed_destination(Locality::Remote)
                .wait(),
            with DiscoveryError::DiscoveryQuery,
            "Failed to query for static config of {}", key
        );
        self.pending.borrow_mut().insert(*service_hash, handler);
        Ok(())
    }

    /// Drains the cached handlers for replies received over the network and
    /// dispatches each resolved [`StaticConfig`] to `on_service_details`.
    /// Re-issues the query for any handler whose channel closed without a
    /// usable reply.
    fn process_service_details<OnServiceDetails>(
        &self,
        mut on_service_details: OnServiceDetails,
    ) -> Result<(), DiscoveryError>
    where
        OnServiceDetails: FnMut(StaticConfig) -> Result<(), DiscoveryError>,
    {
        let mut resolved: Vec<StaticConfig> = Vec::new();
        let mut failed: Vec<ServiceHash> = Vec::new();

        // Check status of pending queries.
        {
            let pending = self.pending.borrow();
            for (hash, handler) in pending.iter() {
                match check_pending(hash, handler) {
                    Ok(Some(static_config)) => resolved.push(static_config),
                    Ok(None) => {}
                    Err(()) => failed.push(*hash),
                }
            }
        }

        // Drop resolved entries from pending.
        {
            let mut pending = self.pending.borrow_mut();
            for static_config in &resolved {
                pending.remove(static_config.service_hash());
            }
        }

        // Re-issue queries for failed requests.
        for hash in failed {
            self.request_service_details(&hash)?;
        }

        // Processed received service details.
        for static_config in resolved {
            on_service_details(static_config)?;
        }

        Ok(())
    }
}

fn check_pending(
    hash: &ServiceHash,
    handler: &FifoChannelHandler<Reply>,
) -> Result<Option<StaticConfig>, ()> {
    loop {
        match handler.try_recv() {
            Ok(Some(reply)) => match reply.result() {
                Ok(sample) => {
                    match serde_json::from_slice::<StaticConfig>(&sample.payload().to_bytes()) {
                        Ok(static_config) => return Ok(Some(static_config)),
                        Err(e) => warn!(
                            "Skipping unparseable reply for service {}: {}",
                            hash.as_str(),
                            e
                        ),
                    }
                }
                Err(e) => warn!("Erroneous reply for service {}: {:?}", hash.as_str(), e),
            },
            Ok(None) => return Ok(None),
            Err(_) => return Err(()),
        }
    }
}

fn parse_service_hash(key: &str) -> Option<ServiceHash> {
    let suffix = key.rsplit('/').next()?;
    ServiceHash::try_from(suffix).ok()
}
