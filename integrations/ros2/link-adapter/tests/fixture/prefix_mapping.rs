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

use iceoryx2::service::service_name::ServiceName;
use iceoryx2::testing::generate_service_name;
use iceoryx2_integrations_ros2_link_adapter::{AllowList, PrefixMapping};
use iceoryx2_link_conformance_tests::parameters::PublishSubscribeName;

use super::MappingUnderTest;

/// The prefix of the services the prefix mapping covers.
pub const SERVICE_PREFIX: &str = "ros2://topics";

/// Tests the prefix mapping with an allow list admitting every topic.
pub struct PrefixMapped;

impl MappingUnderTest for PrefixMapped {
    type Mapping = PrefixMapping;

    fn mapping(_: &str) -> PrefixMapping {
        PrefixMapping::new(AllowList::all())
    }
}

impl PublishSubscribeName for PrefixMapped {
    fn service_name() -> ServiceName {
        let unique = generate_service_name();
        ServiceName::new(&format!("{SERVICE_PREFIX}/{}", unique.as_str()))
            .expect("a valid service name")
    }
}
