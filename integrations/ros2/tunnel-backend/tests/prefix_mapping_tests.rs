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

use iceoryx2::service::local::Service;
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_integrations_ros2_tunnel_backend::{
    Durability, History, PrefixMapping, QosProfile, TopicDescription, TopicName, TypeName,
};
use iceoryx2_services_tunnel_backend::traits::Mapping;
use iceoryx2_services_tunnel_backend::types::service_description::{
    EventDescription, PatternDescription, PatternSettings, PublishSubscribeDescription,
    PublishSubscribeSettings, ServiceDescription, TypeDescription,
};

fn service_description_with_default_settings(name: &str, type_name: &str) -> ServiceDescription {
    ServiceDescription::new::<Service>(
        name.try_into().unwrap(),
        PatternDescription::PublishSubscribe(PublishSubscribeDescription {
            user_header: TypeDescription::from(&TypeDetail::new::<()>(TypeVariant::FixedSize)),
            payload: TypeDescription {
                variant: TypeVariant::Dynamic,
                type_name: type_name.to_string(),
                size: 1,
                alignment: 1,
            },
            settings: PatternSettings::UnknownApplyDefaults,
        }),
    )
}

fn service_description_with_settings(
    name: &str,
    type_name: &str,
    settings: PublishSubscribeSettings,
) -> ServiceDescription {
    ServiceDescription::new::<Service>(
        name.try_into().unwrap(),
        PatternDescription::PublishSubscribe(PublishSubscribeDescription {
            user_header: TypeDescription::from(&TypeDetail::new::<()>(TypeVariant::FixedSize)),
            payload: TypeDescription {
                variant: TypeVariant::Dynamic,
                type_name: type_name.to_string(),
                size: 1,
                alignment: 1,
            },
            settings: PatternSettings::Value(settings),
        }),
    )
}

#[test]
fn maps_prefixed_publish_subscribe_services() {
    let sut = PrefixMapping;
    let description =
        service_description_with_default_settings("ros2://topics/chatter", "std_msgs/msg/String");

    assert!(sut.remote(&description).is_some());
}

#[test]
fn ignores_unprefixed_names_and_invalid_type_names() {
    let sut = PrefixMapping;

    for (name, type_name) in [
        ("My/Funk/ServiceName", "std_msgs/msg/String"),
        ("ros2://topics", "std_msgs/msg/String"),
        ("ros2://topics/", "std_msgs/msg/String"),
        ("iox2://something", "std_msgs/msg/String"),
        ("/chatter", "std_msgs/msg/String"),
        ("ros2://topics/chatter", "NotARosTypeName"),
    ] {
        assert!(
            sut.remote(&service_description_with_default_settings(name, type_name))
                .is_none(),
            "{name}"
        );
    }
}

#[test]
fn ignores_event_services() {
    let sut = PrefixMapping;
    let description = ServiceDescription::new::<Service>(
        "ros2://topics/chatter".try_into().unwrap(),
        PatternDescription::Event(EventDescription {
            settings: PatternSettings::UnknownApplyDefaults,
        }),
    );

    assert!(sut.remote(&description).is_none());
}

#[test]
fn maps_service_description_to_topic_description() {
    let sut = PrefixMapping;
    let service_description = service_description_with_default_settings(
        "ros2://topics/Camera/FrontRight",
        "sensor_msgs/msg/Image",
    );

    let topic_description = sut
        .remote(&service_description)
        .expect("service maps to a topic");

    assert_eq!(topic_description.topic.as_str(), "/Camera/FrontRight");
    assert_eq!(
        topic_description.type_name.as_str(),
        "sensor_msgs/msg/Image"
    );

    assert_eq!(topic_description.qos, QosProfile::default());
}

#[test]
fn maps_history_setting_to_durability_qos() {
    const SUBSCRIBER_MAX_BUFFER_SIZE: usize = 4;
    const HISTORY_SIZE: usize = 2;

    let sut = PrefixMapping;
    let service_description = service_description_with_settings(
        "ros2://topics/chatter",
        "std_msgs/msg/String",
        PublishSubscribeSettings {
            max_subscribers: 8,
            max_publishers: 8,
            max_nodes: 8,
            history_size: HISTORY_SIZE,
            subscriber_max_buffer_size: SUBSCRIBER_MAX_BUFFER_SIZE,
            subscriber_max_borrowed_samples: 2,
            safe_overflow: true,
        },
    );

    let topic_description = sut
        .remote(&service_description)
        .expect("service maps to a topic");

    assert_eq!(
        topic_description.qos,
        QosProfile {
            history: History::KeepLast(SUBSCRIBER_MAX_BUFFER_SIZE),
            durability: Durability::TransientLocal,
            ..QosProfile::default()
        }
    );
}

#[test]
fn maps_non_overflowing_setting_to_keep_all_qos() {
    let sut = PrefixMapping;
    let service_description = service_description_with_settings(
        "ros2://topics/chatter",
        "std_msgs/msg/String",
        PublishSubscribeSettings {
            max_subscribers: 8,
            max_publishers: 8,
            max_nodes: 8,
            history_size: 0,
            subscriber_max_buffer_size: 4,
            subscriber_max_borrowed_samples: 2,
            safe_overflow: false,
        },
    );

    let topic_description = sut
        .remote(&service_description)
        .expect("service maps to a topic");

    assert_eq!(
        topic_description.qos,
        QosProfile {
            history: History::KeepAll,
            durability: Durability::Volatile,
            ..QosProfile::default()
        }
    );
}

#[test]
fn maps_durability_qos_to_history_setting() {
    const DEPTH: usize = 7;

    let sut = PrefixMapping;
    let topic_description = TopicDescription {
        topic: TopicName::new("/chatter").unwrap(),
        type_name: TypeName::new("std_msgs/msg/String").unwrap(),
        qos: QosProfile {
            history: History::KeepLast(DEPTH),
            durability: Durability::TransientLocal,
            ..QosProfile::default()
        },
    };

    let local = sut
        .local::<Service>(&topic_description)
        .expect("topic maps to a service");

    let PatternDescription::PublishSubscribe(description) = &local.pattern else {
        panic!("expected a publish-subscribe pattern description");
    };
    let PatternSettings::Value(settings) = &description.settings else {
        panic!("settings must be derived");
    };

    assert_eq!(settings.subscriber_max_buffer_size, DEPTH);
    assert_eq!(settings.history_size, DEPTH);
    assert!(settings.safe_overflow);
}

#[test]
fn maps_keep_all_qos_to_non_overflowing_setting() {
    let sut = PrefixMapping;
    let topic_description = TopicDescription {
        topic: TopicName::new("/chatter").unwrap(),
        type_name: TypeName::new("std_msgs/msg/String").unwrap(),
        qos: QosProfile {
            history: History::KeepAll,
            durability: Durability::Volatile,
            ..QosProfile::default()
        },
    };

    let local = sut
        .local::<Service>(&topic_description)
        .expect("topic maps to a service");

    let PatternDescription::PublishSubscribe(description) = &local.pattern else {
        panic!("expected a publish-subscribe pattern description");
    };
    let PatternSettings::Value(settings) = &description.settings else {
        panic!("settings must be derived");
    };

    assert!(!settings.safe_overflow);
}

#[test]
fn roundtrip_default_qos() {
    let sut = PrefixMapping;
    let topic_description = TopicDescription {
        topic: TopicName::new("/chatter").unwrap(),
        type_name: TypeName::new("std_msgs/msg/String").unwrap(),
        qos: QosProfile::default(),
    };

    let service_description = sut
        .local::<Service>(&topic_description)
        .expect("topic maps to a service");

    assert_eq!(service_description.name.as_str(), "ros2://topics/chatter");
    assert_eq!(
        sut.remote(&service_description)
            .expect("service maps to a topic"),
        topic_description
    );
}

#[test]
fn roundtrip_derived_qos() {
    let sut = PrefixMapping;

    for qos in [
        QosProfile {
            history: History::KeepLast(3),
            durability: Durability::TransientLocal,
            ..QosProfile::default()
        },
        QosProfile {
            history: History::KeepAll,
            ..QosProfile::default()
        },
    ] {
        let topic_description = TopicDescription {
            topic: TopicName::new("/chatter").unwrap(),
            type_name: TypeName::new("std_msgs/msg/String").unwrap(),
            qos,
        };

        let service_description = sut
            .local::<Service>(&topic_description)
            .expect("topic maps to a service");

        assert_eq!(
            sut.remote(&service_description)
                .expect("service maps to a topic"),
            topic_description
        );
    }
}
