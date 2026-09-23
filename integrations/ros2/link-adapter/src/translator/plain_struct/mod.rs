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

mod cdr_transcoder;
mod layout;

pub use cdr_transcoder::{CdrTranscoder, TranscodeFailure};
use layout::layout_of;

use iceoryx2::service::static_config::message_type_details::TypeVariant;
use iceoryx2_link_adapter::{
    PublishSubscribeTranslation, SampleTranscoders, SampleTranscodings, Transcoding, Translator,
};
use iceoryx2_link_backend::service_description::{SampleTypes, ServiceTypes, TypeDescription};
use iceoryx2_log::{fail, origin};

use super::{MirroredHeader, NoHeader, TranslationError, inbound_header};
use crate::config::TypeName;
use crate::endpoint_description::TopicTypes;
use crate::typesupport;

/// The translator for services whose payload is the C struct rosidl
/// generates for a ROS 2 message, serialized to CDR on the way out and
/// deserialized on the way in.
///
/// The payload type must be named after the ROS 2 message type, such as
/// `geometry_msgs/msg/Twist`, and must match the struct's size and
/// alignment. The message must be a plain struct. Strings, sequences and
/// other dynamically sized members are refused.
#[derive(Debug, Default, Clone, Copy)]
pub struct PlainStructTranslator {
    /// The user header of the services mirroring topics.
    pub header: MirroredHeader,
}

impl Translator for PlainStructTranslator {
    type EndpointTypes = TopicTypes;
    type Error = TranslationError;
    type Transcoder = SampleTranscoders<NoHeader, CdrTranscoder>;

    fn local(&self, remote: &TopicTypes) -> Result<ServiceTypes, Self::Error> {
        let origin = origin!("PlainStructTranslator::local");

        let layout = fail!(
            from origin,
            when layout_of(remote.type_name.as_str()),
            "Failed to lay out ROS 2 type '{}'", remote.type_name.as_str()
        );

        Ok(ServiceTypes::PublishSubscribe(SampleTypes {
            payload: TypeDescription {
                variant: TypeVariant::FixedSize,
                type_name: remote.type_name.as_str().to_string(),
                size: layout.size(),
                alignment: layout.align(),
            },
            user_header: self.header.type_description(),
        }))
    }

    fn remote(&self, local: &ServiceTypes) -> Result<TopicTypes, Self::Error> {
        let origin = origin!("PlainStructTranslator::remote");
        let types = local.publish_subscribe();

        let type_name = fail!(
            from origin,
            when TypeName::new(&types.payload.type_name),
            with TranslationError::InvalidTypeName,
            "Payload type '{}' is not a ROS 2 type name", types.payload.type_name
        );

        Ok(TopicTypes { type_name })
    }

    fn publish_subscribe(
        &self,
        local: &ServiceTypes,
        remote: &TopicTypes,
    ) -> Result<PublishSubscribeTranslation<SampleTranscoders<NoHeader, CdrTranscoder>>, Self::Error>
    {
        let origin = origin!("PlainStructTranslator::translation");
        let types = local.publish_subscribe();

        // The local payload must be the type's C struct.
        let type_name = remote.type_name.as_str();
        let layout = fail!(
            from origin,
            when layout_of(type_name),
            "Failed to lay out ROS 2 type '{}'", type_name
        );
        if types.payload.variant != TypeVariant::FixedSize
            || types.payload.size != layout.size()
            || types.payload.alignment != layout.align()
        {
            fail!(
                from origin,
                with TranslationError::LayoutMismatch,
                "Payload '{}' ({} bytes, align {}) is not the C struct of ROS 2 type '{}' ({} bytes, align {})",
                types.payload.type_name, types.payload.size, types.payload.alignment,
                type_name, layout.size(), layout.align()
            );
        }

        // Typesupport to use for (de)serialization.
        let type_support = fail!(
            from origin,
            when typesupport::load(type_name),
            with TranslationError::TypeSupport,
            "Failed to load typesupport for type '{}'", type_name
        );

        let inbound_header_transcoding = fail!(
            from origin,
            when inbound_header(types),
            "Header '{}' is not the RosHeader, ROS 2 cannot fill it", types.user_header.type_name
        );

        // The payload is CDR on the wire in both directions, the outbound header is encoded to nothing.
        Ok(PublishSubscribeTranslation::Transcode {
            outbound: SampleTranscodings {
                header: Transcoding::Transcode,
                payload: Transcoding::Transcode,
            },
            inbound: SampleTranscodings {
                header: inbound_header_transcoding,
                payload: Transcoding::Transcode,
            },
            transcoder: SampleTranscoders {
                headers: NoHeader,
                payloads: CdrTranscoder {
                    type_name: type_name.to_string(),
                    type_support,
                    layout,
                },
            },
        })
    }
}
