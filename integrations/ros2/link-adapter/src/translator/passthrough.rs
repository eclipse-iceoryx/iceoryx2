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
use iceoryx2::service::static_config::message_type_details::TypeVariant;
use iceoryx2_link_adapter::{
    NoTranscoder, PublishSubscribeTranslation, SampleTranscoders, SampleTranscodings, Transcoding,
    Translator,
};
use iceoryx2_link_backend::service_description::{SampleTypes, ServiceTypes, TypeDescription};
use iceoryx2_log::{fail, origin};

use super::{MirroredHeader, NoHeader, TranslationError, inbound_header};
use crate::config::TypeName;
use crate::endpoint_description::TopicTypes;

/// The translator for services whose payload is the CDR bytes of a ROS 2
/// message.
///
/// The payload type must be a byte slice whose type name is the ROS 2 message
/// type e.g. `geometry_msgs/msg/Twist`. Applications serialize and
/// deserialize the bytes themselves.
#[derive(Debug, Default, Clone, Copy)]
pub struct PassthroughTranslator {
    /// The user header of the services mirroring topics.
    pub header: MirroredHeader,
}

impl Translator for PassthroughTranslator {
    type EndpointTypes = TopicTypes;
    type Error = TranslationError;
    type Transcoder = SampleTranscoders<NoHeader, NoTranscoder>;

    fn local(&self, remote: &TopicTypes) -> Result<ServiceTypes, TranslationError> {
        Ok(ServiceTypes::PublishSubscribe(SampleTypes {
            payload: cdr_payload_type(remote.type_name.as_str()),
            user_header: self.header.type_description(),
        }))
    }

    fn remote(&self, local: &ServiceTypes) -> Result<TopicTypes, TranslationError> {
        let origin = origin!("PassthroughTranslator::remote");

        let types = local.publish_subscribe();

        // The payload's type name must be a ROS 2 message type.
        let type_name = fail!(
            from origin,
            when TypeName::new(&types.payload.type_name),
            with TranslationError::InvalidTypeName,
            "Payload type '{}' is not a ROS 2 type name", types.payload.type_name
        );

        // The payload must be a byte slice under that name, not a struct.
        if types.payload != cdr_payload_type(type_name.as_str()) {
            fail!(
                from origin,
                with TranslationError::LayoutMismatch,
                "Payload '{}' ({:?}, {} bytes, align {}) is not the byte slice carrying the CDR of ROS 2 type '{}'",
                types.payload.type_name, types.payload.variant, types.payload.size,
                types.payload.alignment, type_name.as_str()
            );
        }

        Ok(TopicTypes { type_name })
    }

    fn publish_subscribe(
        &self,
        local: &ServiceTypes,
        remote: &TopicTypes,
    ) -> Result<
        PublishSubscribeTranslation<SampleTranscoders<NoHeader, NoTranscoder>>,
        TranslationError,
    > {
        let origin = origin!("PassthroughTranslator::translation");

        let types = local.publish_subscribe();

        // The payload must be the byte slice with topic's message type name.
        if types.payload != cdr_payload_type(remote.type_name.as_str()) {
            fail!(
                from origin,
                with TranslationError::LayoutMismatch,
                "Payload '{}' is not the byte slice carrying the CDR of ROS 2 type '{}'",
                types.payload.type_name, remote.type_name.as_str()
            );
        }

        // If the service declares its header to be a `RosHeader`, populate
        // it. Otherwise, the `NoHeader` transcoder is used, which discards
        // the incoming header details.
        //
        // Other header types are unsupported.
        let inbound_header_transcoding = fail!(
            from origin,
            when inbound_header(types),
            "Header '{}' is not the RosHeader, ROS 2 cannot fill it", types.user_header.type_name
        );

        // The payload passes through, the outbound header is dropped.
        Ok(PublishSubscribeTranslation::Transcode {
            outbound: SampleTranscodings {
                header: Transcoding::Transcode,
                payload: Transcoding::Passthrough,
            },
            inbound: SampleTranscodings {
                header: inbound_header_transcoding,
                payload: Transcoding::Passthrough,
            },
            transcoder: SampleTranscoders {
                headers: NoHeader,
                payloads: NoTranscoder,
            },
        })
    }
}

/// The byte slice carrying the CDR of the ROS 2 type `type_name`.
fn cdr_payload_type(type_name: &str) -> TypeDescription {
    TypeDescription {
        variant: TypeVariant::Dynamic,
        type_name: type_name.to_string(),
        size: 1,
        alignment: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::service::static_config::message_type_details::TypeDetail;
    use iceoryx2_bb_testing::assert_that;

    use crate::ros_header::RosHeader;

    const TYPE_NAME: &str = "std_msgs/msg/String";

    fn topic() -> TopicTypes {
        TopicTypes {
            type_name: TypeName::new(TYPE_NAME).expect("a valid type name"),
        }
    }

    fn service(payload: TypeDescription, user_header: TypeDescription) -> ServiceTypes {
        ServiceTypes::PublishSubscribe(SampleTypes {
            payload,
            user_header,
        })
    }

    fn ros_header() -> TypeDescription {
        TypeDescription::from(&RosHeader::type_detail())
    }

    fn no_user_header() -> TypeDescription {
        TypeDescription::from(&TypeDetail::new::<()>(TypeVariant::FixedSize))
    }

    #[test]
    fn a_topic_is_mirrored_as_the_cdr_bytes_without_a_header() {
        let sut = PassthroughTranslator::default();

        let local = sut.local(&topic());

        assert_that!(local, eq Ok(service(cdr_payload_type(TYPE_NAME), no_user_header())));
    }

    #[test]
    fn a_topic_is_mirrored_under_the_message_info_when_configured() {
        let sut = PassthroughTranslator {
            header: MirroredHeader::RosHeader,
        };

        let local = sut.local(&topic());

        assert_that!(local, eq Ok(service(cdr_payload_type(TYPE_NAME), ros_header())));
    }

    #[test]
    fn a_byte_slice_named_as_a_ros_type_maps_to_its_topic() {
        let sut = PassthroughTranslator::default();

        let remote = sut.remote(&service(cdr_payload_type(TYPE_NAME), no_user_header()));

        assert_that!(remote, eq Ok(topic()));
    }

    #[test]
    fn a_payload_that_is_not_the_cdr_bytes_is_refused() {
        let sut = PassthroughTranslator::default();
        let payload = TypeDescription::from(&TypeDetail::new::<u64>(TypeVariant::FixedSize));
        let mut named = payload.clone();
        named.type_name = TYPE_NAME.to_string();

        let remote = sut.remote(&service(named, no_user_header()));

        assert_that!(remote, eq Err(TranslationError::LayoutMismatch));
    }

    #[test]
    fn the_payload_passes_through_and_the_header_is_encoded_to_nothing() {
        let sut = PassthroughTranslator::default();

        let translation = sut
            .publish_subscribe(
                &service(cdr_payload_type(TYPE_NAME), ros_header()),
                &topic(),
            )
            .expect("the service translates");

        let PublishSubscribeTranslation::Transcode {
            outbound, inbound, ..
        } = translation
        else {
            panic!("the header is transcoded");
        };
        assert_that!(outbound.header, eq Transcoding::Transcode);
        assert_that!(outbound.payload, eq Transcoding::Passthrough);
        assert_that!(inbound.header, eq Transcoding::Passthrough);
        assert_that!(inbound.payload, eq Transcoding::Passthrough);
    }

    #[test]
    fn a_service_without_a_header_has_it_transcoded_to_nothing_inbound() {
        let sut = PassthroughTranslator::default();

        let translation = sut
            .publish_subscribe(
                &service(cdr_payload_type(TYPE_NAME), no_user_header()),
                &topic(),
            )
            .expect("the service translates");

        let PublishSubscribeTranslation::Transcode { inbound, .. } = translation else {
            panic!("the header is transcoded");
        };
        assert_that!(inbound.header, eq Transcoding::Transcode);
    }

    #[test]
    fn a_service_with_another_header_is_refused() {
        let sut = PassthroughTranslator::default();
        let header = TypeDescription::from(&TypeDetail::new::<u64>(TypeVariant::FixedSize));

        let translation =
            sut.publish_subscribe(&service(cdr_payload_type(TYPE_NAME), header), &topic());

        assert_that!(translation.is_err(), eq true);
        assert_that!(
            translation.err(),
            eq Some(TranslationError::UnsupportedHeader)
        );
    }
}
