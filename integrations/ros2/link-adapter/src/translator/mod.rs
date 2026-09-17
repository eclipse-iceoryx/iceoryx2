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

//! The translators between local types and ROS 2 message types.
//!
//! ROS 2 has no user header. A mirrored topic gets the [`RosHeader`] to hold
//! the message info of each taken message. A local service must carry that
//! header if it requires that info, or none. No header is published to ROS 2.

pub mod passthrough;
pub mod plain_struct;

pub use passthrough::PassthroughTranslator;
pub use plain_struct::{CdrTranscoder, PlainStructTranslator, TranscodeFailure};

use iceoryx2_link_adapter::{
    HeaderTranscoder, Region, TranscodeError, Transcoding, WritableSample,
};
use iceoryx2_link_backend::service_description::{SampleTypes, TypeDescription};
use iceoryx2_log::{fail, origin};

use crate::ros_header::RosHeader;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TranslationError {
    /// The type's introspection typesupport could not be loaded.
    Introspection,
    /// The type's typesupport could not be loaded.
    TypeSupport,
    /// The type has dynamically sized or platform-defined members.
    UnsupportedType,
    /// The local payload is not the form the translator carries.
    LayoutMismatch,
    /// The local payload's type name is not a ROS 2 type name.
    InvalidTypeName,
    /// The local user header is neither the [`RosHeader`] nor absent.
    UnsupportedHeader,
}

impl core::fmt::Display for TranslationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TranslationError::{self:?}")
    }
}

impl core::error::Error for TranslationError {}

/// How the inbound header reaches a local service of `types`. The
/// [`RosHeader`] is taken as is since it is byte-accurate, no header is
/// transcoded to nothing, any other header is unsupported.
fn inbound_header(types: &SampleTypes) -> Result<Transcoding, TranslationError> {
    let origin = origin!("inbound_header");

    if types.user_header == TypeDescription::from(&RosHeader::type_detail()) {
        Ok(Transcoding::Passthrough)
    } else if types.user_header.size == 0 {
        Ok(Transcoding::Transcode)
    } else {
        fail!(
            from origin,
            with TranslationError::UnsupportedHeader,
            "Header '{}' is not the RosHeader, ROS 2 cannot fill it", types.user_header.type_name
        );
    }
}

/// The header transcoder of ROS 2. Transcodes to nothing in both direction as
/// ROS 2 carries no header.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoHeader;

impl HeaderTranscoder for NoHeader {
    type Failure = core::convert::Infallible;

    fn encode<R: Region>(
        &self,
        _: &[u8],
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Failure>> {
        let origin = origin!("NoHeader::encode");

        fail!(
            from origin,
            when no_header(into),
            "The header region rejected a length of 0"
        );
        Ok(())
    }

    fn decode<W: WritableSample>(
        &self,
        _: &[u8],
        into: &mut W,
    ) -> Result<(), TranscodeError<Self::Failure>> {
        let origin = origin!("NoHeader::decode");

        fail!(
            from origin,
            when into.header(0),
            to TranscodeError<Self::Failure>,
            "The sample's header rejected a length of 0"
        );
        Ok(())
    }
}

/// Transcodes a header to nothing.
fn no_header<R: Region, F>(into: &mut R) -> Result<(), TranscodeError<F>> {
    let origin = origin!("no_header");

    fail!(
        from origin,
        when into.for_length(0),
        to TranscodeError<F>,
        "The header region rejected a length of 0"
    );
    Ok(())
}
