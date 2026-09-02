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
use iceoryx2_gateway_backend::types::identity::GatewayId;

use crate::wire::descriptor::ServiceDescriptor;
use crate::wire::fingerprint::Fingerprint;

/// Namespace of all keys.
pub const NAMESPACE: &str = "iox2";

/// Version of the key scheme.
pub const VERSION: &str = "v1";

/// The zenoh key for discovering the details of service descriptions
/// announced by any gateway.
pub fn service_discovery() -> String {
    format!("{NAMESPACE}/{VERSION}/service_description/*/*/*")
}

/// The zenoh key at which the given gateway announces the details of the
/// described service.
pub fn service_description(descriptor: &ServiceDescriptor, gateway: &GatewayId) -> String {
    format!(
        "{NAMESPACE}/{VERSION}/service_description/{}/{}/{}",
        descriptor.service_hash.as_str(),
        descriptor.fingerprint.as_str(),
        gateway
    )
}

/// The zenoh key matching the details of the described service at any
/// gateway.
pub fn service_description_any(descriptor: &ServiceDescriptor) -> String {
    format!(
        "{NAMESPACE}/{VERSION}/service_description/{}/{}/*",
        descriptor.service_hash.as_str(),
        descriptor.fingerprint.as_str()
    )
}

/// Recovers the descriptor and gateway from a key built by
/// [`service_description`].
pub fn parse_service_description(key: &str) -> Option<(ServiceDescriptor, GatewayId)> {
    let mut segments = key.rsplit('/');
    let gateway = segments.next()?.parse().ok()?;
    let fingerprint = Fingerprint::try_from(segments.next()?).ok()?;
    let service_hash = ServiceHash::try_from(segments.next()?).ok()?;
    Some((ServiceDescriptor::new(service_hash, fingerprint), gateway))
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

    use iceoryx2::node::NodeBuilder;
    use iceoryx2::service::ipc;
    use iceoryx2::service::service_name::ServiceName;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_gateway_backend::types::identity::{BACKEND_ID_LENGTH, BackendId};
    use iceoryx2_gateway_backend::types::service_description::{
        EventDescription, PatternDescription, PortSettings, ServiceDescription,
    };

    use crate::wire::descriptor::describe;

    #[test]
    fn service_description_key_round_trips() {
        let description = ServiceDescription::new::<ipc::Service>(
            ServiceName::new("keys/round-trip").expect("valid service name"),
            PatternDescription::Event(EventDescription {
                settings: PortSettings::LocalDefaults,
            }),
        );
        let descriptor = describe(&description).expect("description is encodable");
        let node = NodeBuilder::new()
            .create::<ipc::Service>()
            .expect("node creation succeeds");
        let gateway = GatewayId::new(*node.id(), BackendId::new([7; BACKEND_ID_LENGTH]));

        let key = service_description(&descriptor, &gateway);
        let parsed = parse_service_description(&key).expect("key is parsable");

        assert_that!(parsed, eq(descriptor, gateway));
    }
}
