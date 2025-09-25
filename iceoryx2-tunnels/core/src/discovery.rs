// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use iceoryx2::service::static_config::StaticConfig;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Error {
    Error,
}

/// Enables implementation of discovery behaviour.
pub trait Discovery<ServiceType: iceoryx2::service::Service> {
    type DiscoveryError;

    fn discover<OnDiscovered: FnMut(&StaticConfig) -> Result<(), Self::DiscoveryError>>(
        &mut self,
        on_discovered: &mut OnDiscovered,
    ) -> Result<(), Self::DiscoveryError>;
}
