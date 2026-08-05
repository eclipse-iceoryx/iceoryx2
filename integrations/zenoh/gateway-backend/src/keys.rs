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

use iceoryx2::service::service_hash::ServiceHash;

/// The zenoh key for discovering available service details.
pub fn service_discovery() -> String {
    "iox2/service_details/*".into()
}

/// The zenoh key at which the service details for the given service id can be received.
pub fn service_details(service_hash: &ServiceHash) -> String {
    format!("iox2/service_details/{}", service_hash.as_str())
}

/// The zenoh key at which payloads for a given publish-subscribe service id can be received.
pub fn publish_subscribe(service_hash: &ServiceHash) -> String {
    format!("iox2/publish_subscribe/{}", service_hash.as_str())
}

/// The zenoh key at which notifications for a given event service can be received.
pub fn event(service_hash: &ServiceHash) -> String {
    format!("iox2/event/{}", service_hash.as_str())
}
