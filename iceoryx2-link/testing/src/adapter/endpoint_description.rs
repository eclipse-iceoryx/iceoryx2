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
use iceoryx2_link_adapter::EndpointDescription;
use iceoryx2_link_backend::service_description::Identified;
use iceoryx2_link_backend::service_description::ServiceDescription;
use iceoryx2_link_backend::service_description::{PatternSettings, ServiceTypes};

/// The fake middleware's settings of an endpoint, its name and the
/// settings it was created with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeEndpointSettings {
    pub name: ServiceName,
    pub settings: PatternSettings,
}

impl Identified for FakeEndpointSettings {
    type Id = ServiceName;

    fn id(&self) -> ServiceName {
        self.name
    }
}

/// The fake middleware's types of an endpoint, the local
/// types themselves, its data needing no translation.
pub type FakeEndpointTypes = ServiceTypes;

/// The fake middleware's description of an endpoint.
pub type FakeEndpointDescription = EndpointDescription<FakeEndpointSettings, FakeEndpointTypes>;

/// The endpoint a service maps to under the fake mapping.
pub fn endpoint_of(description: &ServiceDescription) -> FakeEndpointDescription {
    EndpointDescription {
        settings: FakeEndpointSettings {
            name: description.name(),
            settings: description.settings().pattern.clone(),
        },
        types: description.types().clone(),
    }
}

/// The size of the user header leading each message under `description`.
pub fn header_size(description: &FakeEndpointDescription) -> usize {
    match &description.types {
        ServiceTypes::PublishSubscribe(types) => types.user_header.size,
        ServiceTypes::Event => 0,
    }
}
