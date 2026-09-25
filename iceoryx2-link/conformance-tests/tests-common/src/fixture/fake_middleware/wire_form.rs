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

use iceoryx2_link_adapter::{Passthrough, SampleShape, TranscodesSamples, Translator};
use iceoryx2_link_backend::service_description::SampleTypes;
use iceoryx2_link_testing::{FakeSwapTranslator, swapped_bytes};

/// The payload type every suite speaks through the fixtures.
pub(crate) type Payload = u64;

/// The header type the header suite speaks through the fixtures.
pub(crate) type Header = u64;

/// A payload as the fake middleware carries it.
pub(crate) type EncodedPayload = [u8; core::mem::size_of::<Payload>()];

/// A header as the fake middleware carries it.
pub(crate) type EncodedHeader = [u8; core::mem::size_of::<Header>()];

/// The form the fake middleware carries data in, as the far application
/// writes and reads it, paired with the translator that serves it.
pub(crate) trait WireForm:
    Translator<SampleShape, RemoteTypes = SampleTypes, Transcoders: TranscodesSamples> + Default
{
    fn encode_payload(payload: Payload) -> EncodedPayload;
    fn decode_payload(encoded: EncodedPayload) -> Payload;
    fn encode_header(header: Header) -> EncodedHeader;
    fn decode_header(encoded: EncodedHeader) -> Header;
}

impl WireForm for Passthrough {
    fn encode_payload(payload: Payload) -> EncodedPayload {
        payload.to_ne_bytes()
    }

    fn decode_payload(encoded: EncodedPayload) -> Payload {
        Payload::from_ne_bytes(encoded)
    }

    fn encode_header(header: Header) -> EncodedHeader {
        header.to_ne_bytes()
    }

    fn decode_header(encoded: EncodedHeader) -> Header {
        Header::from_ne_bytes(encoded)
    }
}

impl WireForm for FakeSwapTranslator {
    fn encode_payload(payload: Payload) -> EncodedPayload {
        swapped_bytes(payload)
    }

    fn decode_payload(encoded: EncodedPayload) -> Payload {
        Payload::from_ne_bytes(swapped_bytes(Payload::from_ne_bytes(encoded)))
    }

    fn encode_header(header: Header) -> EncodedHeader {
        swapped_bytes(header)
    }

    fn decode_header(encoded: EncodedHeader) -> Header {
        Header::from_ne_bytes(swapped_bytes(Header::from_ne_bytes(encoded)))
    }
}
