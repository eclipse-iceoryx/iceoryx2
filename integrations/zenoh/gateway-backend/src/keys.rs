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

use crate::descriptor::Fingerprint;
use crate::descriptor::ServiceDescriptor;

/// Namespace of all keys.
pub const NAMESPACE: &str = "iox2";

/// Version of the key scheme.
pub const VERSION: &str = "v1";

/// The zenoh key for discovering the details of announced service
/// descriptions.
pub fn service_discovery() -> String {
    format!("{NAMESPACE}/{VERSION}/service_description/*/*")
}

/// The zenoh key at which the details of the described service can be
/// received.
pub fn service_description(descriptor: &ServiceDescriptor) -> String {
    format!(
        "{NAMESPACE}/{VERSION}/service_description/{}/{}",
        descriptor.service_hash.as_str(),
        descriptor.fingerprint.as_str()
    )
}

/// Recovers the descriptor from a key built by [`service_description`].
pub fn parse_service_description(key: &str) -> Option<ServiceDescriptor> {
    let mut segments = key.rsplit('/');
    let fingerprint = Fingerprint::try_from(segments.next()?).ok()?;
    let service_hash = ServiceHash::try_from(segments.next()?).ok()?;
    Some(ServiceDescriptor {
        service_hash,
        fingerprint,
    })
}

/// The zenoh key at which payloads of the described publish-subscribe
/// service can be received.
pub fn publish_subscribe(descriptor: &ServiceDescriptor) -> String {
    format!(
        "{NAMESPACE}/{VERSION}/publish_subscribe/{}/{}",
        descriptor.service_hash.as_str(),
        descriptor.fingerprint.as_str()
    )
}

/// The zenoh key at which notifications of the described event service can
/// be received.
pub fn event(descriptor: &ServiceDescriptor) -> String {
    format!(
        "{NAMESPACE}/{VERSION}/event/{}/{}",
        descriptor.service_hash.as_str(),
        descriptor.fingerprint.as_str()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::service::ipc;
    use iceoryx2::service::service_name::ServiceName;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_gateway_backend::types::service_description::{
        EventDescription, PatternDescription, PortSettings, ServiceDescription,
    };

    #[test]
    fn service_description_key_round_trips() {
        let description = ServiceDescription::new::<ipc::Service>(
            ServiceName::new("keys/round-trip").expect("valid service name"),
            PatternDescription::Event(EventDescription {
                settings: PortSettings::LocalDefaults,
            }),
        );
        let descriptor = ServiceDescriptor::new(&description);

        let key = service_description(&descriptor);
        let parsed = parse_service_description(&key).expect("key is parsable");

        assert_that!(parsed, eq descriptor);
    }
}
