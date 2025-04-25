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

#![warn(missing_docs)]

//! Common components for internal iceoryx2 services.

/// Prefix used to identify internal iceoryx2 services.
///
/// This prefix is used to distinguish between user-defined services and internal
/// iceoryx2 services. Services with this prefix are considered internal and are
/// managed by the iceoryx2 system.
pub const INTERNAL_SERVICE_PREFIX: &str = "iox2://";

/// Checks if a service is an internal iceoryx2 service.
///
/// # Arguments
///
/// * `name` - The service name to check
///
/// # Returns
///
/// `true` if the service name starts with the iceoryx2 service prefix, `false` otherwise
pub fn is_internal_service(name: &str) -> bool {
    name.starts_with(INTERNAL_SERVICE_PREFIX)
}
