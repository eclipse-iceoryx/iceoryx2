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
use iceoryx2_gateway_backend::traits::Mapping;
pub use iceoryx2_gateway_backend::types::allow_list::AllowList;
use iceoryx2_gateway_backend::types::service_description::ServiceDescription;

/// Identity mapping scoped to an allow list of service names.
#[derive(Debug, Default)]
pub struct AllowListMapping {
    allowlist: AllowList,
}

impl AllowListMapping {
    /// Creates a mapping scoped to `allowlist`.
    pub fn new(allowlist: AllowList) -> Self {
        Self { allowlist }
    }

    fn admits(&self, description: &ServiceDescription) -> bool {
        self.allowlist.admits(description.name.as_str())
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
    use iceoryx2_bb_testing::assert_that;
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
    fn allow_all_list_admits_every_service_in_both_directions() {
        let sut = AllowListMapping::new(AllowList::all());
        let service = service_description("service");

        assert_that!(sut.remote(&service), is_some);
        assert_that!(sut.local::<ipc::Service>(&service), is_some);
    }

    #[test]
    fn admitted_services_are_mapped_unchanged() {
        let sut = AllowListMapping::new(AllowList::all());
        let service = service_description("service");

        assert_that!(sut.remote(&service), eq Some(service.clone()));
        assert_that!(sut.local::<ipc::Service>(&service), eq Some(service));
    }

    #[test]
    fn empty_allow_list_admits_no_service_in_both_directions() {
        let sut = AllowListMapping::default();
        let service = service_description("service");

        assert_that!(sut.remote(&service), is_none);
        assert_that!(sut.local::<ipc::Service>(&service), is_none);
    }

    #[test]
    fn allow_list_is_applied_in_both_directions() {
        let sut = AllowListMapping::new(AllowList::new(&["allowed"]));
        let allowed = service_description("allowed");
        let blocked = service_description("blocked");

        assert_that!(sut.remote(&allowed), is_some);
        assert_that!(sut.local::<ipc::Service>(&allowed), is_some);
        assert_that!(sut.remote(&blocked), is_none);
        assert_that!(sut.local::<ipc::Service>(&blocked), is_none);
    }
}
