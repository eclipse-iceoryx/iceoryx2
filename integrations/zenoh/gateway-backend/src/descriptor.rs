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

//! Identity of a service description on the zenoh wire.

use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_cal::hash::Hash;
use iceoryx2_cal::hash::sha1::Sha1;
use iceoryx2_gateway_backend::types::service_description::ServiceDescription;

/// Identifies a [`ServiceDescription`] on the wire, the service hash it
/// belongs to and its fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ServiceDescriptor {
    pub service_hash: ServiceHash,
    pub fingerprint: Fingerprint,
}

impl ServiceDescriptor {
    pub fn new(service_hash: ServiceHash, fingerprint: Fingerprint) -> Self {
        Self {
            service_hash,
            fingerprint,
        }
    }
}

/// Derives the descriptor of a description by encoding it.
pub fn describe(description: &ServiceDescription) -> Result<ServiceDescriptor, postcard::Error> {
    let encoded = EncodedDescription::encode(description)?;
    Ok(ServiceDescriptor::new(
        description.service_hash,
        encoded.fingerprint(),
    ))
}

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

/// A digest over an entire [`ServiceDescription`].
///
/// Equal fingerprints mean identical name, hash, pattern, types and
/// settings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fingerprint(String);

/// A string that is not the textual form of a [`DescriptionFingerprint`].
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct InvalidFingerprint;

impl core::fmt::Display for InvalidFingerprint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "InvalidFingerprint")
    }
}

impl core::error::Error for InvalidFingerprint {}

impl Fingerprint {
    /// Create a fingerprint from a byte digest.
    pub fn digest(bytes: &[u8]) -> Self {
        Self(Sha1::new(bytes).value().into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for Fingerprint {
    type Error = InvalidFingerprint;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        const SHA1_HEX_LENGTH: usize = 40;

        let is_hex_digest =
            value.len() == SHA1_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_hexdigit());
        if !is_hex_digest {
            return Err(InvalidFingerprint);
        }
        Ok(Self(value.into()))
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
    fn fingerprint_to_string_round_trips() {
        let fingerprint = fingerprint_of(&service_description(
            "fingerprint/round-trip",
            type_description("u64", 8),
            PublishSubscribeSettings::default(),
        ));

        let parsed = Fingerprint::try_from(fingerprint.as_str())
            .expect("textual form of a fingerprint is valid");

        assert_that!(parsed, eq fingerprint);
    }

    #[test]
    fn rejects_text_that_is_not_a_hex_digest() {
        const SHA1_HEX_LENGTH: usize = 40;
        assert_that!(Fingerprint::try_from(""), eq Err(InvalidFingerprint));
        assert_that!(Fingerprint::try_from("abc"), eq Err(InvalidFingerprint));
        assert_that!(
            Fingerprint::try_from("g".repeat(SHA1_HEX_LENGTH).as_str()),
            eq Err(InvalidFingerprint)
        );
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
