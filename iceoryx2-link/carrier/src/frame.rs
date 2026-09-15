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

use alloc::vec::Vec;

use iceoryx2_link_backend::service_description::PublishSubscribeTypes;
use iceoryx2_link_backend::wire::publish_subscribe::fits;
use iceoryx2_log::{fail, origin};

/// A publish-subscribe sample as it crosses a channel, the user header
/// bytes followed by the payload bytes.
#[derive(Debug, Clone, Copy)]
pub struct Frame<'a> {
    pub header: &'a [u8],
    pub payload: &'a [u8],
}

impl<'a> Frame<'a> {
    /// The frame as one buffer, header then payload.
    pub fn to_bytes(&self) -> Vec<u8> {
        [self.header, self.payload].concat()
    }

    /// The frame in `bytes`, malformed if header or payload do not fit
    /// the types.
    pub fn parse(bytes: &'a [u8], types: &PublishSubscribeTypes) -> Result<Self, Malformed> {
        let origin = origin!("Frame::parse");
        let header_size = types.user_header.size;
        if bytes.len() < header_size {
            fail!(
                from origin,
                with Malformed,
                "A frame of {} bytes cannot hold a user header of {}", bytes.len(), header_size
            );
        }
        let (header, payload) = bytes.split_at(header_size);
        if !fits(types, header.len(), payload.len()) {
            fail!(
                from origin,
                with Malformed,
                "A payload of {} bytes does not fit the service's types", payload.len()
            );
        }
        Ok(Self { header, payload })
    }
}

/// The bytes do not fit the service's description.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Malformed;

impl core::fmt::Display for Malformed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Malformed")
    }
}

impl core::error::Error for Malformed {}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::string::String;
    use iceoryx2::service::static_config::message_type_details::TypeVariant;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_link_backend::service_description::{PublishSubscribeTypes, TypeDescription};

    const HEADER_SIZE: usize = 2;
    const PAYLOAD_SIZE: usize = 3;
    const ELEMENT_SIZE: usize = 2;
    const ALIGNMENT: usize = 1;
    const BYTES: [u8; 6] = [1, 2, 3, 4, 5, 6];

    fn types(variant: TypeVariant, payload_size: usize) -> PublishSubscribeTypes {
        PublishSubscribeTypes {
            payload: TypeDescription {
                variant,
                type_name: String::from("payload"),
                size: payload_size,
                alignment: ALIGNMENT,
            },
            user_header: TypeDescription {
                variant: TypeVariant::FixedSize,
                type_name: String::from("header"),
                size: HEADER_SIZE,
                alignment: ALIGNMENT,
            },
        }
    }

    #[test]
    fn bytes_split_into_header_and_payload() {
        let types = types(TypeVariant::FixedSize, PAYLOAD_SIZE);
        let frame = Frame::parse(&BYTES[..HEADER_SIZE + PAYLOAD_SIZE], &types).expect("fits");
        assert_that!(frame.header, eq & BYTES[..HEADER_SIZE]);
        assert_that!(
            frame.payload,
            eq & BYTES[HEADER_SIZE..HEADER_SIZE + PAYLOAD_SIZE]
        );
    }

    #[test]
    fn bytes_shorter_than_the_header_are_malformed() {
        let types = types(TypeVariant::FixedSize, PAYLOAD_SIZE);
        assert_that!(Frame::parse(&BYTES[..HEADER_SIZE - 1], &types).is_err(), eq true);
    }

    #[test]
    fn fixed_size_payload_of_wrong_size_is_malformed() {
        let types = types(TypeVariant::FixedSize, PAYLOAD_SIZE);
        assert_that!(Frame::parse(&BYTES[..HEADER_SIZE + PAYLOAD_SIZE - 1], &types).is_err(), eq true);
        assert_that!(Frame::parse(&BYTES[..HEADER_SIZE + PAYLOAD_SIZE + 1], &types).is_err(), eq true);
    }

    #[test]
    fn dynamic_payload_must_be_a_multiple_of_the_element_size() {
        let types = types(TypeVariant::Dynamic, ELEMENT_SIZE);
        assert_that!(Frame::parse(&BYTES[..HEADER_SIZE], &types).is_ok(), eq true);
        assert_that!(Frame::parse(&BYTES[..HEADER_SIZE + 2 * ELEMENT_SIZE], &types).is_ok(), eq true);
        assert_that!(Frame::parse(&BYTES[..HEADER_SIZE + ELEMENT_SIZE / 2], &types).is_err(), eq true);
    }
}
