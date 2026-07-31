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

use core::alloc::Layout;

use iceoryx2::service::local::Service;
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_bb_testing::assert_that;
use iceoryx2_integrations_ros2_tunnel_backend::{
    PlainStructTranslator, QosProfile, TopicDescription, TopicName, TranslationError, TypeName,
};
use iceoryx2_services_tunnel_backend::traits::{
    PayloadLayout, Transcoder, Translation, Translator,
};
use iceoryx2_services_tunnel_backend::types::service_description::{
    PatternDescription, PortSettings, PublishSubscribeDescription, ServiceDescription,
    TypeDescription,
};

const TWIST_TYPE_NAME: &str = "geometry_msgs/msg/Twist";

fn service_description(payload: TypeDetail) -> ServiceDescription {
    ServiceDescription::new::<Service>(
        "translator-tests".try_into().expect("valid service name"),
        PatternDescription::PublishSubscribe(PublishSubscribeDescription {
            user_header: TypeDescription::from(&TypeDetail::new::<()>(TypeVariant::FixedSize)),
            payload: TypeDescription::from(&payload),
            settings: PortSettings::LocalDefaults,
        }),
    )
}

fn topic_description(type_name: &str) -> TopicDescription {
    TopicDescription {
        topic: TopicName::new("/translator_tests").expect("valid topic name"),
        type_name: TypeName::new(type_name).expect("valid type name"),
        qos: QosProfile::default(),
    }
}

#[test]
fn create_succeeds_for_fixed_size_types() {
    let translator = PlainStructTranslator;

    let translation = translator
        .create(
            &service_description(TypeDetail::new::<u8>(TypeVariant::Dynamic)),
            &topic_description(TWIST_TYPE_NAME),
        )
        .expect("fixed-size types resolve");

    let Translation::Transcode { payload_layout, .. } = translation else {
        panic!("fixed-size types resolve to translation");
    };
    assert_that!(
        payload_layout,
        eq PayloadLayout::FixedSize(Layout::from_size_align(48, 8).expect("valid layout"))
    );
}

#[test]
fn create_fails_for_dynamically_sized_types() {
    let translator = PlainStructTranslator;

    let result = translator.create(
        &service_description(TypeDetail::new::<u8>(TypeVariant::Dynamic)),
        &topic_description("std_msgs/msg/String"),
    );

    assert_that!(matches!(result, Err(TranslationError::UnsupportedType)), eq true);
}

#[test]
fn create_accepts_layout_compatible_fixed_size_payloads() {
    let translator = PlainStructTranslator;

    let result = translator.create(
        &service_description(TypeDetail::new::<[f64; 6]>(TypeVariant::FixedSize)),
        &topic_description(TWIST_TYPE_NAME),
    );

    assert_that!(result, is_ok);
}

#[test]
fn create_fails_on_layout_incompatible_fixed_size_payloads() {
    let translator = PlainStructTranslator;

    let result = translator.create(
        &service_description(TypeDetail::new::<u64>(TypeVariant::FixedSize)),
        &topic_description(TWIST_TYPE_NAME),
    );

    assert_that!(matches!(result, Err(TranslationError::LayoutMismatch)), eq true);
}

#[test]
fn twist_round_trips_through_the_wire() {
    let translator = PlainStructTranslator;
    let service = service_description(TypeDetail::new::<[f64; 6]>(TypeVariant::FixedSize));
    let endpoint = topic_description(TWIST_TYPE_NAME);

    let Translation::Transcode { transcoder, .. } = translator
        .create(&service, &endpoint)
        .expect("twist resolves")
    else {
        panic!("twist resolves to translation");
    };

    // Twist is six little-endian f64s, both natively and as CDR.
    let payload: Vec<u8> = (1..=6)
        .flat_map(|value| (value as f64).to_le_bytes())
        .collect();

    let mut wire = Vec::<u8>::new();
    let written = transcoder
        .to_wire(&payload, &mut wire)
        .expect("serialization succeeds");

    // Encapsulation header (4 bytes) + body.
    assert_that!(written, eq 4 + payload.len());
    assert_that!(&wire[4..written], eq & payload[..]);

    let mut roundtrip = Vec::<u8>::new();
    let read = transcoder
        .from_wire(&wire[..written], &mut roundtrip)
        .expect("deserialization succeeds");
    assert_that!(read, eq payload.len());
    assert_that!(&roundtrip[..read], eq & payload[..]);
}
