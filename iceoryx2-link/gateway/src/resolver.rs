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
use iceoryx2::service::Service;
use iceoryx2::service::messaging_pattern::MessagingPattern as Pattern;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_link_backend::resolver::{self, Resolution};
use iceoryx2_link_backend::service_description::{
    Identified, PatternSettings, ServiceDescription, ServiceTypes,
};

use iceoryx2_link_adapter::Mapping;
use iceoryx2_link_adapter::{EndpointDescription, EndpointTypes};
use iceoryx2_link_adapter::{SampleShape, Translator};

/// Reasons why a service or remote endpoint is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal<E> {
    /// The mapping refuses it, for the reason it gives.
    Mapping(E),
    /// The mapping leads from its counterpart back to another name.
    LeadsElsewhere,
    /// The translator has no counterpart for its types.
    Untranslatable,
    /// The mapping's settings and the translator's types are of different
    /// patterns.
    PatternMismatch,
    /// The gateway does not bridge its pattern.
    UnsupportedPattern,
    /// More than one endpoint maps to the service.
    Ambiguous,
}

impl<E: core::fmt::Display> core::fmt::Display for Refusal<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Refusal::Mapping(reason) => write!(f, "is refused by the mapping, {reason}"),
            Refusal::LeadsElsewhere => write!(f, "is not where the mapping leads back to"),
            Refusal::Untranslatable => write!(f, "cannot be translated"),
            Refusal::PatternMismatch => write!(f, "has settings and types of different patterns"),
            Refusal::UnsupportedPattern => write!(f, "has a pattern the gateway does not bridge"),
            Refusal::Ambiguous => write!(f, "is mapped from more than one endpoint"),
        }
    }
}

/// Resolves services against the middleware's endpoints through a
/// mapping and a translator. A local service is exported to the endpoint
/// it maps to, the one endpoint mapping to a service nobody offers is
/// imported as the mapped service.
pub struct Resolver<M, T> {
    pub(crate) mapping: M,
    pub(crate) translator: T,
}

impl<M: Mapping, T: Translator<SampleShape>> Resolver<M, T> {
    /// The middleware's types of an endpoint carrying a service of `types`.
    fn remote_types(
        &self,
        types: &ServiceTypes,
    ) -> Result<EndpointTypes<T::RemoteTypes>, T::Error> {
        Ok(match types {
            ServiceTypes::PublishSubscribe(types) => {
                EndpointTypes::PublishSubscribe(self.translator.remote(types)?)
            }
            ServiceTypes::Event => EndpointTypes::Event,
        })
    }

    /// The local types of a service mirroring an endpoint of `types`.
    fn local_types(&self, types: &EndpointTypes<T::RemoteTypes>) -> Result<ServiceTypes, T::Error> {
        Ok(match types {
            EndpointTypes::PublishSubscribe(types) => {
                ServiceTypes::PublishSubscribe(self.translator.local(types)?)
            }
            EndpointTypes::Event => ServiceTypes::Event,
        })
    }

    /// The endpoint a local service is exported to.
    #[allow(clippy::type_complexity)] // the resolution names the gateway's endpoint description in full
    fn export(
        &self,
        local: &ServiceDescription,
    ) -> Resolution<
        EndpointDescription<M::EndpointSettings, EndpointTypes<T::RemoteTypes>>,
        Refusal<M::Error>,
    > {
        if !bridged(local.types()) {
            return Resolution::Refused(Refusal::UnsupportedPattern);
        }
        let settings = match self.mapping.remote(local.settings()) {
            Ok(Some(settings)) => settings,
            Ok(None) => return Resolution::OutOfScope,
            Err(error) => return Resolution::Refused(Refusal::Mapping(error)),
        };
        if let Ok(Some(defined)) = self.mapping.local(&settings)
            && defined.id() != local.settings().id()
        {
            return Resolution::Refused(Refusal::LeadsElsewhere);
        }
        let Ok(types) = self.remote_types(local.types()) else {
            return Resolution::Refused(Refusal::Untranslatable);
        };
        Resolution::Exported(EndpointDescription { settings, types })
    }

    /// The local service an endpoint is imported as, hashed with the name
    /// hasher of `S`.
    #[allow(clippy::type_complexity)] // the resolution names the gateway's endpoint description in full
    fn import<S: Service>(
        &self,
        remote: &EndpointDescription<M::EndpointSettings, EndpointTypes<T::RemoteTypes>>,
    ) -> Resolution<
        EndpointDescription<M::EndpointSettings, EndpointTypes<T::RemoteTypes>>,
        Refusal<M::Error>,
    > {
        let settings = match self.mapping.local(&remote.settings) {
            Ok(Some(settings)) => settings,
            Ok(None) => return Resolution::OutOfScope,
            Err(error) => return Resolution::Refused(Refusal::Mapping(error)),
        };
        if let Ok(Some(defined)) = self.mapping.remote(&settings)
            && defined.id() != remote.settings.id()
        {
            return Resolution::Refused(Refusal::LeadsElsewhere);
        }
        let Ok(types) = self.local_types(&remote.types) else {
            return Resolution::Refused(Refusal::Untranslatable);
        };
        if !bridged(&types) {
            return Resolution::Refused(Refusal::UnsupportedPattern);
        }
        let name = settings.id();
        let mirror = match (settings.pattern, types) {
            (
                PatternSettings::PublishSubscribe(settings),
                ServiceTypes::PublishSubscribe(types),
            ) => ServiceDescription::compose_publish_subscribe::<S>(name, settings, types),
            (PatternSettings::Event(settings), ServiceTypes::Event) => {
                ServiceDescription::compose_event::<S>(name, settings)
            }
            _ => return Resolution::Refused(Refusal::PatternMismatch),
        };
        Resolution::Imported(mirror, remote.clone())
    }
}

impl<S: Service, M: Mapping, T: Translator<SampleShape>> resolver::Resolver<S> for Resolver<M, T> {
    type RemoteId = <M::EndpointSettings as Identified>::Id;
    type RemoteDescription =
        EndpointDescription<M::EndpointSettings, EndpointTypes<T::RemoteTypes>>;
    type Refusal = Refusal<M::Error>;

    fn service_hash(
        &self,
        remote: &Self::RemoteDescription,
    ) -> Result<Option<ServiceHash>, Refusal<M::Error>> {
        let settings = match self.mapping.local(&remote.settings) {
            Ok(Some(settings)) => settings,
            Ok(None) => return Ok(None),
            Err(error) => return Err(Refusal::Mapping(error)),
        };
        let pattern = match settings.pattern {
            PatternSettings::PublishSubscribe(_) => Pattern::PublishSubscribe,
            PatternSettings::Event(_) => Pattern::Event,
        };
        Ok(Some(ServiceHash::new::<S::ServiceNameHasher>(
            &settings.id(),
            pattern,
        )))
    }

    fn resolve<'a>(
        &self,
        local: Option<&ServiceDescription>,
        mut remotes: impl Iterator<Item = &'a Self::RemoteDescription> + Clone,
    ) -> Resolution<Self::RemoteDescription, Refusal<M::Error>> {
        // A local service takes precedence over the endpoints mapping to
        // its name, it is exported whatever they list.
        if let Some(local) = local {
            return self.export(local);
        }
        let Some(remote) = remotes.next() else {
            return Resolution::OutOfScope;
        };
        if remotes.next().is_some() {
            return Resolution::Refused(Refusal::Ambiguous);
        }
        self.import::<S>(remote)
    }
}

/// Whether the gateway bridges services of these types' pattern.
fn bridged(types: &ServiceTypes) -> bool {
    matches!(types, ServiceTypes::PublishSubscribe(_))
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::service::local;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_link_backend::service_description::{SampleTypes, ServiceSettings};

    use crate::testing::{
        DefiningMapping, IdentityMapping, PartialMapping, RedirectingMapping, Refused,
        RefusingTranslator, StubEndpointDescription, description, description_of, endpoint,
        endpoint_of, event, settings,
    };
    use iceoryx2_link_adapter::Passthrough;

    const HISTORY_SIZE: usize = 1;
    const OTHER_HISTORY_SIZE: usize = 2;
    const REFUSED_PAYLOAD: &str = "refused";

    fn service_hash<M: Mapping<EndpointSettings = crate::testing::StubEndpointSettings>>(
        sut: &Resolver<M, Passthrough>,
        remote: &StubEndpointDescription,
    ) -> Result<Option<ServiceHash>, Refusal<M::Error>> {
        <Resolver<M, Passthrough> as resolver::Resolver<local::Service>>::service_hash(sut, remote)
    }

    fn resolve<M, T>(
        sut: &Resolver<M, T>,
        local: Option<&ServiceDescription>,
        remotes: &[StubEndpointDescription],
    ) -> Resolution<StubEndpointDescription, Refusal<M::Error>>
    where
        M: Mapping<EndpointSettings = crate::testing::StubEndpointSettings>,
        T: Translator<SampleShape, RemoteTypes = SampleTypes>,
    {
        <Resolver<M, T> as resolver::Resolver<local::Service>>::resolve(sut, local, remotes.iter())
    }

    #[test]
    fn an_endpoint_belongs_to_the_service_it_maps_to() {
        const SERVICE: &str = "resolver/hash";

        let sut = Resolver {
            mapping: IdentityMapping,
            translator: Passthrough,
        };
        let remote = endpoint(SERVICE, HISTORY_SIZE);
        let hash = service_hash(&sut, &remote);

        assert_that!(hash, eq Ok(Some(description(SERVICE, HISTORY_SIZE).hash())));
    }

    #[test]
    fn an_endpoint_the_mapping_does_not_cover_belongs_to_no_service() {
        const SERVICE: &str = "uncovered/hash";

        let sut = Resolver {
            mapping: PartialMapping,
            translator: Passthrough,
        };
        let remote = endpoint(SERVICE, HISTORY_SIZE);
        let hash = service_hash(&sut, &remote);

        assert_that!(hash, eq Ok(None));
    }

    #[test]
    fn an_endpoint_the_mapping_cannot_map_is_refused_a_service() {
        const SERVICE: &str = "unmappable/hash";

        let sut = Resolver {
            mapping: PartialMapping,
            translator: Passthrough,
        };
        let remote = endpoint(SERVICE, HISTORY_SIZE);
        let hash = service_hash(&sut, &remote);

        assert_that!(hash, eq Err(Refusal::Mapping(Refused)));
    }

    #[test]
    fn a_service_the_mapping_does_not_define_is_exported_to_its_endpoint() {
        const SERVICE: &str = "resolver/export";

        let sut = Resolver {
            mapping: IdentityMapping,
            translator: Passthrough,
        };
        let local = description(SERVICE, HISTORY_SIZE);
        let resolution = resolve(&sut, Some(&local), &[]);

        assert_that!(resolution, eq Resolution::Exported(endpoint(SERVICE, HISTORY_SIZE)));
    }

    #[test]
    fn a_service_is_exported_whatever_its_endpoint_lists() {
        const SERVICE: &str = "resolver/export/listed";

        let sut = Resolver {
            mapping: IdentityMapping,
            translator: Passthrough,
        };
        let listed = [endpoint(SERVICE, OTHER_HISTORY_SIZE)];
        let local = description(SERVICE, HISTORY_SIZE);
        let resolution = resolve(&sut, Some(&local), &listed);

        assert_that!(resolution, eq Resolution::Exported(endpoint(SERVICE, HISTORY_SIZE)));
    }

    #[test]
    fn a_service_matching_what_the_mapping_defines_is_exported() {
        const SERVICE: &str = "resolver/export/defined";

        let local = description(SERVICE, HISTORY_SIZE);
        let defined = ServiceSettings::new(
            local.name(),
            PatternSettings::PublishSubscribe(settings(HISTORY_SIZE)),
        );
        let sut = Resolver {
            mapping: DefiningMapping(alloc::vec![defined]),
            translator: Passthrough,
        };
        let resolution = resolve(&sut, Some(&local), &[]);

        assert_that!(resolution, eq Resolution::Exported(endpoint(SERVICE, HISTORY_SIZE)));
    }

    #[test]
    fn a_service_differing_from_what_the_mapping_defines_is_refused() {
        const SERVICE: &str = "resolver/export/differing";

        let local = description(SERVICE, HISTORY_SIZE);
        let defined = ServiceSettings::new(
            local.name(),
            PatternSettings::PublishSubscribe(settings(OTHER_HISTORY_SIZE)),
        );
        let sut = Resolver {
            mapping: DefiningMapping(alloc::vec![defined]),
            translator: Passthrough,
        };
        let resolution = resolve(&sut, Some(&local), &[]);

        assert_that!(resolution, eq Resolution::Refused(Refusal::Mapping(Refused)));
    }

    #[test]
    fn a_service_whose_endpoint_maps_to_another_service_is_refused() {
        const SERVICE: &str = "redirected/export";

        let sut = Resolver {
            mapping: RedirectingMapping,
            translator: Passthrough,
        };
        let local = description(SERVICE, HISTORY_SIZE);
        let resolution = resolve(&sut, Some(&local), &[]);

        assert_that!(resolution, eq Resolution::Refused(Refusal::LeadsElsewhere));
    }

    #[test]
    fn a_service_the_mapping_does_not_cover_is_out_of_scope() {
        const SERVICE: &str = "uncovered/export";

        let sut = Resolver {
            mapping: PartialMapping,
            translator: Passthrough,
        };
        let local = description(SERVICE, HISTORY_SIZE);
        let resolution = resolve(&sut, Some(&local), &[]);

        assert_that!(resolution, eq Resolution::OutOfScope);
    }

    #[test]
    fn a_service_the_mapping_cannot_map_is_refused() {
        const SERVICE: &str = "unmappable/export";

        let sut = Resolver {
            mapping: PartialMapping,
            translator: Passthrough,
        };
        let local = description(SERVICE, HISTORY_SIZE);
        let resolution = resolve(&sut, Some(&local), &[]);

        assert_that!(resolution, eq Resolution::Refused(Refusal::Mapping(Refused)));
    }

    #[test]
    fn a_service_the_translator_cannot_express_is_refused() {
        const SERVICE: &str = "resolver/export/untranslatable";

        let sut = Resolver {
            mapping: IdentityMapping,
            translator: RefusingTranslator,
        };
        let local = description_of(SERVICE, HISTORY_SIZE, REFUSED_PAYLOAD);
        let resolution = resolve(&sut, Some(&local), &[]);

        assert_that!(resolution, eq Resolution::Refused(Refusal::Untranslatable));
    }

    #[test]
    fn a_local_event_service_is_refused() {
        const SERVICE: &str = "resolver/export/event";

        let sut = Resolver {
            mapping: IdentityMapping,
            translator: Passthrough,
        };
        let local = event(SERVICE);
        let resolution = resolve(&sut, Some(&local), &[]);

        assert_that!(resolution, eq Resolution::Refused(Refusal::UnsupportedPattern));
    }

    #[test]
    fn an_endpoint_is_imported_as_the_composed_service() {
        const SERVICE: &str = "resolver/import";

        let sut = Resolver {
            mapping: IdentityMapping,
            translator: Passthrough,
        };
        let remote = endpoint(SERVICE, HISTORY_SIZE);
        let resolution = resolve(&sut, None, core::slice::from_ref(&remote));

        let mirror = description(SERVICE, HISTORY_SIZE);
        assert_that!(resolution, eq Resolution::Imported(mirror, remote));
    }

    #[test]
    fn a_service_more_than_one_endpoint_maps_to_is_refused() {
        const SERVICE: &str = "resolver/import/ambiguous";

        let sut = Resolver {
            mapping: IdentityMapping,
            translator: Passthrough,
        };
        let listed = [
            endpoint(SERVICE, HISTORY_SIZE),
            endpoint(SERVICE, OTHER_HISTORY_SIZE),
        ];
        let resolution = resolve(&sut, None, &listed);

        assert_that!(resolution, eq Resolution::Refused(Refusal::Ambiguous));
    }

    #[test]
    fn an_endpoint_the_mapping_does_not_cover_is_out_of_scope() {
        const SERVICE: &str = "uncovered/import";

        let sut = Resolver {
            mapping: PartialMapping,
            translator: Passthrough,
        };
        let remote = endpoint(SERVICE, HISTORY_SIZE);
        let resolution = resolve(&sut, None, &[remote]);

        assert_that!(resolution, eq Resolution::OutOfScope);
    }

    #[test]
    fn an_endpoint_the_mapping_cannot_map_is_refused() {
        const SERVICE: &str = "unmappable/import";

        let sut = Resolver {
            mapping: PartialMapping,
            translator: Passthrough,
        };
        let remote = endpoint(SERVICE, HISTORY_SIZE);
        let resolution = resolve(&sut, None, &[remote]);

        assert_that!(resolution, eq Resolution::Refused(Refusal::Mapping(Refused)));
    }

    #[test]
    fn an_endpoint_differing_from_what_the_mapping_defines_is_refused() {
        const SERVICE: &str = "resolver/import/differing";

        let remote = endpoint(SERVICE, HISTORY_SIZE);
        let defined = ServiceSettings::new(
            remote.settings.name,
            PatternSettings::PublishSubscribe(settings(OTHER_HISTORY_SIZE)),
        );
        let sut = Resolver {
            mapping: DefiningMapping(alloc::vec![defined]),
            translator: Passthrough,
        };
        let resolution = resolve(&sut, None, &[remote]);

        assert_that!(resolution, eq Resolution::Refused(Refusal::Mapping(Refused)));
    }

    #[test]
    fn an_endpoint_whose_service_maps_to_another_endpoint_is_refused() {
        const SERVICE: &str = "redirected/import";

        let sut = Resolver {
            mapping: RedirectingMapping,
            translator: Passthrough,
        };
        let remote = endpoint(SERVICE, HISTORY_SIZE);
        let resolution = resolve(&sut, None, &[remote]);

        assert_that!(resolution, eq Resolution::Refused(Refusal::LeadsElsewhere));
    }

    #[test]
    fn an_endpoint_the_translator_cannot_express_is_refused() {
        const SERVICE: &str = "resolver/import/untranslatable";

        let sut = Resolver {
            mapping: IdentityMapping,
            translator: RefusingTranslator,
        };
        let remote = endpoint_of(SERVICE, HISTORY_SIZE, REFUSED_PAYLOAD);
        let resolution = resolve(&sut, None, &[remote]);

        assert_that!(resolution, eq Resolution::Refused(Refusal::Untranslatable));
    }

    #[test]
    fn an_event_endpoint_is_refused() {
        const SERVICE: &str = "resolver/import/event";

        let sut = Resolver {
            mapping: IdentityMapping,
            translator: Passthrough,
        };
        let mut remote = endpoint(SERVICE, HISTORY_SIZE);
        remote.types = EndpointTypes::Event;

        let resolution = resolve(&sut, None, &[remote]);

        assert_that!(resolution, eq Resolution::Refused(Refusal::UnsupportedPattern));
    }

    #[test]
    fn a_service_with_neither_side_is_out_of_scope() {
        let sut = Resolver {
            mapping: IdentityMapping,
            translator: Passthrough,
        };
        let resolution = resolve(&sut, None, &[]);

        assert_that!(resolution, eq Resolution::OutOfScope);
    }
}
