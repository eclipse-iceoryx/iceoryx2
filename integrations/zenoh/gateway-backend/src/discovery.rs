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

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use iceoryx2::service::Service;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_gateway_backend::traits::Mapping;
use iceoryx2_gateway_backend::types::discovery::{Announcement, DiscoveryUpdate};
use iceoryx2_gateway_backend::types::identity::GatewayId;
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

use crate::wire::description::EncodedDescription;
use crate::wire::descriptor::ServiceDescriptor;
use crate::wire::fingerprint::Fingerprint;
use crate::wire::keys;

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

// A local service announced to be remotely discoverable.
#[derive(Debug)]
struct AnnouncedService {
    // The service as offered locally.
    description: ServiceDescription,
    // The descriptor the service is announced under.
    descriptor: ServiceDescriptor,
    // A liveliness token which zenoh makes visible to all remotes. Declaring
    // it triggers a `Put` event, dropping it triggers a `Delete` event.
    _token: LivelinessToken,
    // A queryable that remotes can ping for the description of the announced
    // service, in the remote representation according to the mapping.
    _queryable: Queryable<()>,
}

/// The local services announced to be remotely discoverable, keyed by the
/// local hash.
///
/// NOTE: The local hash may differ from the (remote) hash representing this
///       service remotely, depending on the mapping strategy.
#[derive(Debug)]
struct LocalAnnouncements {
    services: BTreeMap<ServiceHash, AnnouncedService>,
}

impl LocalAnnouncements {
    fn new() -> Self {
        Self {
            services: BTreeMap::new(),
        }
    }

    /// Returns true if a service with the specified `local_hash` has been
    /// announced.
    fn contains(&self, local_hash: &ServiceHash) -> bool {
        self.services.contains_key(local_hash)
    }

    /// Insert a local service that has been announced to the tracked set.
    fn insert(&mut self, local_hash: ServiceHash, service: AnnouncedService) {
        self.services.insert(local_hash, service);
    }

    /// Remove a local service that had been announced from the tracked set.
    fn remove(&mut self, local_hash: &ServiceHash) {
        self.services.remove(local_hash);
    }

    /// The local description of the service announced under `descriptor`,
    /// if any.
    fn announced_under(&self, descriptor: &ServiceDescriptor) -> Option<&ServiceDescription> {
        self.services
            .values()
            .find(|service| service.descriptor == *descriptor)
            .map(|service| &service.description)
    }
}

/// The gateways offering each remotely announced description.
#[derive(Debug)]
struct RemoteOffers {
    gateways: BTreeMap<ServiceDescriptor, BTreeSet<GatewayId>>,
}

impl RemoteOffers {
    fn new() -> Self {
        Self {
            gateways: BTreeMap::new(),
        }
    }

    /// Records that a gateway offers the description.
    ///
    /// Returns true if the gateway was not offering it.
    fn add(&mut self, remote_descriptor: ServiceDescriptor, gateway: GatewayId) -> bool {
        self.gateways
            .entry(remote_descriptor)
            .or_default()
            .insert(gateway)
    }

    /// Records that a gateway no longer offers the given description.
    ///
    /// Returns true if the gateway was offering it.
    fn remove(&mut self, remote_descriptor: &ServiceDescriptor, gateway_id: &GatewayId) -> bool {
        let Some(gateways) = self.gateways.get_mut(remote_descriptor) else {
            return false;
        };
        let removed = gateways.remove(gateway_id);
        if gateways.is_empty() {
            self.gateways.remove(remote_descriptor);
        }
        removed
    }

    /// Returns true if at least one gateway offers the description.
    fn is_offered(&self, remote_descriptor: &ServiceDescriptor) -> bool {
        self.gateways.contains_key(remote_descriptor)
    }

    /// Returns true if `gateway` offers the description.
    fn is_offered_by(&self, remote_descriptor: &ServiceDescriptor, gateway: &GatewayId) -> bool {
        self.gateways
            .get(remote_descriptor)
            .is_some_and(|gateways| gateways.contains(gateway))
    }

    /// The gateways offering the description.
    fn gateways(&self, remote_descriptor: &ServiceDescriptor) -> Vec<GatewayId> {
        self.gateways
            .get(remote_descriptor)
            .map(|gateways| gateways.iter().copied().collect())
            .unwrap_or_default()
    }

    /// The offered descriptors of the service with `remote_hash`. More than
    /// one means the service is announced with conflicting descriptions.
    fn descriptors_of(
        &self,
        remote_hash: &ServiceHash,
    ) -> impl Iterator<Item = &ServiceDescriptor> {
        self.gateways
            .keys()
            .filter(move |descriptor| descriptor.service_hash == *remote_hash)
    }
}

/// What is known locally about a remotely announced description.
#[allow(clippy::large_enum_variant)] // the large, Resolved variant is the target state
#[derive(Debug)]
enum RemoteDescription {
    // Query in flight. Replies are filled asynchronously by zenoh.
    Pending(FifoChannelHandler<Reply>),
    // Received and mapped into the local representation.
    Resolved(ServiceDescription),
    // Received but the mapping excludes the service.
    Excluded,
}

/// What each remotely announced descriptor is locally, the mapping
/// materialized. Keyed by remote descriptors, values are in the local
/// representation.
#[derive(Debug)]
struct RemoteDescriptions {
    by_descriptor: BTreeMap<ServiceDescriptor, RemoteDescription>,
}

impl RemoteDescriptions {
    fn new() -> Self {
        Self {
            by_descriptor: BTreeMap::new(),
        }
    }

    /// Returns true if the descriptor is tracked.
    fn contains(&self, remote_descriptor: &ServiceDescriptor) -> bool {
        self.by_descriptor.contains_key(remote_descriptor)
    }

    /// Tracks or replaces the description of `remote_descriptor`.
    fn set(&mut self, remote_descriptor: ServiceDescriptor, description: RemoteDescription) {
        self.by_descriptor.insert(remote_descriptor, description);
    }

    /// Stops tracking the description of `remote_descriptor`.
    fn remove(&mut self, remote_descriptor: &ServiceDescriptor) {
        self.by_descriptor.remove(remote_descriptor);
    }

    /// The local description of `remote_descriptor`, if resolved.
    fn resolved(&self, remote_descriptor: &ServiceDescriptor) -> Option<&ServiceDescription> {
        match self.by_descriptor.get(remote_descriptor) {
            Some(RemoteDescription::Resolved(local_description)) => Some(local_description),
            _ => None,
        }
    }

    /// Retrieves the result of pending queries.
    ///
    /// Returns a tuple containing answered queries and failed queries.
    fn poll_pending(
        &self,
    ) -> (
        Vec<(ServiceDescriptor, ServiceDescription)>,
        Vec<ServiceDescriptor>,
    ) {
        let mut answered = Vec::new();
        let mut failed = Vec::new();

        for (remote_descriptor, description) in self.by_descriptor.iter() {
            let RemoteDescription::Pending(query) = description else {
                continue;
            };
            match check_pending(remote_descriptor, query) {
                Ok(Some(remote_description)) => {
                    answered.push((remote_descriptor.clone(), remote_description))
                }
                Ok(None) => {}
                Err(_) => failed.push(remote_descriptor.clone()),
            }
        }

        (answered, failed)
    }
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
    local_announcements: LocalAnnouncements,
    remote_offers: RemoteOffers,
    remote_descriptions: RemoteDescriptions,
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
            local_announcements: LocalAnnouncements::new(),
            remote_offers: RemoteOffers::new(),
            remote_descriptions: RemoteDescriptions::new(),
            _phantom: core::marker::PhantomData,
        })
    }

    /// Makes a service discoverable by remote peers.
    ///
    /// Declares a liveliness token encoding the [`ServiceDescriptor`] and
    /// [`GatewayId`], and a queryable for remotes to retrieve the
    /// [`ServiceDescription`].
    ///
    /// No-op if the service has already been announced or the mapping
    /// excludes it.
    fn announce_added(
        &mut self,
        own_id: GatewayId,
        local_description: &ServiceDescription,
    ) -> Result<(), AnnouncementError> {
        let local_hash = local_description.service_hash;
        if self.local_announcements.contains(&local_hash) {
            return Ok(());
        }
        let Some(remote_description) = self.mapping.remote(local_description) else {
            return Ok(());
        };

        let encoded = fail!(
            from self,
            when EncodedDescription::encode(&remote_description),
            with AnnouncementError::Serialization,
            "Failed to encode service description"
        );
        let descriptor =
            ServiceDescriptor::new(remote_description.service_hash, encoded.fingerprint());
        let key = keys::service_description(&descriptor, &own_id);

        // Declare the queryable **before** the liveliness token. Peers
        // receive the token's Put as soon as it is declared.
        let queryable = self.declare_queryable(&key, encoded)?;
        let token = self.declare_liveliness_token(&key)?;

        self.local_announcements.insert(
            local_hash,
            AnnouncedService {
                description: local_description.clone(),
                descriptor,
                _token: token,
                _queryable: queryable,
            },
        );
        Ok(())
    }

    /// Stops making a service discoverable by remote peers.
    ///
    /// Drops the liveliness token, which propagates the event to all remotes
    /// and stops replying to queries for the [`ServiceDescription`].
    fn announce_removed(&mut self, local_hash: &ServiceHash) {
        self.local_announcements.remove(local_hash);
    }

    /// Declares a queryable that responds to remote peers' `get` requests for
    /// a service's [`ServiceDescription`] with the pre-encoded payload.
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
    /// remote gateways. Dropping the returned token propagates a Delete.
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

    /// Issues a zenoh `get` for the described service's details at any
    /// announcing gateway. Every gateway announcing the descriptor holds the
    /// identical description, so the first reply serves them all.
    fn request_service_description(
        &self,
        remote_descriptor: &ServiceDescriptor,
    ) -> Result<FifoChannelHandler<Reply>, DiscoveryError> {
        let key = keys::service_description_any(remote_descriptor);
        let handler = fail!(
            from self,
            when self.session
                .get(key.clone())
                .allowed_destination(Locality::Remote)
                .wait(),
            with DiscoveryError::DiscoveryQuery,
            "Failed to query for static config of {}", key
        );
        Ok(handler)
    }

    /// Discover remote services by observing liveliness token changes. Tokens
    /// are announced at keys encoding the service descriptor and the id of
    /// the gateway offering it. Tokens announced by `own_id` are ignored.
    fn discover_remote_services<E, F>(
        &mut self,
        own_id: GatewayId,
        process_discovery: &mut F,
    ) -> Result<(), DiscoveryError>
    where
        E: core::error::Error,
        F: FnMut(DiscoveryUpdate) -> Result<(), E>,
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
            let Some((remote_descriptor, gateway_id)) = keys::parse_service_description(key) else {
                warn!("Skipping liveliness sample with unparsable key: {}", key);
                continue;
            };
            if gateway_id == own_id {
                continue;
            }

            match sample.kind() {
                SampleKind::Put => {
                    self.on_added_service(remote_descriptor, gateway_id, process_discovery)?
                }
                SampleKind::Delete => {
                    self.on_removed_service(remote_descriptor, gateway_id, process_discovery)?
                }
            }
        }
        Ok(())
    }

    /// Tracks a remotely discovered service and sends a query for its
    /// [`ServiceDescription`] if it is not already tracked.
    ///
    /// Provides a [`DiscoveryUpdate`] if the [`ServiceDescription`] is
    /// already known.
    fn on_added_service<E, F>(
        &mut self,
        remote_descriptor: ServiceDescriptor,
        gateway_id: GatewayId,
        process_discovery: &mut F,
    ) -> Result<(), DiscoveryError>
    where
        E: core::error::Error,
        F: FnMut(DiscoveryUpdate) -> Result<(), E>,
    {
        // Track the discovered service. Request the description if not
        // already known.
        if !self.remote_descriptions.contains(&remote_descriptor) {
            let description = match self.local_announcements.announced_under(&remote_descriptor) {
                Some(local_description) => RemoteDescription::Resolved(local_description.clone()),
                None => RemoteDescription::Pending(
                    self.request_service_description(&remote_descriptor)?,
                ),
            };
            self.remote_descriptions
                .set(remote_descriptor.clone(), description);
        }

        // Track that the gateway is offering the description.
        if !self
            .remote_offers
            .add(remote_descriptor.clone(), gateway_id)
        {
            return Ok(());
        }

        // Provide the update to the caller.
        let Some(local_description) = self
            .remote_descriptions
            .resolved(&remote_descriptor)
            .cloned()
        else {
            return Ok(());
        };
        fail!(
            from self,
            when process_discovery(DiscoveryUpdate::Added(gateway_id, local_description)),
            with DiscoveryError::DiscoveryProcessing,
            "Failed to process Added discovery event for {}", remote_descriptor.service_hash.as_str()
        );

        Ok(())
    }

    /// Removes a tracked gateway from a discovered service and provides a
    /// [`DiscoveryUpdate`] for processing by the caller.
    ///
    /// If no gateways are offering the service, stop tracking it.
    fn on_removed_service<E, F>(
        &mut self,
        remote_descriptor: ServiceDescriptor,
        gateway: GatewayId,
        process_discovery: &mut F,
    ) -> Result<(), DiscoveryError>
    where
        E: core::error::Error,
        F: FnMut(DiscoveryUpdate) -> Result<(), E>,
    {
        if !self.remote_offers.remove(&remote_descriptor, &gateway) {
            return Ok(());
        }

        let local_hash = self
            .remote_descriptions
            .resolved(&remote_descriptor)
            .map(|local_description| local_description.service_hash);

        // Stop tracking the description when there are no more gateways
        // offering it.
        if !self.remote_offers.is_offered(&remote_descriptor) {
            self.remote_descriptions.remove(&remote_descriptor);
        }

        // A gateway that re-announced the service under another description
        // had its offer replaced when that description was resolved. Its
        // stale token must not withdraw the new offer.
        let offer_replaced = self
            .remote_offers
            .descriptors_of(&remote_descriptor.service_hash)
            .any(|descriptor| {
                self.remote_offers.is_offered_by(descriptor, &gateway)
                    && self.remote_descriptions.resolved(descriptor).is_some()
            });
        if offer_replaced {
            return Ok(());
        }

        // Provide the update to the caller.
        if let Some(local_hash) = local_hash {
            fail!(
                from self,
                when process_discovery(DiscoveryUpdate::Removed(gateway, local_hash)),
                with DiscoveryError::DiscoveryProcessing,
                "Failed to process Removed discovery event for {}", remote_descriptor.service_hash.as_str()
            );
        }

        Ok(())
    }

    /// Receive the [`ServiceDescription`] of remotely discovered services.
    ///
    /// Queries for the [`ServiceDescription`] of discovered services are
    /// issued on discovery but are replied to asynchronously. Checks for
    /// queries which have completed or failed and processes them.
    ///
    /// Issues a [`DiscoveryUpdate`] for completed queries and stores the
    /// local representation of the [`ServiceDescription`]. Failed queries
    /// are re-issued.
    fn receive_service_descriptions<E, F>(
        &mut self,
        process_discovery: &mut F,
    ) -> Result<(), DiscoveryError>
    where
        E: core::error::Error,
        F: FnMut(DiscoveryUpdate) -> Result<(), E>,
    {
        let (answered, failed) = self.remote_descriptions.poll_pending();

        for remote_descriptor in failed {
            let query = self.request_service_description(&remote_descriptor)?;
            self.remote_descriptions
                .set(remote_descriptor, RemoteDescription::Pending(query));
        }

        for (remote_descriptor, remote_description) in answered {
            let Some(local_description) = self.mapping.local::<S>(&remote_description) else {
                self.remote_descriptions
                    .set(remote_descriptor, RemoteDescription::Excluded);
                continue;
            };
            self.remote_descriptions.set(
                remote_descriptor.clone(),
                RemoteDescription::Resolved(local_description.clone()),
            );

            for gateway in self.remote_offers.gateways(&remote_descriptor) {
                fail!(
                    from self,
                    when process_discovery(DiscoveryUpdate::Added(gateway, local_description.clone())),
                    with DiscoveryError::DiscoveryProcessing,
                    "Failed to process Added discovery event for {}", remote_descriptor.service_hash.as_str()
                );
            }
        }

        Ok(())
    }
}

impl<S: Service, M: Mapping<EndpointDescription = ServiceDescription>>
    iceoryx2_gateway_backend::traits::Discovery for Discovery<S, M>
{
    type DiscoveryError = DiscoveryError;
    type AnnouncementError = AnnouncementError;

    fn announce(
        &mut self,
        own_id: GatewayId,
        announcement: Announcement<'_>,
    ) -> Result<(), Self::AnnouncementError> {
        match announcement {
            Announcement::Added(local_description) => {
                self.announce_added(own_id, local_description)
            }
            Announcement::Removed(local_hash) => {
                self.announce_removed(local_hash);
                Ok(())
            }
        }
    }

    fn discover<E: core::error::Error, F: FnMut(DiscoveryUpdate) -> Result<(), E>>(
        &mut self,
        own_id: GatewayId,
        mut process_discovery: F,
    ) -> Result<(), DiscoveryError> {
        self.discover_remote_services(own_id, &mut process_discovery)?;
        self.receive_service_descriptions(&mut process_discovery)?;
        Ok(())
    }
}

#[derive(Debug)]
struct Disconnected;

/// Polls a query for the described service. Replies whose payload does not
/// match the descriptor's fingerprint are skipped.
fn check_pending(
    remote_descriptor: &ServiceDescriptor,
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
                    remote_descriptor.service_hash.as_str(),
                    e
                );
                continue;
            }
        };

        let bytes = sample.payload().to_bytes();
        if Fingerprint::digest(&bytes) != remote_descriptor.fingerprint {
            warn!(
                "Skipping reply for service {} that does not match the announced fingerprint",
                remote_descriptor.service_hash.as_str()
            );
            continue;
        }

        match EncodedDescription::decode(&bytes) {
            Ok(remote_description) => return Ok(Some(remote_description)),
            Err(e) => warn!(
                "Skipping unparsable reply for service {}: {}",
                remote_descriptor.service_hash.as_str(),
                e
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::node::NodeBuilder;
    use iceoryx2::service::ipc;
    use iceoryx2::service::service_name::ServiceName;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_gateway_backend::types::identity::{BACKEND_ID_LENGTH, BackendId};
    use iceoryx2_gateway_backend::types::service_description::{
        EventDescription, EventSettings, PatternDescription, PortSettings,
    };

    use crate::wire::descriptor::describe;

    fn gateway_id(discriminator: u8) -> GatewayId {
        let node = NodeBuilder::new()
            .create::<ipc::Service>()
            .expect("node creation succeeds");
        GatewayId::new(
            *node.id(),
            BackendId::new([discriminator; BACKEND_ID_LENGTH]),
        )
    }

    /// Two descriptions of one service whose settings differ.
    fn differing_descriptions(name: &str) -> (ServiceDescription, ServiceDescription) {
        let default_settings = ServiceDescription::new::<ipc::Service>(
            ServiceName::new(name).expect("valid service name"),
            PatternDescription::Event(EventDescription {
                settings: PortSettings::LocalDefaults,
            }),
        );
        let custom_settings = ServiceDescription::new::<ipc::Service>(
            ServiceName::new(name).expect("valid service name"),
            PatternDescription::Event(EventDescription {
                settings: PortSettings::Value(EventSettings {
                    max_notifiers: 1,
                    max_listeners: 1,
                    max_nodes: 1,
                    event_id_max_value: 1,
                    deadline: None,
                    notifier_created_event: None,
                    notifier_dropped_event: None,
                    notifier_dead_event: None,
                }),
            }),
        );
        (default_settings, custom_settings)
    }

    mod remote_offers {
        use super::*;

        #[test]
        fn tracks_offering_gateways_per_descriptor() {
            let first = gateway_id(1);
            let second = gateway_id(2);
            let (description, _) = differing_descriptions("discovery/offers");
            let descriptor = describe(&description).expect("description is encodable");

            let mut sut = RemoteOffers::new();
            assert_that!(sut.add(descriptor.clone(), first), eq true);
            assert_that!(sut.add(descriptor.clone(), first), eq false);
            assert_that!(sut.add(descriptor.clone(), second), eq true);

            assert_that!(sut.is_offered_by(&descriptor, &first), eq true);
            assert_that!(sut.is_offered_by(&descriptor, &second), eq true);
        }

        #[test]
        fn offer_ends_with_removal_of_last_gateway() {
            let gateway = gateway_id(1);
            let (description, _) = differing_descriptions("discovery/last-gateway");
            let descriptor = describe(&description).expect("description is encodable");

            let mut sut = RemoteOffers::new();
            sut.add(descriptor.clone(), gateway);
            assert_that!(sut.is_offered(&descriptor), eq true);

            assert_that!(sut.remove(&descriptor, &gateway), eq true);
            assert_that!(sut.is_offered(&descriptor), eq false);
            assert_that!(sut.remove(&descriptor, &gateway), eq false);
        }

        #[test]
        fn descriptors_of_yields_every_descriptor_of_the_service() {
            let gateway = gateway_id(1);
            let (first, second) = differing_descriptions("discovery/two-descriptors");
            let (other, _) = differing_descriptions("discovery/other-service");

            let mut sut = RemoteOffers::new();
            sut.add(describe(&first).expect("description is encodable"), gateway);
            sut.add(
                describe(&second).expect("description is encodable"),
                gateway,
            );
            sut.add(describe(&other).expect("description is encodable"), gateway);

            assert_that!(sut.descriptors_of(&first.service_hash).count(), eq 2);
            assert_that!(sut.descriptors_of(&other.service_hash).count(), eq 1);
        }
    }

    mod remote_descriptions {
        use super::*;

        #[test]
        fn resolved_yields_the_local_description_only_when_resolved() {
            let (resolved, excluded) = differing_descriptions("discovery/resolutions");
            let resolved_descriptor = describe(&resolved).expect("description is encodable");
            let excluded_descriptor = describe(&excluded).expect("description is encodable");

            let mut sut = RemoteDescriptions::new();
            sut.set(
                resolved_descriptor.clone(),
                RemoteDescription::Resolved(resolved.clone()),
            );
            sut.set(excluded_descriptor.clone(), RemoteDescription::Excluded);

            assert_that!(sut.resolved(&resolved_descriptor), eq Some(&resolved));
            assert_that!(sut.resolved(&excluded_descriptor), eq None);
        }
    }
}
