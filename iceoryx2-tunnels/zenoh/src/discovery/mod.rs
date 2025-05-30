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

use iceoryx2::service::static_config::StaticConfig as IceoryxServiceConfig;

mod iceoryx;
mod zenoh;

pub(crate) use iceoryx::*;
pub(crate) use zenoh::*;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    Error,
}

impl core::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        core::write!(f, "DiscoveryError::{:?}", self)
    }
}

impl core::error::Error for DiscoveryError {}

pub(crate) trait Discovery<S: iceoryx2::service::Service> {
    fn discover<OnDiscovered: FnMut(&IceoryxServiceConfig)>(
        &mut self,
        on_discovered: &mut OnDiscovered,
    ) -> Result<(), DiscoveryError>;
}
