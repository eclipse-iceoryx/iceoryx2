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

use super::token::TopicInfo;

pub(super) const LIVELINESS_ADMIN_SPACE: &str = "@ros2_lv";
pub(super) const KEY_SEPARATOR: &str = "/";
const WILDCARD: &str = "**";

pub fn topic_key(domain_id: usize, topic: &TopicInfo) -> String {
    let name = topic.name.as_str();
    let name = name.strip_prefix(KEY_SEPARATOR).unwrap_or(name);
    let name = name.strip_suffix(KEY_SEPARATOR).unwrap_or(name);
    format!(
        "{domain_id}{KEY_SEPARATOR}{name}{KEY_SEPARATOR}{}{KEY_SEPARATOR}{}",
        topic.type_name, topic.type_hash
    )
}

pub fn liveliness_subscription_key(domain_id: usize) -> String {
    format!("{LIVELINESS_ADMIN_SPACE}{KEY_SEPARATOR}{domain_id}{KEY_SEPARATOR}{WILDCARD}")
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::codec::QosProfile;

    const DOMAIN_ID: usize = 0;
    const OTHER_DOMAIN_ID: usize = 42;
    const STRING_TYPE: &str = "std_msgs::msg::dds_::String_";
    const STRING_HASH: &str =
        "RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18";

    fn string_topic(name: &str) -> TopicInfo {
        TopicInfo {
            name: name.to_string(),
            type_name: STRING_TYPE.to_string(),
            type_hash: STRING_HASH.to_string(),
            qos: QosProfile::default(),
        }
    }

    #[test]
    fn topic_key_matches_the_captured_key() {
        assert_eq!(
            topic_key(DOMAIN_ID, &string_topic("/chatter")),
            "0/chatter/std_msgs::msg::dds_::String_/RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18"
        );
    }

    #[test]
    fn service_key_matches_the_captured_key() {
        assert_eq!(
            topic_key(
                DOMAIN_ID,
                &TopicInfo {
                    name: "/add_two_ints".to_string(),
                    type_name: "example_interfaces::srv::dds_::AddTwoInts_".to_string(),
                    type_hash:
                        "RIHS01_e118de6bf5eeb66a2491b5bda11202e7b68f198d6f67922cf30364858239c81a"
                            .to_string(),
                    qos: QosProfile::default(),
                }
            ),
            "0/add_two_ints/example_interfaces::srv::dds_::AddTwoInts_/RIHS01_e118de6bf5eeb66a2491b5bda11202e7b68f198d6f67922cf30364858239c81a"
        );
    }

    #[test]
    fn topic_key_strips_one_leading_and_one_trailing_slash() {
        assert_eq!(
            topic_key(OTHER_DOMAIN_ID, &string_topic("//ns/chatter//")),
            format!("42//ns/chatter//{STRING_TYPE}/{STRING_HASH}")
        );
    }

    #[test]
    fn topic_key_keeps_a_name_without_slashes() {
        assert_eq!(
            topic_key(DOMAIN_ID, &string_topic("ns/chatter")),
            format!("0/ns/chatter/{STRING_TYPE}/{STRING_HASH}")
        );
    }

    #[test]
    fn liveliness_subscription_key_covers_the_domain() {
        assert_eq!(liveliness_subscription_key(DOMAIN_ID), "@ros2_lv/0/**");
        assert_eq!(
            liveliness_subscription_key(OTHER_DOMAIN_ID),
            "@ros2_lv/42/**"
        );
    }
}
