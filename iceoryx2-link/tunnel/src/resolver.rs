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
use iceoryx2::config::Config;
use iceoryx2::service::Service;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_link_backend::resolver::{self, Resolution};
use iceoryx2_link_backend::service_description::{ServiceDescription, ServiceDescriptor};
use iceoryx2_log::{origin, warn};

use iceoryx2_link_carrier::PeerId;

/// Why a service the peers offer is not bridged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The peers offer the service under differing types.
    Disputed,
}

impl core::fmt::Display for Refusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Refusal::Disputed => write!(f, "has differing types among its peers"),
        }
    }
}

/// Resolves services against the peers' offers. A local service is
/// exported, a service the peers unanimously offer is imported with the
/// local defaults.
pub struct Resolver {
    /// The configuration of the local system, whose defaults mirrors are
    /// created with.
    config: Config,
}

impl Resolver {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl<S: Service> resolver::Resolver<S> for Resolver {
    type RemoteId = PeerId;
    type RemoteDescription = ServiceDescriptor;
    type Refusal = Refusal;

    fn service_hash(&self, remote: &ServiceDescriptor) -> Result<Option<ServiceHash>, Refusal> {
        Ok(Some(remote.hash))
    }

    fn resolve<'a>(
        &self,
        local: Option<&ServiceDescription>,
        mut remotes: impl Iterator<Item = &'a ServiceDescriptor> + Clone,
    ) -> Resolution<ServiceDescriptor, Refusal> {
        let origin = origin!("Resolver::resolve");

        // A peer offering the local service under other types never
        // receives its samples, the channel is keyed by the types.
        if let Some(local) = local {
            if remotes.any(|offered| offered.types != *local.types()) {
                warn!(
                    from origin,
                    "Service {} has other types on a peer, samples cannot be exchanged with it",
                    local.name()
                );
            }
            return Resolution::Exported(ServiceDescriptor::from(local));
        }
        let Some(first) = remotes.next() else {
            return Resolution::OutOfScope;
        };
        if remotes.any(|offered| offered.types != first.types) {
            return Resolution::Refused(Refusal::Disputed);
        }
        let mirror = ServiceDescription::with_default_settings::<S>(
            first.name,
            first.types.clone(),
            &self.config,
        );
        Resolution::Imported(mirror, first.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::service::local;
    use iceoryx2_bb_testing::assert_that;

    use crate::testing::{description, descriptor};

    const PAYLOAD: &str = "u64";
    const OTHER_PAYLOAD: &str = "u32";

    fn resolve(
        sut: &Resolver,
        local: Option<&ServiceDescription>,
        offers: &[ServiceDescriptor],
    ) -> Resolution<ServiceDescriptor, Refusal> {
        <Resolver as resolver::Resolver<local::Service>>::resolve(sut, local, offers.iter())
    }

    #[test]
    fn a_descriptor_belongs_to_the_service_whose_hash_it_carries() {
        const SERVICE: &str = "resolver/hash";

        let sut = Resolver::new(&Config::default());
        let offered = descriptor(SERVICE, PAYLOAD);
        let hash = <Resolver as resolver::Resolver<local::Service>>::service_hash(&sut, &offered);

        assert_that!(hash, eq Ok(Some(offered.hash)));
    }

    #[test]
    fn a_local_service_nobody_offers_is_exported() {
        const SERVICE: &str = "resolver/export";

        let sut = Resolver::new(&Config::default());
        let local = description(SERVICE, PAYLOAD);
        let resolution = resolve(&sut, Some(&local), &[]);

        assert_that!(resolution, eq Resolution::Exported(ServiceDescriptor::from(&local)));
    }

    #[test]
    fn a_local_service_a_peer_offers_under_the_same_types_is_exported() {
        const SERVICE: &str = "resolver/export/agreed";

        let sut = Resolver::new(&Config::default());
        let local = description(SERVICE, PAYLOAD);
        let offers = [descriptor(SERVICE, PAYLOAD)];
        let resolution = resolve(&sut, Some(&local), &offers);

        assert_that!(resolution, eq Resolution::Exported(ServiceDescriptor::from(&local)));
    }

    #[test]
    fn a_local_service_a_peer_offers_under_other_types_is_exported_with_its_own_types() {
        const SERVICE: &str = "resolver/export/contested";

        let sut = Resolver::new(&Config::default());
        let local = description(SERVICE, PAYLOAD);
        let offers = [
            descriptor(SERVICE, PAYLOAD),
            descriptor(SERVICE, OTHER_PAYLOAD),
        ];
        let resolution = resolve(&sut, Some(&local), &offers);

        assert_that!(resolution, eq Resolution::Exported(ServiceDescriptor::from(&local)));
    }

    #[test]
    fn a_service_the_peers_agree_on_is_imported_with_the_local_defaults() {
        const SERVICE: &str = "resolver/import";

        let sut = Resolver::new(&Config::default());
        let offers = [descriptor(SERVICE, PAYLOAD), descriptor(SERVICE, PAYLOAD)];
        let resolution = resolve(&sut, None, &offers);

        let mirror = description(SERVICE, PAYLOAD);
        assert_that!(resolution, eq Resolution::Imported(mirror, descriptor(SERVICE, PAYLOAD)));
    }

    #[test]
    fn a_service_the_peers_disagree_on_is_disputed() {
        const SERVICE: &str = "resolver/import/disputed";

        let sut = Resolver::new(&Config::default());
        let offers = [
            descriptor(SERVICE, PAYLOAD),
            descriptor(SERVICE, OTHER_PAYLOAD),
        ];
        let resolution = resolve(&sut, None, &offers);

        assert_that!(resolution, eq Resolution::Refused(Refusal::Disputed));
    }

    #[test]
    fn a_service_with_neither_side_is_out_of_scope() {
        let sut = Resolver::new(&Config::default());
        let resolution = resolve(&sut, None, &[]);

        assert_that!(resolution, eq Resolution::OutOfScope);
    }
}
