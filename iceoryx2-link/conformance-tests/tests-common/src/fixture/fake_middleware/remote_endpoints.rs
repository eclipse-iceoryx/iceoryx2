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

use alloc::vec::Vec;

use iceoryx2_link_adapter::EndpointDescription;
use iceoryx2_link_backend::service_description::ServiceDescription;
use iceoryx2_link_conformance_tests::fixture::MessageEndpoints;
use iceoryx2_link_testing::{
    FakeEndpointDescription, FakeEndpointSettings, FakeEndpointTypes, FakeEndpoints, endpoint_of,
};

use super::wire_form::WireForm;

/// The endpoint `service` maps to, its types as `T` has the middleware
/// name them.
pub(crate) fn endpoint_for<T: WireForm>(service: &ServiceDescription) -> FakeEndpointDescription {
    EndpointDescription {
        types: T::default()
            .remote(service.types())
            .expect("the fake translators never fail"),
        ..endpoint_of(service)
    }
}

/// Remote endpoints on a fake middleware, on one endpoint.
pub(crate) struct RemoteMessageEndpoints {
    pub(crate) description: FakeEndpointDescription,
    pub(crate) remote: FakeEndpoints,
}

impl MessageEndpoints<FakeEndpointSettings, FakeEndpointTypes> for RemoteMessageEndpoints {
    fn description(&self) -> &FakeEndpointDescription {
        &self.description
    }

    fn send_message(&self, message: &[u8]) {
        self.remote.send(message);
    }

    fn receive_message(&self) -> Option<Vec<u8>> {
        self.remote.receive()
    }
}
