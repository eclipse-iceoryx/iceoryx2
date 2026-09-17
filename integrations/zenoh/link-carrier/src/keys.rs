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
use iceoryx2_link_carrier::PeerId;
use zenoh::key_expr::{OwnedKeyExpr, keyexpr};

use crate::fingerprint::Fingerprint;

/// Namespace of all keys.
pub const NAMESPACE: &str = "iox2";

/// Version of the key scheme.
pub const VERSION: &str = "v1";

/// The key matching every peer's offers.
pub fn offers() -> OwnedKeyExpr {
    key(format!("{NAMESPACE}/{VERSION}/offer/*/*/*"))
}

/// The key `peer` offers the service under.
pub fn offer(hash: &ServiceHash, fingerprint: &Fingerprint, peer: &PeerId) -> OwnedKeyExpr {
    key(format!(
        "{NAMESPACE}/{VERSION}/offer/{}/{}/{}",
        hash.as_str(),
        fingerprint.as_str(),
        peer
    ))
}

/// The key matching the service's channels under every description.
pub fn channels_of(hash: &ServiceHash) -> OwnedKeyExpr {
    key(format!("{NAMESPACE}/{VERSION}/channel/{}/*", hash.as_str()))
}

/// Recovers hash, fingerprint and peer from a key built by [`offer`].
pub fn parse_offer(key: &keyexpr) -> Option<(ServiceHash, Fingerprint, PeerId)> {
    let mut segments = key.as_str().rsplit('/');
    let peer = parse_peer(segments.next()?)?;
    let fingerprint = Fingerprint::try_from(segments.next()?).ok()?;
    let hash = ServiceHash::try_from(segments.next()?).ok()?;
    Some((hash, fingerprint, peer))
}

/// The key the frames of the service cross on.
pub fn channel(hash: &ServiceHash, fingerprint: &Fingerprint) -> OwnedKeyExpr {
    key(format!(
        "{NAMESPACE}/{VERSION}/channel/{}/{}",
        hash.as_str(),
        fingerprint.as_str()
    ))
}

/// A key built from text this module formats, always well formed.
fn key(text: String) -> OwnedKeyExpr {
    OwnedKeyExpr::try_from(text).expect("a key this module formats is well formed")
}

fn parse_peer(text: &str) -> Option<PeerId> {
    if text.len() != 2 * PeerId::LENGTH {
        return None;
    }
    let mut bytes = [0u8; PeerId::LENGTH];
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[2 * i..2 * i + 2], 16).ok()?;
    }
    Some(PeerId::new(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::service::messaging_pattern::MessagingPattern;
    use iceoryx2::service::service_name::ServiceName;
    use iceoryx2::service::{Service, local};
    use iceoryx2_bb_testing::assert_that;

    const PEER: u8 = 7;

    #[test]
    fn an_offer_key_round_trips() {
        const SERVICE: &str = "keys/round-trip";

        let hash = ServiceHash::new::<<local::Service as Service>::ServiceNameHasher>(
            &ServiceName::new(SERVICE).expect("valid service name"),
            MessagingPattern::PublishSubscribe,
        );
        let fingerprint = Fingerprint::digest(b"keys/round-trip");
        let peer = PeerId::new([PEER; PeerId::LENGTH]);
        let key = offer(&hash, &fingerprint, &peer);

        assert_that!(parse_offer(&key), eq Some((hash, fingerprint, peer)));
    }

    #[test]
    fn a_key_that_is_not_an_offer_does_not_parse() {
        let key = keyexpr::new("iox2/v1/offer/not/an/offer").expect("a valid key expression");

        assert_that!(parse_offer(key), is_none);
    }
}
