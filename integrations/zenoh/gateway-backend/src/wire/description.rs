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

use iceoryx2_gateway_backend::types::service_description::ServiceDescription;

use crate::wire::fingerprint::Fingerprint;

/// A service description in the form carried on the zenoh wire.
#[derive(Debug, Clone)]
pub struct EncodedDescription(Vec<u8>);

impl EncodedDescription {
    pub fn encode(description: &ServiceDescription) -> Result<Self, postcard::Error> {
        postcard::to_allocvec(description).map(Self)
    }

    /// Decodes wire bytes produced by encode.
    pub fn decode(bytes: &[u8]) -> Result<ServiceDescription, postcard::Error> {
        postcard::from_bytes(bytes)
    }

    pub fn fingerprint(&self) -> Fingerprint {
        Fingerprint::digest(&self.0)
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::service::ipc;
    use iceoryx2::service::service_name::ServiceName;
    use iceoryx2::service::static_config::message_type_details::TypeVariant;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_gateway_backend::types::service_description::{
        PatternDescription, PortSettings, PublishSubscribeDescription, PublishSubscribeSettings,
        TypeDescription,
    };

    fn type_description(type_name: &str, size: usize) -> TypeDescription {
        TypeDescription {
            variant: TypeVariant::FixedSize,
            type_name: type_name.into(),
            size,
            alignment: size,
        }
    }

    fn service_description(
        name: &str,
        payload: TypeDescription,
        settings: PublishSubscribeSettings,
    ) -> ServiceDescription {
        ServiceDescription::new::<ipc::Service>(
            ServiceName::new(name).expect("valid service name"),
            PatternDescription::PublishSubscribe(PublishSubscribeDescription {
                user_header: type_description("()", 0),
                payload,
                settings: PortSettings::Value(settings),
            }),
        )
    }

    fn fingerprint_of(description: &ServiceDescription) -> Fingerprint {
        EncodedDescription::encode(description)
            .expect("description is encodable")
            .fingerprint()
    }

    #[test]
    fn identical_descriptions_share_a_fingerprint() {
        let first = service_description(
            "fingerprint/identical",
            type_description("u64", 8),
            PublishSubscribeSettings::default(),
        );
        let second = service_description(
            "fingerprint/identical",
            type_description("u64", 8),
            PublishSubscribeSettings::default(),
        );

        assert_that!(fingerprint_of(&first), eq fingerprint_of(&second));
    }

    #[test]
    fn differing_names_produce_differing_fingerprints() {
        let first = service_description(
            "fingerprint/name-a",
            type_description("u64", 8),
            PublishSubscribeSettings::default(),
        );
        let second = service_description(
            "fingerprint/name-b",
            type_description("u64", 8),
            PublishSubscribeSettings::default(),
        );

        assert_that!(fingerprint_of(&first), ne fingerprint_of(&second));
    }

    #[test]
    fn differing_types_produce_differing_fingerprints() {
        let first = service_description(
            "fingerprint/types",
            type_description("u64", 8),
            PublishSubscribeSettings::default(),
        );
        let second = service_description(
            "fingerprint/types",
            type_description("i64", 8),
            PublishSubscribeSettings::default(),
        );

        assert_that!(fingerprint_of(&first), ne fingerprint_of(&second));
    }

    #[test]
    fn differing_settings_produce_differing_fingerprints() {
        let first = service_description(
            "fingerprint/settings",
            type_description("u64", 8),
            PublishSubscribeSettings::default(),
        );
        let second = service_description(
            "fingerprint/settings",
            type_description("u64", 8),
            PublishSubscribeSettings {
                history_size: PublishSubscribeSettings::default().history_size + 1,
                ..PublishSubscribeSettings::default()
            },
        );

        assert_that!(fingerprint_of(&first), ne fingerprint_of(&second));
    }
}
