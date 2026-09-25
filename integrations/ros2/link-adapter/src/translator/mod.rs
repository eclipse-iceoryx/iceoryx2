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
//! ROS 2 has no user header. A mirrored topic gets the [`MirroredHeader`]
//! the translator is configured with, which is either empty or the
//! [`RosHeader`] holding the message info of each taken message.

pub mod passthrough;
pub mod plain_struct;

pub use passthrough::PassthroughTranslator;
pub use plain_struct::{CdrTranscoder, PlainStructTranslator, TranscodeFailure};

use core::convert::Infallible;

use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_link_adapter::{Region, SampleBytesRef, TranscodeError, Transcoder};
use iceoryx2_link_backend::service_description::{SampleTypes, TypeDescription};
use iceoryx2_log::{fail, origin};

use crate::ros_header::RosHeader;

/// The user header to use for the services mirroring topics.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum MirroredHeader {
    /// No user header.
    #[default]
    None,
    /// The [`RosHeader`] holding the message info.
    RosHeader,
}

/// The header transcoder of a service without a user header. ROS 2
/// carries no header, so both directions transcode to nothing.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct EmptyHeader;

impl<'a> Transcoder<SampleBytesRef<'a>> for EmptyHeader {
    type Error = Infallible;

    fn encode<R: Region>(
        &self,
        _: SampleBytesRef<'a>,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>> {
        let origin = origin!("EmptyHeader::encode");

        empty_header(origin, into)
    }

    fn decode<R: Region>(
        &self,
        _: SampleBytesRef<'a>,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>> {
        let origin = origin!("EmptyHeader::decode");

        empty_header(origin, into)
    }
}

/// Transcodes a header to nothing.
fn empty_header<R: Region>(origin: &str, into: &mut R) -> Result<(), TranscodeError<Infallible>> {
    fail!(
        from origin,
        when into.for_length(0),
        to TranscodeError<Infallible>,
        "The header region rejected a length of 0"
    );
    Ok(())
}

impl MirroredHeader {
    pub(crate) fn type_description(self) -> TypeDescription {
        match self {
            MirroredHeader::None => {
                TypeDescription::from(&TypeDetail::new::<()>(TypeVariant::FixedSize))
            }
            MirroredHeader::RosHeader => TypeDescription::from(&RosHeader::type_detail()),
        }
    }
}

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

/// The header of a local service of `types`.
fn mirrored_header(types: &SampleTypes) -> Result<MirroredHeader, TranslationError> {
    let origin = origin!("mirrored_header");

    if types.user_header == TypeDescription::from(&RosHeader::type_detail()) {
        Ok(MirroredHeader::RosHeader)
    } else if types.user_header.size == 0 {
        Ok(MirroredHeader::None)
    } else {
        fail!(
            from origin,
            with TranslationError::UnsupportedHeader,
            "Header '{}' is not the RosHeader, ROS 2 cannot fill it", types.user_header.type_name
        );
    }
}
