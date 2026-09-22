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

mod prefix_mapping;
mod static_mapping;

pub use prefix_mapping::PrefixMapped;
pub use static_mapping::StaticMapped;

use iceoryx2_integrations_ros2_link_adapter::TopicSettings;
use iceoryx2_link_adapter::Mapping;
use iceoryx2_link_conformance_tests::parameters::PublishSubscribeName;

/// A mapping under test. It also names the services it covers.
pub trait MappingUnderTest: PublishSubscribeName + 'static {
    type Mapping: Mapping<EndpointSettings = TopicSettings>;

    /// The mapping for services with the payload type `payload_type`.
    fn mapping(payload_type: &str) -> Self::Mapping;
}
