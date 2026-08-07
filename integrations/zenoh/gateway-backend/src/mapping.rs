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

use std::collections::BTreeSet;

use iceoryx2::service::Service;
use iceoryx2_gateway_backend::traits::Mapping;
use iceoryx2_gateway_backend::types::service_description::ServiceDescription;

/// Identity mapping scoped to an optional allow list of exact service names.
#[derive(Debug, Default)]
pub struct AllowListMapping {
    allowlist: Option<BTreeSet<String>>,
}

impl AllowListMapping {
    /// Creates a mapping admitting only `services`. An empty iterator leaves
    /// the identity mapping unrestricted.
    pub fn new<I, N>(services: I) -> Self
    where
        I: IntoIterator<Item = N>,
        N: Into<String>,
    {
        let services: BTreeSet<String> = services.into_iter().map(Into::into).collect();
        Self {
            allowlist: (!services.is_empty()).then_some(services),
        }
    }

    fn admits(&self, description: &ServiceDescription) -> bool {
        self.allowlist
            .as_ref()
            .is_none_or(|services| services.contains(description.name.as_str()))
    }
}

impl Mapping for AllowListMapping {
    type EndpointDescription = ServiceDescription;

    fn remote(&self, description: &ServiceDescription) -> Option<ServiceDescription> {
        self.admits(description).then(|| description.clone())
    }

    fn local<S: Service>(&self, remote: &ServiceDescription) -> Option<ServiceDescription> {
        self.admits(remote).then(|| remote.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iceoryx2::service::ipc;
    use iceoryx2_gateway_backend::types::service_description::{
        EventDescription, PatternDescription, PortSettings,
    };

    fn service_description(name: &str) -> ServiceDescription {
        ServiceDescription::new::<ipc::Service>(
            name.try_into().expect("valid service name"),
            PatternDescription::Event(EventDescription {
                settings: PortSettings::LocalDefaults,
            }),
        )
    }

    #[test]
    fn omitted_allow_list_admits_every_service_in_both_directions() {
        let sut = AllowListMapping::default();
        let service = service_description("service");

        assert!(sut.remote(&service).is_some());
        assert!(sut.local::<ipc::Service>(&service).is_some());
    }

    #[test]
    fn allow_list_is_applied_in_both_directions() {
        let sut = AllowListMapping::new(["allowed"]);
        let allowed = service_description("allowed");
        let blocked = service_description("blocked");

        assert!(sut.remote(&allowed).is_some());
        assert!(sut.local::<ipc::Service>(&allowed).is_some());
        assert!(sut.remote(&blocked).is_none());
        assert!(sut.local::<ipc::Service>(&blocked).is_none());
    }
}
