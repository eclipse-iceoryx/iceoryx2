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
use std::sync::Arc;

use iceoryx2::service::Service;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_gateway_backend::traits::Mapping;
use iceoryx2_gateway_backend::types::discovery::{DiscoveryUpdate, DiscoveryUpdateRef};
use iceoryx2_gateway_backend::types::service_description::ServiceDescription;
use iceoryx2_log::{error, fail, warn};

use zenoh::{
    Session, Wait,
    handlers::FifoChannelHandler,
    liveliness::LivelinessToken,
    pubsub::Subscriber,
    query::{Queryable, Reply},
    sample::{Locality, Sample, SampleKind},
};

use crate::descriptor::{EncodedDescription, Fingerprint, ServiceDescriptor};
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
pub struct Discovery<S: Service, M: Mapping<EndpointDescription = ServiceDescription>> {
    session: Session,
    mapping: Arc<M>,
    // Subscribes to liveliness changes for service announcements.
    subscriber: Subscriber<FifoChannelHandler<Sample>>,
    // Keeps track of services that have been announced locally.
    announced: BTreeMap<ServiceHash, AnnouncedService>,
    // Cache for replies to requests for remote service details.
    // Replies are filled asynchronously by Zenoh but only processed on
    // subsequent discover calls. Enables non-blocking implementation.
    pending: BTreeMap<ServiceDescriptor, FifoChannelHandler<Reply>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service, M: Mapping<EndpointDescription = ServiceDescription>> Discovery<S, M> {
    pub fn create(session: &Session, mapping: Arc<M>) -> Result<Self, CreationError> {
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
            mapping,
            subscriber,
            announced: BTreeMap::new(),
            pending: BTreeMap::new(),
            _phantom: core::marker::PhantomData,
        })
    }
}

impl<S: Service, M: Mapping<EndpointDescription = ServiceDescription>>
    iceoryx2_gateway_backend::traits::Discovery for Discovery<S, M>
{
    type DiscoveryError = DiscoveryError;
    type AnnouncementError = AnnouncementError;

    fn announce(&mut self, update: DiscoveryUpdateRef<'_>) -> Result<(), Self::AnnouncementError> {
        match update {
            DiscoveryUpdateRef::Added(description) => {
                if self.mapping.remote(description).is_none() {
                    return Ok(());
                }
                self.announce_added(description)
            }
            DiscoveryUpdateRef::Removed(service_hash) => self.announce_removed(service_hash),
        }
    }

    fn discover<E: core::error::Error, F: FnMut(DiscoveryUpdate) -> Result<(), E>>(
        &mut self,
        mut process_discovery: F,
    ) -> Result<(), DiscoveryError> {
        for (descriptor, kind) in self.receive_liveliness_changes()? {
            match kind {
                // A new service was detected on the network. Request its
                // details. The reply is picked up by a subsequent discover call.
                // Subject to network latency.
                SampleKind::Put => self.request_service_description(&descriptor)?,
                // The service disappeared. Remove it from the gateway and
                // cancel any in-flight details request.
                SampleKind::Delete => {
                    let service_hash = descriptor.service_hash;
                    self.pending.remove(&descriptor);

                    fail!(
                        from self,
                        when process_discovery(DiscoveryUpdate::Removed(service_hash)),
                        with DiscoveryError::DiscoveryProcessing,
                        "Failed to process Removed discovery event for {}", service_hash.as_str()
                    );
                }
            }
        }

        // Poll the pending detail queries. A service is added to the
        // gateway once the reply with its details has been received.
        let (resolved, failed) = self.receive_service_descriptions();
        for descriptor in failed {
            self.request_service_description(&descriptor)?;
        }
        for (descriptor, description) in resolved {
            self.pending.remove(&descriptor);
            let Some(description) = self.mapping.local::<S>(&description) else {
                continue;
            };
            let service_hash = description.service_hash;
            fail!(
                from self,
                when process_discovery(DiscoveryUpdate::Added(description)),
                with DiscoveryError::DiscoveryProcessing,
                "Failed to process Added discovery event for {}", service_hash.as_str()
            );
        }

        Ok(())
    }
}

impl<S: Service, M: Mapping<EndpointDescription = ServiceDescription>> Discovery<S, M> {
    /// Makes a service available to remote peers by declaring a queryable for
    /// its details and a liveliness token at its key.
    ///
    /// No-op if the service has already been announced.
    fn announce_added(
        &mut self,
        description: &ServiceDescription,
    ) -> Result<(), AnnouncementError> {
        let service_hash = description.service_hash;

        if self.announced.contains_key(&service_hash) {
            return Ok(());
        }

        let encoded = fail!(
            from self,
            when EncodedDescription::encode(description),
            with AnnouncementError::Serialization,
            "Failed to encode service description"
        );
        let descriptor = ServiceDescriptor::new(description.service_hash, encoded.fingerprint());
        let key = keys::service_description(&descriptor);

        // Declare the queryable **before** the liveliness token. Peers
        // receive the token's Put as soon as it is declared.
        let queryable = self.declare_queryable(&key, encoded)?;
        let token = self.declare_liveliness_token(&key)?;

        self.announced.insert(
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
    fn announce_removed(&mut self, service_hash: &ServiceHash) -> Result<(), AnnouncementError> {
        self.announced.remove(service_hash);
        Ok(())
    }

    /// Declares a queryable that responds to remote peers' `get` requests for
    /// a service's ServiceDescription with the pre-encoded payload.
    fn declare_queryable(
        &self,
        key: &str,
        encoded: EncodedDescription,
    ) -> Result<Queryable<()>, AnnouncementError> {
        let reply_key = key.to_string();
        let queryable = fail!(
            from self,
            when self.session
                .declare_queryable(key)
                .callback(move |query| {
                    let _ = query
                        .reply(reply_key.clone(), encoded.clone().into_bytes())
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

    /// Drains the liveliness samples received since the last call and
    /// returns the (descriptor, change) pairs in order of arrival. Samples
    /// with unparsable keys are skipped.
    fn receive_liveliness_changes(
        &self,
    ) -> Result<Vec<(ServiceDescriptor, SampleKind)>, DiscoveryError> {
        let mut changes = Vec::new();
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
            let descriptor = match keys::parse_service_description(key) {
                Some(descriptor) => descriptor,
                None => {
                    warn!("Skipping liveliness sample with unparsable key: {}", key);
                    continue;
                }
            };

            changes.push((descriptor, sample.kind()));
        }
        Ok(changes)
    }

    /// Issues a Zenoh `get` for the service's [`ServiceDescription`] and stores the
    /// reply handler.
    /// Replies are picked up by [`Discovery::receive_service_descriptions`] on
    /// subsequent iterations.
    fn request_service_description(
        &mut self,
        descriptor: &ServiceDescriptor,
    ) -> Result<(), DiscoveryError> {
        let key = keys::service_description(descriptor);
        let handler = fail!(
            from self,
            when self.session
                .get(key.clone())
                .allowed_destination(Locality::Remote)
                .wait(),
            with DiscoveryError::DiscoveryQuery,
            "Failed to query for static config of {}", key
        );
        self.pending.insert(descriptor.clone(), handler);
        Ok(())
    }

    /// Receives service descriptions from previously-issued queries.
    /// Returns the resolved queries as (descriptor, description) pairs and
    /// the descriptors whose query channel closed without a usable reply.
    fn receive_service_descriptions(
        &self,
    ) -> (
        Vec<(ServiceDescriptor, ServiceDescription)>,
        Vec<ServiceDescriptor>,
    ) {
        let mut resolved: Vec<(ServiceDescriptor, ServiceDescription)> = Vec::new();
        let mut failed: Vec<ServiceDescriptor> = Vec::new();

        for (descriptor, handler) in self.pending.iter() {
            match check_pending(descriptor, handler) {
                Ok(Some(description)) => resolved.push((descriptor.clone(), description)),
                Ok(None) => {}
                Err(_) => failed.push(descriptor.clone()),
            }
        }

        (resolved, failed)
    }
}

#[derive(Debug)]
struct Disconnected;

/// Polls a query for the described service. Replies whose payload does not
/// match the descriptor's fingerprint are skipped.
fn check_pending(
    descriptor: &ServiceDescriptor,
    handler: &FifoChannelHandler<Reply>,
) -> Result<Option<ServiceDescription>, Disconnected> {
    loop {
        let Some(reply) = handler.try_recv().map_err(|_| Disconnected)? else {
            return Ok(None);
        };

        let sample = match reply.result() {
            Ok(sample) => sample,
            Err(e) => {
                warn!(
                    "Skipping erroneous reply for service {}: {:?}",
                    descriptor.service_hash.as_str(),
                    e
                );
                continue;
            }
        };

        let bytes = sample.payload().to_bytes();
        if Fingerprint::digest(&bytes) != descriptor.fingerprint {
            warn!(
                "Skipping reply for service {} that does not match the announced fingerprint",
                descriptor.service_hash.as_str()
            );
            continue;
        }

        match EncodedDescription::decode(&bytes) {
            Ok(description) => return Ok(Some(description)),
            Err(e) => warn!(
                "Skipping unparsable reply for service {}: {}",
                descriptor.service_hash.as_str(),
                e
            ),
        }
    }
}
