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

use xxhash_rust::xxh3::xxh3_128;

pub(super) const GID_SIZE: usize = 16;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Gid(pub [u8; GID_SIZE]);

impl Gid {
    pub fn from_token_key(key: &str) -> Self {
        Self(xxh3_128(key.as_bytes()).to_le_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TALKER_PUBLISHER_TOKEN: &str = "@ros2_lv/0/6eeb7dddf6026d46786c8f11706568af/0/11/MP/%/%/talker/%chatter/std_msgs::msg::dds_::String_/RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18/::,7:,:,:,,";
    const TALKER_PUBLISHER_GID: [u8; GID_SIZE] = [
        0x2a, 0xaf, 0xd1, 0xf8, 0xdf, 0xec, 0xe2, 0x28, 0x8c, 0xb9, 0x8d, 0x1f, 0x04, 0xed, 0x0d,
        0x66,
    ];

    #[test]
    fn gid_of_a_token_matches_the_captured_attachment() {
        assert_eq!(
            Gid::from_token_key(TALKER_PUBLISHER_TOKEN),
            Gid(TALKER_PUBLISHER_GID)
        );
    }
}
