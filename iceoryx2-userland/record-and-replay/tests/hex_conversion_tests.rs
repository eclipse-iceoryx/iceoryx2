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

#[cfg(test)]
mod hex_conversion_tests {
    use iceoryx2_pal_testing::assert_that;
    use iceoryx2_userland_record_and_replay::hex_conversion::{
        bytes_to_hex_string, hex_string_to_bytes, HexToBytesConversionError,
    };

    #[test]
    fn back_and_forth_conversion_works() {
        let sut = "don't stop me now";
        let hex_sut = bytes_to_hex_string(sut.as_bytes());

        let roundtrip_sut = hex_string_to_bytes(&hex_sut).unwrap();
        assert_that!(sut.as_bytes(), eq roundtrip_sut);
    }

    #[test]
    fn invalid_hex_characters_fail() {
        let hex_sut = "0a ab fe xf";

        assert_that!(hex_string_to_bytes(&hex_sut).err(), eq Some(HexToBytesConversionError::InvalidHexCode));
    }

    #[test]
    fn empty_bytes_can_be_converted() {
        let sut = "";
        let hex_sut = bytes_to_hex_string(sut.as_bytes());

        let roundtrip_sut = hex_string_to_bytes(&hex_sut).unwrap();
        assert_that!(sut.as_bytes(), eq roundtrip_sut);
    }

    #[test]
    fn missing_spacing_fails() {
        let hex_sut = "0aabfe";

        assert_that!(hex_string_to_bytes(&hex_sut).err(), eq Some(HexToBytesConversionError::InvalidHexCode));
    }
}
