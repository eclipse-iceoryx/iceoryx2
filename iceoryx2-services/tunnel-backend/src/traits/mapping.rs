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

use core::fmt::Debug;

use iceoryx2::service::Service;

use crate::types::service_description::ServiceDescription;

/// Strategy for mapping iceoryx2 service descriptions to backend endpoint
/// descriptions.
pub trait Mapping: Default + Debug + Send + 'static {
    /// The backend-side description of a tunneled service's endpoints.
    type EndpointDescription;

    /// Maps an iceoryx2 service description to a backend's endpoint description,
    /// or `None` if the strategy does not map this service.
    fn remote(&self, description: &ServiceDescription) -> Option<Self::EndpointDescription>;

    /// Maps a remote backend's endpoint description to an iceoryx2 service description,
    /// or `None` if the strategy does not map this endpoint.
    fn local<S: Service>(&self, remote: &Self::EndpointDescription) -> Option<ServiceDescription>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Identity;

impl Mapping for Identity {
    type EndpointDescription = ServiceDescription;

    fn remote(&self, description: &ServiceDescription) -> Option<ServiceDescription> {
        Some(description.clone())
    }

    fn local<S: Service>(&self, remote: &ServiceDescription) -> Option<ServiceDescription> {
        Some(remote.clone())
    }
}
