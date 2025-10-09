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

use core::error::Error;

use crate::types::discovery::ProcessDiscoveryFn;

/// Discovery functionality to discover services utilizing the backend
/// communication mechanism.
pub trait Discovery {
    /// Error type that can occur during discovery operations.
    type DiscoveryError: Error;

    /// Discovers entities and processes them using the provided discovery
    /// function.
    fn discover<ProcessDiscoveryError>(
        &self,
        process_discovery: &mut ProcessDiscoveryFn<ProcessDiscoveryError>,
    ) -> Result<(), Self::DiscoveryError>;
}
