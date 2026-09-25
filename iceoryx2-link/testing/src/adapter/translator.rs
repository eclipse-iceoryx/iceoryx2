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
    LocalTypes, Region, SampleBytesRef, SampleShape, SampleTranscoders, TranscodeError, Transcoder,
    Translator,
};
use iceoryx2_link_backend::service_description::{SampleTypes, TypeDescription};
use iceoryx2_log::{fail, origin};

/// The prefix of a type name on the middleware, its data byte-swapped.
const SWAPPED: &str = "swapped:";

/// The width of the words the middleware's data is swapped in.
const WORD: usize = 8;

/// The translator of a fake middleware whose data is the local
/// data with every eight-byte word reversed, header and payload alike.
#[derive(Debug, Default, Clone, Copy)]
pub struct FakeSwapTranslator;

impl Translator<SampleShape> for FakeSwapTranslator {
    type RemoteTypes = SampleTypes;
    type Transcoders = SampleTranscoders<SwapHeader, SwapPayload>;
    type Error = Infallible;

    fn local(&self, remote: &SampleTypes) -> Result<LocalTypes<SampleShape>, Self::Error> {
        Ok(map_types(remote, unswapped))
    }

    fn remote(&self, local: &LocalTypes<SampleShape>) -> Result<SampleTypes, Self::Error> {
        Ok(map_types(local, swapped))
    }

    fn transcoders(
        &self,
        _: &LocalTypes<SampleShape>,
        _: &SampleTypes,
    ) -> Result<Self::Transcoders, Self::Error> {
        Ok(SampleTranscoders::TranscodeBoth(SwapHeader, SwapPayload))
    }
}

fn map_types(types: &SampleTypes, name: fn(&str) -> String) -> SampleTypes {
    SampleTypes {
        payload: map_type(&types.payload, name),
        user_header: map_type(&types.user_header, name),
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
    for (to, from) in into.chunks_mut(WORD).zip(bytes.chunks(WORD)) {
        for (to, from) in to.iter_mut().zip(from.iter().rev()) {
            *to = *from;
        }
    }
    Ok(())
}

impl<'a> Transcoder<SampleBytesRef<'a>> for SwapHeader {
    type Error = Infallible;

    fn encode<R: Region>(
        &self,
        local: SampleBytesRef<'a>,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>> {
        swap_words(local.header, into)
    }

    fn decode<R: Region>(
        &self,
        wire: SampleBytesRef<'a>,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>> {
        swap_words(wire.header, into)
    }
}

impl<'a> Transcoder<SampleBytesRef<'a>> for SwapPayload {
    type Error = Infallible;

    fn encode<R: Region>(
        &self,
        local: SampleBytesRef<'a>,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>> {
        swap_words(local.payload, into)
    }

    fn decode<R: Region>(
        &self,
        wire: SampleBytesRef<'a>,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>> {
        swap_words(wire.payload, into)
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
        let local = SampleTypes {
            payload: description.clone(),
            user_header: description,
        };

        let remote = FakeSwapTranslator.remote(&local).expect("never fails");

        assert_that!(FakeSwapTranslator.local(&remote).expect("never fails"), eq local);
    }
}
