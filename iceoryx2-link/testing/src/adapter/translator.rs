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

use alloc::format;
use alloc::string::String;
use core::convert::Infallible;

use iceoryx2_link_adapter::{
    HeaderTranscoder, LoanableSample, PayloadTranscoder, PublishSubscribeTranslation, Region,
    SampleTranscoders, SampleTranscodings, TranscodeError, Translator, WritableSample,
};
use iceoryx2_link_backend::service_description::{SampleTypes, ServiceTypes, TypeDescription};
use iceoryx2_log::{fail, origin};

use crate::adapter::FakeEndpointTypes;

/// The prefix of a type name on the middleware, its data byte-swapped.
const SWAPPED: &str = "swapped:";

/// The width of the words the middleware's data is swapped in.
const WORD: usize = 8;

/// The translator of a fake middleware whose data is the local
/// data with every eight-byte word reversed, header and payload alike.
#[derive(Debug, Default, Clone, Copy)]
pub struct FakeSwapTranslator;

impl Translator for FakeSwapTranslator {
    type EndpointTypes = FakeEndpointTypes;
    type Error = Infallible;
    type Transcoder = SwapTranscoder;

    fn local(&self, remote: &FakeEndpointTypes) -> Result<ServiceTypes, Self::Error> {
        Ok(map_types(remote, unswapped))
    }

    fn remote(&self, local: &ServiceTypes) -> Result<FakeEndpointTypes, Self::Error> {
        Ok(map_types(local, swapped))
    }

    fn publish_subscribe(
        &self,
        local: &ServiceTypes,
        _: &FakeEndpointTypes,
    ) -> Result<PublishSubscribeTranslation<SwapTranscoder>, Self::Error> {
        Ok(match local {
            ServiceTypes::PublishSubscribe(_) => PublishSubscribeTranslation::Transcode {
                outbound: SampleTranscodings::TRANSCODE,
                inbound: SampleTranscodings::TRANSCODE,
                transcoder: SampleTranscoders {
                    headers: SwapHeader,
                    payloads: SwapPayload,
                },
            },
            ServiceTypes::Event => PublishSubscribeTranslation::Passthrough,
        })
    }
}

fn map_types(types: &ServiceTypes, name: fn(&str) -> String) -> ServiceTypes {
    match types {
        ServiceTypes::PublishSubscribe(types) => ServiceTypes::PublishSubscribe(SampleTypes {
            payload: map_type(&types.payload, name),
            user_header: map_type(&types.user_header, name),
        }),
        ServiceTypes::Event => ServiceTypes::Event,
    }
}

fn map_type(description: &TypeDescription, name: fn(&str) -> String) -> TypeDescription {
    TypeDescription {
        type_name: name(&description.type_name),
        ..description.clone()
    }
}

fn swapped(name: &str) -> String {
    format!("{SWAPPED}{name}")
}

fn unswapped(name: &str) -> String {
    String::from(name.strip_prefix(SWAPPED).unwrap_or(name))
}

/// Reverses the bytes of every eight-byte word of both regions, in either
/// direction.
pub type SwapTranscoder = SampleTranscoders<SwapHeader, SwapPayload>;

/// Reverses the bytes of every eight-byte word of the header.
#[derive(Debug, Default, Clone, Copy)]
pub struct SwapHeader;

/// Reverses the bytes of every eight-byte word of the payload.
#[derive(Debug, Default, Clone, Copy)]
pub struct SwapPayload;

fn swap_words<R: Region>(bytes: &[u8], into: &mut R) -> Result<(), TranscodeError<Infallible>> {
    let origin = origin!("swap_words");

    let into = match into.for_length(bytes.len()) {
        Ok(into) => into,
        Err(refusal) => {
            fail!(
                from origin,
                with TranscodeError::from(refusal),
                "The region rejected the {} bytes to swap", bytes.len()
            );
        }
    };
    swap_words_into(bytes, into);
    Ok(())
}

fn swap_words_into(bytes: &[u8], into: &mut [u8]) {
    for (to, from) in into.chunks_mut(WORD).zip(bytes.chunks(WORD)) {
        for (to, from) in to.iter_mut().zip(from.iter().rev()) {
            *to = *from;
        }
    }
}

impl HeaderTranscoder for SwapHeader {
    type Failure = Infallible;

    fn encode<R: Region>(
        &self,
        header: &[u8],
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Failure>> {
        swap_words(header, into)
    }

    fn decode<W: WritableSample>(
        &self,
        wire: &[u8],
        writable: &mut W,
    ) -> Result<(), TranscodeError<Self::Failure>> {
        let origin = origin!("SwapHeader::decode");

        let into = match writable.header(wire.len()) {
            Ok(into) => into,
            Err(refusal) => {
                fail!(
                    from origin,
                    with TranscodeError::from(refusal),
                    "The sample rejected a header of {} bytes to swap", wire.len()
                );
            }
        };
        swap_words_into(wire, into);
        Ok(())
    }
}

impl PayloadTranscoder for SwapPayload {
    type Failure = Infallible;

    fn encode<R: Region>(
        &self,
        payload: &[u8],
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Failure>> {
        swap_words(payload, into)
    }

    fn decode<L: LoanableSample>(
        &self,
        wire: &[u8],
        loanable: L,
    ) -> Result<L::Sample, TranscodeError<Self::Failure>> {
        let origin = origin!("SwapPayload::decode");

        let mut writable = match loanable.loan(wire.len()) {
            Ok(writable) => writable,
            Err(refusal) => {
                fail!(
                    from origin,
                    with TranscodeError::from(refusal),
                    "The sample rejected the {} bytes to swap", wire.len()
                );
            }
        };
        swap_words_into(wire, writable.payload());
        Ok(writable)
    }
}

/// The bytes of `value` as the swapping middleware carries them.
pub fn swapped_bytes(value: u64) -> [u8; WORD] {
    let mut bytes = value.to_ne_bytes();
    bytes.reverse();
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn the_type_names_round_trip() {
        let description = TypeDescription::from(&TypeDetail::new::<u64>(TypeVariant::FixedSize));
        let local = ServiceTypes::PublishSubscribe(SampleTypes {
            payload: description.clone(),
            user_header: description,
        });

        let remote = FakeSwapTranslator.remote(&local).expect("never fails");

        assert_that!(FakeSwapTranslator.local(&remote).expect("never fails"), eq local);
    }
}
