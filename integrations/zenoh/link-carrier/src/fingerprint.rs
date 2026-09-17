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

use iceoryx2_cal::hash::Hash;
use iceoryx2_cal::hash::sha1::Sha1;
use iceoryx2_link_backend::service_description::ServiceDescriptor;
use iceoryx2_log::{fail, origin};

/// The digest of a descriptor as encoded on the wire.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fingerprint(String);

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct InvalidFingerprint;

impl core::fmt::Display for InvalidFingerprint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "InvalidFingerprint")
    }
}

impl core::error::Error for InvalidFingerprint {}

impl Fingerprint {
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
            fail!(
                from origin!("Fingerprint::try_from"),
                with InvalidFingerprint,
                "Not a fingerprint: {:?}", value
            );
        }
        Ok(Self(value.into()))
    }
}

/// A descriptor encoded for the wire, and its fingerprint.
pub struct Encoded {
    bytes: Vec<u8>,
    fingerprint: Fingerprint,
}

impl Encoded {
    pub fn encode(descriptor: &ServiceDescriptor) -> Result<Self, postcard::Error> {
        let origin = origin!("Encoded::encode");

        let bytes = fail!(
            from origin,
            when postcard::to_allocvec(descriptor),
            "Failed to encode a service descriptor"
        );
        let fingerprint = Fingerprint::digest(&bytes);
        Ok(Self { bytes, fingerprint })
    }

    /// The descriptor in `bytes`, none unless they fingerprint to
    /// `fingerprint` and decode.
    pub fn decode(bytes: &[u8], fingerprint: &Fingerprint) -> Option<ServiceDescriptor> {
        if Fingerprint::digest(bytes) != *fingerprint {
            return None;
        }
        postcard::from_bytes(bytes).ok()
    }

    pub fn fingerprint(&self) -> &Fingerprint {
        &self.fingerprint
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn a_fingerprint_round_trips_through_its_text() {
        let fingerprint = Fingerprint::digest(b"fingerprint/round-trip");

        let parsed = Fingerprint::try_from(fingerprint.as_str()).expect("valid");

        assert_that!(parsed, eq fingerprint);
    }

    #[test]
    fn text_that_is_not_a_hex_digest_is_no_fingerprint() {
        const SHA1_HEX_LENGTH: usize = 40;

        assert_that!(Fingerprint::try_from(""), eq Err(InvalidFingerprint));
        assert_that!(Fingerprint::try_from("abc"), eq Err(InvalidFingerprint));
        assert_that!(
            Fingerprint::try_from("g".repeat(SHA1_HEX_LENGTH).as_str()),
            eq Err(InvalidFingerprint)
        );
    }

    #[test]
    fn bytes_of_another_fingerprint_do_not_decode() {
        let fingerprint = Fingerprint::digest(b"something else");

        assert_that!(Encoded::decode(b"not it", &fingerprint), is_none);
    }
}
