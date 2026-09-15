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

use iceoryx2_link_adapter::Mapping;
use iceoryx2_link_backend::service_description::{Identified, ServiceSettings};

use crate::adapter::FakeEndpointSettings;

/// Maps endpoints to the services of the same name, defining nothing.
#[derive(Debug, Default, Clone, Copy)]
pub struct FakeMapping;

impl Mapping for FakeMapping {
    type EndpointSettings = FakeEndpointSettings;
    type Error = core::convert::Infallible;

    fn local(&self, remote: &FakeEndpointSettings) -> Result<Option<ServiceSettings>, Self::Error> {
        Ok(Some(ServiceSettings::new(
            remote.name,
            remote.settings.clone(),
        )))
    }

    fn remote(&self, local: &ServiceSettings) -> Result<Option<FakeEndpointSettings>, Self::Error> {
        Ok(Some(FakeEndpointSettings {
            name: local.id(),
            settings: local.pattern.clone(),
        }))
    }
}
