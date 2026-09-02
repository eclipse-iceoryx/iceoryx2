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

use iceoryx2_cal::hash::Hash;
use iceoryx2_cal::hash::sha1::Sha1;

/// A digest over the service description.
///
/// Equal fingerprints mean identical name, hash, pattern, types and
/// settings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fingerprint(String);

/// A string that is not the textual form of a [`Fingerprint`].
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

    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn fingerprint_to_string_round_trips() {
        let fingerprint = Fingerprint::digest(b"fingerprint/round-trip");

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
}
