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

use alloc::collections::BTreeMap;

use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_gateway_backend::types::identity::GatewayId;
use iceoryx2_gateway_backend::types::service_description::{
    EventSettings, PatternDescription, PortSettings, PublishSubscribeSettings, ServiceDescription,
};

/// The settings a service created with the local config receives when
/// none are specified.
#[derive(Debug, Clone)]
struct DefaultSettings {
    publish_subscribe: PublishSubscribeSettings,
    event: EventSettings,
}

impl DefaultSettings {
    fn from_config(config: &iceoryx2::config::Config) -> Self {
        Self {
            publish_subscribe: PublishSubscribeSettings::from_config(config),
            event: EventSettings::from_config(config),
        }
    }
}

impl Default for DefaultSettings {
    fn default() -> Self {
        Self::from_config(&iceoryx2::config::Config::default())
    }
}

/// Returns `description` in its explicit representation, `LocalDefaults`
/// replaced by the values of `defaults`.
fn normalize_settings(
    mut description: ServiceDescription,
    defaults: &DefaultSettings,
) -> ServiceDescription {
    match &mut description.pattern {
        PatternDescription::PublishSubscribe(pattern) => {
            if let PortSettings::LocalDefaults = pattern.settings {
                pattern.settings = PortSettings::Value(defaults.publish_subscribe.clone());
            }
        }
        PatternDescription::Event(pattern) => {
            if let PortSettings::LocalDefaults = pattern.settings {
                pattern.settings = PortSettings::Value(defaults.event.clone());
            }
        }
    }
    description
}

/// The local description of a service as seen on a specific update epoch.
#[derive(Debug)]
struct LocalDescription {
    description: ServiceDescription,
    last_seen: u64,
}

impl LocalDescription {
    // Get the description.
    fn get(&self) -> &ServiceDescription {
        &self.description
    }

    /// Whether the service was recorded as offered in `epoch`.
    fn seen_in(&self, epoch: u64) -> bool {
        self.last_seen == epoch
    }
}

/// The set of services offered locally.
#[derive(Debug, Default)]
pub(crate) struct LocalServices {
    offered: BTreeMap<ServiceHash, LocalDescription>,
    // The current update epoch.
    epoch: u64,
    defaults: DefaultSettings,
}

impl LocalServices {
    /// Records `description` as offered, stamping it with `epoch`. The caller                                                                                                                                                                                                                              ║
    /// passes its session epoch so all updates in a session share one stamp.
    fn insert(&mut self, description: ServiceDescription, epoch: u64) {
        let description = normalize_settings(description, &self.defaults);
        self.offered.insert(
            description.service_hash,
            LocalDescription {
                description,
                last_seen: epoch,
            },
        );
    }

    /// Removes `hash` if it was offered.
    fn remove(&mut self, hash: &ServiceHash) {
        self.offered.remove(hash);
    }

    /// Advances and returns the epoch, beginning a new update session.
    fn next_epoch(&mut self) -> u64 {
        self.epoch = self.epoch.wrapping_add(1);
        self.epoch
    }

    /// Whether `hash` is offered locally.
    pub(crate) fn contains(&self, hash: &ServiceHash) -> bool {
        self.offered.contains_key(hash)
    }

    // Get the description of a locally offered service.
    fn description(&self, hash: &ServiceHash) -> Option<&ServiceDescription> {
        self.offered.get(hash).map(LocalDescription::get)
    }

    // Iterate all locally offered services.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&ServiceHash, &ServiceDescription)> {
        self.offered
            .iter()
            .map(|(hash, service)| (hash, service.get()))
    }

    /// Returns a handle for applying delta updates. Advances the epoch,
    /// beginning a new update session.
    pub(crate) fn delta_update(&mut self) -> LocalDeltaUpdate<'_> {
        let epoch = self.next_epoch();
        LocalDeltaUpdate { local: self, epoch }
    }

    /// Forces the offered services to match an external target set.
    pub(crate) fn force_update(&mut self, target: impl Iterator<Item = ServiceDescription>) {
        let epoch = self.next_epoch();

        for description in target {
            self.insert(description, epoch);
        }

        self.offered.retain(|_, service| service.seen_in(epoch));
    }
}

/// Handle for applying incremental (delta) updates to the locally offered
/// services within a single epoch.
pub(crate) struct LocalDeltaUpdate<'a> {
    local: &'a mut LocalServices,
    epoch: u64,
}

impl LocalDeltaUpdate<'_> {
    /// Records `description` as offered, stamped with this handle's epoch.
    pub(crate) fn set_offered(&mut self, description: ServiceDescription) {
        self.local.insert(description, self.epoch);
    }

    /// Marks `hash` as no longer offered.
    pub(crate) fn set_not_offered(&mut self, hash: &ServiceHash) {
        self.local.remove(hash)
    }
}

/// The set of descriptions for the same service offered remotely.
#[derive(Debug, Default)]
pub(crate) struct RemoteDescriptions {
    by_gateway: BTreeMap<GatewayId, ServiceDescription>,
}

impl RemoteDescriptions {
    /// Records `description` as offered by `gateway`, replacing the
    /// description it previously offered.
    fn insert(&mut self, gateway: GatewayId, description: ServiceDescription) {
        self.by_gateway.insert(gateway, description);
    }

    /// Removes the description offered by `gateway`.
    fn remove(&mut self, gateway: &GatewayId) {
        self.by_gateway.remove(gateway);
    }

    /// Whether any gateway offers the service.
    fn offered(&self) -> bool {
        !self.by_gateway.is_empty()
    }

    /// Iterate the descriptions offered by each gateway.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&GatewayId, &ServiceDescription)> {
        self.by_gateway.iter()
    }

    /// Resolve the set of remote descriptions of the same service to a single
    /// description, if possible.
    ///
    /// Returns None if the descriptions cannot be resolved to a single
    /// description.
    fn resolve(&self) -> Option<&ServiceDescription> {
        let mut descriptions = self.by_gateway.values();
        let first = descriptions.next()?;
        descriptions
            .all(|description| description == first)
            .then_some(first)
    }

    /// Whether the descriptions resolve to exactly `description`.
    fn resolves_to(&self, description: &ServiceDescription) -> bool {
        self.resolve() == Some(description)
    }
}

/// The set of services offered remotely over the backend.
#[derive(Debug, Default)]
pub(crate) struct RemoteServices {
    services: BTreeMap<ServiceHash, RemoteDescriptions>,
    defaults: DefaultSettings,
}

impl RemoteServices {
    /// Records `gateway` as offering `description`.
    fn insert(&mut self, gateway: GatewayId, description: &ServiceDescription) {
        let description = normalize_settings(description.clone(), &self.defaults);
        self.services
            .entry(description.service_hash)
            .or_default()
            .insert(gateway, description);
    }

    /// Removes `gateway` from the gateways offering `hash`. The service is
    /// dropped once no gateway offers it anymore.
    fn remove(&mut self, gateway: &GatewayId, hash: &ServiceHash) {
        let Some(descriptions) = self.services.get_mut(hash) else {
            return;
        };
        descriptions.remove(gateway);
        if !descriptions.offered() {
            self.services.remove(hash);
        }
    }

    /// The descriptions of the service with `hash`, if any gateway offers
    /// it.
    fn descriptions(&self, hash: &ServiceHash) -> Option<&RemoteDescriptions> {
        self.services.get(hash)
    }

    /// Iterate all remote services and the offered descriptions.
    fn iter(&self) -> impl Iterator<Item = (&ServiceHash, &RemoteDescriptions)> {
        self.services.iter()
    }

    /// Returns a handle for applying delta updates.
    pub(crate) fn delta_update(&mut self) -> RemoteDeltaUpdate<'_> {
        RemoteDeltaUpdate { remote: self }
    }
}

/// Handle for applying incremental (delta) updates to the remotely offered
/// services.
pub(crate) struct RemoteDeltaUpdate<'a> {
    remote: &'a mut RemoteServices,
}

impl RemoteDeltaUpdate<'_> {
    /// Records `gateway` as offering `description`. A gateway's latest
    /// announcement replaces the description it previously offered.
    pub(crate) fn set_offered(&mut self, gateway: GatewayId, description: &ServiceDescription) {
        self.remote.insert(gateway, description);
    }

    /// Marks `hash` as no longer offered by `gateway`.
    pub(crate) fn set_not_offered(&mut self, gateway: &GatewayId, hash: &ServiceHash) {
        self.remote.remove(gateway, hash);
    }
}

/// A service whose sources disagree on its description.
#[derive(Debug)]
pub(crate) struct Conflict<'a> {
    pub(crate) hash: &'a ServiceHash,
    /// The description offered locally, if the local system offers the
    /// service.
    pub(crate) local: Option<&'a ServiceDescription>,
    /// The descriptions offered by remote gateways.
    pub(crate) remote: &'a RemoteDescriptions,
}

/// A point-in-time view of all offered services.
pub(crate) struct Snapshot<'a> {
    local: &'a LocalServices,
    remote: &'a RemoteServices,
}

impl<'a> Snapshot<'a> {
    /// Every offered hash, whichever side offers it.
    fn hashes(&self) -> impl Iterator<Item = &'a ServiceHash> {
        self.local.iter().map(|(hash, _)| hash).chain(
            self.remote
                .iter()
                .map(|(hash, _)| hash)
                .filter(|hash| !self.local.contains(hash)),
        )
    }

    /// Resolves the local description and remote descriptions of `hash` to
    /// a single description if possible.
    ///
    /// Returns None if the descriptions cannot be resolved.
    fn resolved_description(&self, hash: &ServiceHash) -> Option<&'a ServiceDescription> {
        let local = self.local.description(hash);
        let remote = self.remote.descriptions(hash);
        match (local, remote) {
            (Some(local), None) => Some(local),
            (Some(local), Some(remote)) => remote.resolves_to(local).then_some(local),
            (None, Some(remote)) => remote.resolve(),
            (None, None) => None,
        }
    }

    /// All services that resolve to one description.
    pub(crate) fn resolved(
        &self,
    ) -> impl Iterator<Item = (&'a ServiceHash, &'a ServiceDescription)> {
        self.hashes()
            .filter_map(|hash| Some((hash, self.resolved_description(hash)?)))
    }

    /// Whether `hash` is offered and resolves to one description.
    pub(crate) fn resolves(&self, hash: &ServiceHash) -> bool {
        self.resolved_description(hash).is_some()
    }

    /// The services whose sources do not agree on a service description.
    pub(crate) fn conflicts(&self) -> impl Iterator<Item = Conflict<'a>> {
        self.remote
            .iter()
            .filter_map(|(hash, remote_descriptions)| {
                if self.resolves(hash) {
                    return None;
                }
                Some(Conflict {
                    hash,
                    local: self.local.description(hash),
                    remote: remote_descriptions,
                })
            })
    }
}

/// The services the gateway has discovered, both locally and remotely.
#[derive(Debug, Default)]
pub(crate) struct DiscoveryState {
    local: LocalServices,
    remote: RemoteServices,
}

impl DiscoveryState {
    /// Creates a state whose entering descriptions have `LocalDefaults`
    /// resolved against `config`.
    pub(crate) fn new(config: &iceoryx2::config::Config) -> Self {
        let defaults = DefaultSettings::from_config(config);
        Self {
            local: LocalServices {
                defaults: defaults.clone(),
                ..LocalServices::default()
            },
            remote: RemoteServices {
                defaults,
                ..RemoteServices::default()
            },
        }
    }

    pub(crate) fn local(&self) -> &LocalServices {
        &self.local
    }

    pub(crate) fn local_mut(&mut self) -> &mut LocalServices {
        &mut self.local
    }

    pub(crate) fn remote_mut(&mut self) -> &mut RemoteServices {
        &mut self.remote
    }

    /// A view over all services offered by either side.
    pub(crate) fn snapshot(&self) -> Snapshot<'_> {
        Snapshot {
            local: &self.local,
            remote: &self.remote,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec::Vec;
    use iceoryx2::node::NodeBuilder;
    use iceoryx2::service::local;
    use iceoryx2::service::service_name::ServiceName;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_gateway_backend::types::identity::{BACKEND_ID_LENGTH, BackendId};
    use iceoryx2_gateway_backend::types::service_description::{
        EventDescription, EventSettings, PatternDescription, PortSettings,
    };

    fn gateway_id(discriminator: u8) -> GatewayId {
        let node = NodeBuilder::new()
            .create::<local::Service>()
            .expect("node creation succeeds");
        let backend = BackendId::new([discriminator; BACKEND_ID_LENGTH]);
        GatewayId::new(*node.id(), backend)
    }

    /// Two descriptions of the same service whose settings differ. They share
    /// a hash but do not compare equal.
    fn differing_descriptions(name: &str) -> (ServiceDescription, ServiceDescription) {
        let default_settings = ServiceDescription::new::<local::Service>(
            ServiceName::new(name).expect("valid service name"),
            PatternDescription::Event(EventDescription {
                settings: PortSettings::LocalDefaults,
            }),
        );
        let custom_settings = ServiceDescription::new::<local::Service>(
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

    mod local_services {
        use super::*;

        mod force_update {
            use super::*;

            #[test]
            fn tracks_additions_and_removals() {
                let first = ServiceDescription::new::<local::Service>(
                    ServiceName::new("state/local/first").expect("valid service name"),
                    PatternDescription::Event(EventDescription {
                        settings: PortSettings::LocalDefaults,
                    }),
                );
                let second = ServiceDescription::new::<local::Service>(
                    ServiceName::new("state/local/second").expect("valid service name"),
                    PatternDescription::Event(EventDescription {
                        settings: PortSettings::LocalDefaults,
                    }),
                );
                let third = ServiceDescription::new::<local::Service>(
                    ServiceName::new("state/local/third").expect("valid service name"),
                    PatternDescription::Event(EventDescription {
                        settings: PortSettings::LocalDefaults,
                    }),
                );

                let mut sut = LocalServices::default();
                sut.force_update([first.clone(), second.clone()].into_iter());
                assert_that!(sut.contains(&first.service_hash), eq true);
                assert_that!(sut.contains(&second.service_hash), eq true);
                assert_that!(sut.contains(&third.service_hash), eq false);

                sut.force_update([second.clone(), third.clone()].into_iter());
                assert_that!(sut.contains(&first.service_hash), eq false);
                assert_that!(sut.contains(&second.service_hash), eq true);
                assert_that!(sut.contains(&third.service_hash), eq true);
            }

            #[test]
            fn empty_target_removes_all_services() {
                let first = ServiceDescription::new::<local::Service>(
                    ServiceName::new("state/local/clear-first").expect("valid service name"),
                    PatternDescription::Event(EventDescription {
                        settings: PortSettings::LocalDefaults,
                    }),
                );
                let second = ServiceDescription::new::<local::Service>(
                    ServiceName::new("state/local/clear-second").expect("valid service name"),
                    PatternDescription::Event(EventDescription {
                        settings: PortSettings::LocalDefaults,
                    }),
                );

                let mut sut = LocalServices::default();
                sut.force_update([first.clone(), second.clone()].into_iter());
                sut.force_update(core::iter::empty::<ServiceDescription>());

                assert_that!(sut.contains(&first.service_hash), eq false);
                assert_that!(sut.contains(&second.service_hash), eq false);
            }

            #[test]
            fn latest_description_wins() {
                let (stored, updated) = differing_descriptions("state/local/force-latest");

                let mut sut = LocalServices::default();
                sut.force_update([stored].into_iter());
                sut.force_update([updated.clone()].into_iter());

                let (_, kept) = sut.iter().next().expect("service is offered");
                assert_that!(*kept, eq updated);
            }
        }

        mod delta_update {
            use super::*;

            #[test]
            fn tracks_additions_and_removals() {
                let first = ServiceDescription::new::<local::Service>(
                    ServiceName::new("state/local/delta-first").expect("valid service name"),
                    PatternDescription::Event(EventDescription {
                        settings: PortSettings::LocalDefaults,
                    }),
                );
                let second = ServiceDescription::new::<local::Service>(
                    ServiceName::new("state/local/delta-second").expect("valid service name"),
                    PatternDescription::Event(EventDescription {
                        settings: PortSettings::LocalDefaults,
                    }),
                );

                let mut sut = LocalServices::default();
                let mut update = sut.delta_update();
                update.set_offered(first.clone());
                update.set_offered(second.clone());
                assert_that!(sut.contains(&first.service_hash), eq true);
                assert_that!(sut.contains(&second.service_hash), eq true);

                sut.delta_update().set_not_offered(&first.service_hash);
                assert_that!(sut.contains(&first.service_hash), eq false);
                assert_that!(sut.contains(&second.service_hash), eq true);
            }

            #[test]
            fn latest_description_wins() {
                let (stored, updated) = differing_descriptions("state/local/delta-latest");

                let mut sut = LocalServices::default();
                sut.delta_update().set_offered(stored);
                sut.delta_update().set_offered(updated.clone());

                let (_, kept) = sut.iter().next().expect("service is offered");
                assert_that!(*kept, eq updated);
            }
        }
    }

    mod remote_services {
        use super::*;

        /// The description `hash` resolves to in `sut`, if it is offered
        /// and its offers resolve.
        fn resolved_description<'a>(
            sut: &'a RemoteServices,
            hash: &ServiceHash,
        ) -> Option<&'a ServiceDescription> {
            sut.descriptions(hash)?.resolve()
        }

        mod delta_update {
            use super::*;

            #[test]
            fn tracks_additions_and_removals() {
                let gateway = gateway_id(1);
                let first = ServiceDescription::new::<local::Service>(
                    ServiceName::new("state/remote/delta-first").expect("valid service name"),
                    PatternDescription::Event(EventDescription {
                        settings: PortSettings::LocalDefaults,
                    }),
                );
                let second = ServiceDescription::new::<local::Service>(
                    ServiceName::new("state/remote/delta-second").expect("valid service name"),
                    PatternDescription::Event(EventDescription {
                        settings: PortSettings::LocalDefaults,
                    }),
                );

                let mut sut = RemoteServices::default();
                let mut update = sut.delta_update();
                update.set_offered(gateway, &first);
                update.set_offered(gateway, &second);
                assert_that!(resolved_description(&sut, &first.service_hash), is_some);
                assert_that!(resolved_description(&sut, &second.service_hash), is_some);

                sut.delta_update()
                    .set_not_offered(&gateway, &first.service_hash);
                assert_that!(resolved_description(&sut, &first.service_hash), is_none);
                assert_that!(resolved_description(&sut, &second.service_hash), is_some);
            }

            #[test]
            fn stops_tracking_only_on_matching_hash_and_gateway_id() {
                let offering_gateway = gateway_id(1);
                let other_gateway = gateway_id(2);
                let description = ServiceDescription::new::<local::Service>(
                    ServiceName::new("state/remote/no-op").expect("valid service name"),
                    PatternDescription::Event(EventDescription {
                        settings: PortSettings::LocalDefaults,
                    }),
                );
                let hash = description.service_hash;
                let unknown_hash = ServiceDescription::new::<local::Service>(
                    ServiceName::new("state/remote/unknown").expect("valid service name"),
                    PatternDescription::Event(EventDescription {
                        settings: PortSettings::LocalDefaults,
                    }),
                )
                .service_hash;

                let mut sut = RemoteServices::default();
                let mut update = sut.delta_update();
                update.set_offered(offering_gateway, &description);
                update.set_not_offered(&other_gateway, &hash);
                update.set_not_offered(&offering_gateway, &unknown_hash);

                assert_that!(resolved_description(&sut, &hash), is_some);
            }

            #[test]
            fn tracked_services_remain_until_last_gateway_withdraws() {
                let gateway_a = gateway_id(1);
                let gateway_b = gateway_id(2);
                let description = ServiceDescription::new::<local::Service>(
                    ServiceName::new("state/remote/refcount").expect("valid service name"),
                    PatternDescription::Event(EventDescription {
                        settings: PortSettings::LocalDefaults,
                    }),
                );
                let hash = description.service_hash;

                let mut sut = RemoteServices::default();
                let mut update = sut.delta_update();
                update.set_offered(gateway_a, &description);
                update.set_offered(gateway_b, &description);
                assert_that!(resolved_description(&sut, &hash), is_some);

                sut.delta_update().set_not_offered(&gateway_a, &hash);
                assert_that!(resolved_description(&sut, &hash), is_some);

                sut.delta_update().set_not_offered(&gateway_b, &hash);
                assert_that!(resolved_description(&sut, &hash), is_none);
            }

            #[test]
            fn conflicting_descriptions_hide_the_service() {
                let gateway_a = gateway_id(1);
                let gateway_b = gateway_id(2);
                let (stored, conflicting) = differing_descriptions("state/remote/conflict");
                assert_that!(conflicting.service_hash, eq stored.service_hash);
                assert_that!(conflicting, ne stored);

                let mut sut = RemoteServices::default();
                let mut update = sut.delta_update();
                update.set_offered(gateway_a, &stored);
                update.set_offered(gateway_b, &conflicting);

                assert_that!(resolved_description(&sut, &stored.service_hash), is_none);
                assert_that!(
                    sut.iter()
                        .filter_map(|(_, descriptions)| descriptions.resolve())
                        .count(),
                    eq 0
                );
            }

            #[test]
            fn conflict_clears_when_a_disagreeing_gateway_withdraws() {
                let gateway_a = gateway_id(1);
                let gateway_b = gateway_id(2);
                let (stored, conflicting) = differing_descriptions("state/remote/conflict-clears");
                let hash = stored.service_hash;

                let mut sut = RemoteServices::default();
                let mut update = sut.delta_update();
                update.set_offered(gateway_a, &stored);
                update.set_offered(gateway_b, &conflicting);

                // Both gateways count as offerers. Withdrawal of one restores
                // the other's offer instead of dropping the service.
                sut.delta_update().set_not_offered(&gateway_a, &hash);
                let kept = resolved_description(&sut, &hash).expect("service is offered");
                assert_that!(*kept, eq conflicting);

                sut.delta_update().set_not_offered(&gateway_b, &hash);
                assert_that!(resolved_description(&sut, &hash), is_none);
            }

            #[test]
            fn reannouncement_replaces_the_gateways_offer() {
                let gateway = gateway_id(1);
                let (stored, updated) = differing_descriptions("state/remote/reannounce");

                let mut sut = RemoteServices::default();
                let mut update = sut.delta_update();
                update.set_offered(gateway, &stored);
                update.set_offered(gateway, &updated);

                let kept =
                    resolved_description(&sut, &stored.service_hash).expect("service is offered");
                assert_that!(*kept, eq updated);
            }

            #[test]
            fn repeated_offer_from_same_gateway_is_idempotent() {
                let gateway = gateway_id(1);
                let description = ServiceDescription::new::<local::Service>(
                    ServiceName::new("state/remote/idempotent").expect("valid service name"),
                    PatternDescription::Event(EventDescription {
                        settings: PortSettings::LocalDefaults,
                    }),
                );
                let hash = description.service_hash;

                let mut sut = RemoteServices::default();
                let mut update = sut.delta_update();
                update.set_offered(gateway, &description);
                update.set_offered(gateway, &description);
                update.set_not_offered(&gateway, &hash);

                assert_that!(resolved_description(&sut, &hash), is_none);
            }
        }
    }

    mod defaults {
        use super::*;

        #[test]
        fn local_defaults_resolve_to_the_config_values_on_entry() {
            let spelled_default = ServiceDescription::new::<local::Service>(
                ServiceName::new("state/defaults/resolve").expect("valid service name"),
                PatternDescription::Event(EventDescription {
                    settings: PortSettings::LocalDefaults,
                }),
            );

            let mut sut = LocalServices::default();
            sut.delta_update().set_offered(spelled_default);

            let (_, stored) = sut.iter().next().expect("service is offered");
            let PatternDescription::Event(description) = &stored.pattern else {
                panic!("expected an event pattern description");
            };
            let expected = PortSettings::Value(EventSettings::from_config(
                &iceoryx2::config::Config::default(),
            ));
            assert_that!(description.settings, eq expected);
        }

        #[test]
        fn differing_representations_of_default_settings_resolve() {
            let gateway = gateway_id(1);
            let spelled_default = ServiceDescription::new::<local::Service>(
                ServiceName::new("state/defaults/spellings").expect("valid service name"),
                PatternDescription::Event(EventDescription {
                    settings: PortSettings::LocalDefaults,
                }),
            );
            let explicit_default = ServiceDescription::new::<local::Service>(
                ServiceName::new("state/defaults/spellings").expect("valid service name"),
                PatternDescription::Event(EventDescription {
                    settings: PortSettings::Value(EventSettings::from_config(
                        &iceoryx2::config::Config::default(),
                    )),
                }),
            );

            let mut state = DiscoveryState::default();
            state
                .local_mut()
                .delta_update()
                .set_offered(spelled_default.clone());
            state
                .remote_mut()
                .delta_update()
                .set_offered(gateway, &explicit_default);

            assert_that!(state.snapshot().resolves(&spelled_default.service_hash), eq true);
            assert_that!(state.snapshot().conflicts().count(), eq 0);
        }
    }

    mod snapshot {
        use super::*;

        #[test]
        fn contains_services_offered_by_either_side() {
            let gateway = gateway_id(1);
            let local_offer = ServiceDescription::new::<local::Service>(
                ServiceName::new("state/snapshot/local-only").expect("valid service name"),
                PatternDescription::Event(EventDescription {
                    settings: PortSettings::LocalDefaults,
                }),
            );
            let remote_offer = ServiceDescription::new::<local::Service>(
                ServiceName::new("state/snapshot/remote-only").expect("valid service name"),
                PatternDescription::Event(EventDescription {
                    settings: PortSettings::LocalDefaults,
                }),
            );
            let unknown = ServiceDescription::new::<local::Service>(
                ServiceName::new("state/snapshot/unknown").expect("valid service name"),
                PatternDescription::Event(EventDescription {
                    settings: PortSettings::LocalDefaults,
                }),
            );

            let mut state = DiscoveryState::default();
            state
                .local_mut()
                .delta_update()
                .set_offered(local_offer.clone());
            state
                .remote_mut()
                .delta_update()
                .set_offered(gateway, &remote_offer);

            let snapshot = state.snapshot();
            assert_that!(snapshot.resolves(&local_offer.service_hash), eq true);
            assert_that!(snapshot.resolves(&remote_offer.service_hash), eq true);
            assert_that!(snapshot.resolves(&unknown.service_hash), eq false);
        }

        #[test]
        fn consolidates_services_offered_identically_by_both_sides() {
            let gateway = gateway_id(1);
            let shared = ServiceDescription::new::<local::Service>(
                ServiceName::new("state/snapshot/shared").expect("valid service name"),
                PatternDescription::Event(EventDescription {
                    settings: PortSettings::LocalDefaults,
                }),
            );

            let mut state = DiscoveryState::default();
            state.local_mut().delta_update().set_offered(shared.clone());
            state
                .remote_mut()
                .delta_update()
                .set_offered(gateway, &shared);

            let snapshot = state.snapshot();
            assert_that!(snapshot.resolves(&shared.service_hash), eq true);
            let offered: Vec<ServiceHash> = snapshot.resolved().map(|(hash, _)| *hash).collect();
            assert_that!(offered, len 1);
        }

        #[test]
        fn hides_services_with_disagreeing_local_and_remote_descriptions() {
            let gateway = gateway_id(1);
            let (local_offer, remote_offer) = differing_descriptions("state/snapshot/disagree");
            let remote_only = ServiceDescription::new::<local::Service>(
                ServiceName::new("state/snapshot/remote-only").expect("valid service name"),
                PatternDescription::Event(EventDescription {
                    settings: PortSettings::LocalDefaults,
                }),
            );

            let mut state = DiscoveryState::default();
            state
                .local_mut()
                .delta_update()
                .set_offered(local_offer.clone());
            let mut remote_update = state.remote_mut().delta_update();
            remote_update.set_offered(gateway, &remote_offer);
            remote_update.set_offered(gateway, &remote_only);

            let snapshot = state.snapshot();
            assert_that!(snapshot.resolves(&local_offer.service_hash), eq false);
            let offered: Vec<ServiceHash> = snapshot.resolved().map(|(hash, _)| *hash).collect();
            assert_that!(offered, len 1);
            assert_that!(offered.contains(&remote_only.service_hash), eq true);
        }

        #[test]
        fn hides_services_with_disagreeing_remote_descriptions() {
            let gateway_a = gateway_id(1);
            let gateway_b = gateway_id(2);
            let (first, second) = differing_descriptions("state/snapshot/remote-disagree");

            let mut state = DiscoveryState::default();
            let mut remote_update = state.remote_mut().delta_update();
            remote_update.set_offered(gateway_a, &first);
            remote_update.set_offered(gateway_b, &second);

            let snapshot = state.snapshot();
            assert_that!(snapshot.resolves(&first.service_hash), eq false);
            assert_that!(snapshot.resolved().count(), eq 0);
        }
    }

    mod conflicts {
        use super::*;

        #[test]
        fn lists_services_with_disagreeing_sources() {
            let gateway_a = gateway_id(1);
            let gateway_b = gateway_id(2);
            let (local_offer, remote_offer) = differing_descriptions("state/conflicts/with-local");
            let (remote_first, remote_second) =
                differing_descriptions("state/conflicts/remote-only");

            let mut state = DiscoveryState::default();
            state
                .local_mut()
                .delta_update()
                .set_offered(local_offer.clone());
            let mut remote_update = state.remote_mut().delta_update();
            remote_update.set_offered(gateway_a, &remote_offer);
            remote_update.set_offered(gateway_a, &remote_first);
            remote_update.set_offered(gateway_b, &remote_second);

            let conflicts: Vec<_> = state.snapshot().conflicts().collect();
            assert_that!(conflicts, len 2);

            let with_local = conflicts
                .iter()
                .find(|conflict| *conflict.hash == local_offer.service_hash)
                .expect("conflict with the local side is listed");
            assert_that!(with_local.local, is_some);
            assert_that!(with_local.remote.iter().count(), eq 1);

            let remote_only = conflicts
                .iter()
                .find(|conflict| *conflict.hash == remote_first.service_hash)
                .expect("conflict among remotes is listed");
            assert_that!(remote_only.local, is_none);
            assert_that!(remote_only.remote.iter().count(), eq 2);
        }

        #[test]
        fn sources_that_resolve_produce_no_conflicts() {
            let gateway = gateway_id(1);
            let shared = ServiceDescription::new::<local::Service>(
                ServiceName::new("state/conflicts/resolve").expect("valid service name"),
                PatternDescription::Event(EventDescription {
                    settings: PortSettings::LocalDefaults,
                }),
            );

            let mut state = DiscoveryState::default();
            state.local_mut().delta_update().set_offered(shared.clone());
            state
                .remote_mut()
                .delta_update()
                .set_offered(gateway, &shared);

            assert_that!(state.snapshot().conflicts().count(), eq 0);
        }
    }
}
