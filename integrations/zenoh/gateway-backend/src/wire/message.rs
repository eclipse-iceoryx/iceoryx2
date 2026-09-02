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

use iceoryx2::service::marker::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2::service::static_config::message_type_details::TypeVariant;
use iceoryx2_gateway_backend::types::service_description::TypeDescription;
use serde::{Deserialize, Serialize};

/// Frame carrying the user header and payload of one message.
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageFrame<'a> {
    pub user_header: &'a [u8],
    pub payload: &'a [u8],
}

/// Checks that a frame fits the given user header and payload types.
pub fn validate_frame(
    frame: &MessageFrame<'_>,
    user_header: &TypeDescription,
    payload: &TypeDescription,
) -> bool {
    let user_header_matches = frame.user_header.len() == user_header.size;
    let payload_matches = match payload.variant {
        TypeVariant::FixedSize => frame.payload.len() == payload.size,
        TypeVariant::Dynamic => {
            payload.size != 0 && frame.payload.len().is_multiple_of(payload.size)
        }
    };
    user_header_matches && payload_matches
}

/// Views the user header of an untyped message as bytes.
pub fn user_header_bytes(user_header: &CustomHeaderMarker, size: usize) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(user_header as *const CustomHeaderMarker as *const u8, size)
    }
}

/// Views the payload of an untyped message as bytes.
pub fn payload_bytes(payload: &[CustomPayloadMarker]) -> &[u8] {
    unsafe { core::slice::from_raw_parts(payload.as_ptr() as *const u8, payload.len()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2_bb_testing::assert_that;

    fn type_description(variant: TypeVariant, size: usize) -> TypeDescription {
        TypeDescription {
            variant,
            type_name: "test_type".into(),
            size,
            alignment: 1,
        }
    }

    #[test]
    fn accepts_matching_fixed_size_message() {
        let user_header = type_description(TypeVariant::FixedSize, 4);
        let payload = type_description(TypeVariant::FixedSize, 8);
        let frame = MessageFrame {
            user_header: &[0u8; 4],
            payload: &[0u8; 8],
        };

        assert_that!(validate_frame(&frame, &user_header, &payload), eq true);
    }

    #[test]
    fn accepts_dynamic_payload_of_whole_elements() {
        let user_header = type_description(TypeVariant::FixedSize, 0);
        let payload = type_description(TypeVariant::Dynamic, 4);

        for element_count in [0usize, 1, 3] {
            let bytes = vec![0u8; element_count * 4];
            let frame = MessageFrame {
                user_header: &[],
                payload: &bytes,
            };
            assert_that!(validate_frame(&frame, &user_header, &payload), eq true);
        }
    }

    #[test]
    fn rejects_mismatched_user_header_size() {
        let user_header = type_description(TypeVariant::FixedSize, 4);
        let payload = type_description(TypeVariant::FixedSize, 8);
        let frame = MessageFrame {
            user_header: &[0u8; 8],
            payload: &[0u8; 8],
        };

        assert_that!(validate_frame(&frame, &user_header, &payload), eq false);
    }

    #[test]
    fn rejects_mismatched_fixed_size_payload() {
        let user_header = type_description(TypeVariant::FixedSize, 0);
        let payload = type_description(TypeVariant::FixedSize, 8);
        let frame = MessageFrame {
            user_header: &[],
            payload: &[0u8; 12],
        };

        assert_that!(validate_frame(&frame, &user_header, &payload), eq false);
    }

    #[test]
    fn rejects_dynamic_payload_of_partial_elements() {
        let user_header = type_description(TypeVariant::FixedSize, 0);
        let payload = type_description(TypeVariant::Dynamic, 4);
        let frame = MessageFrame {
            user_header: &[],
            payload: &[0u8; 6],
        };

        assert_that!(validate_frame(&frame, &user_header, &payload), eq false);
    }
}
