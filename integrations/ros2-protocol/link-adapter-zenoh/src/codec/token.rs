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

use super::keys::{KEY_SEPARATOR, LIVELINESS_ADMIN_SPACE};
use super::qos::QosProfile;

const NODE: &str = "NN";
const PUBLISHER: &str = "MP";
const SUBSCRIPTION: &str = "MS";
const SERVICE_SERVER: &str = "SS";
const SERVICE_CLIENT: &str = "SC";
const MANGLED_SEPARATOR: &str = "%";

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TokenError {
    TooFewParts,
    EmptyPart,
    WrongAdminSpace,
    InvalidDomainId,
    UnknownKind,
    InvalidQos,
}

impl core::fmt::Display for TokenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TokenError::{self:?}")
    }
}

impl core::error::Error for TokenError {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EntityKind {
    Node,
    Publisher,
    Subscription,
    ServiceServer,
    ServiceClient,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NodeInfo {
    pub domain_id: usize,
    pub enclave: String,
    pub namespace: String,
    pub name: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TopicInfo {
    pub name: String,
    pub type_name: String,
    pub type_hash: String,
    pub qos: QosProfile,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token {
    pub zid: String,
    pub node_id: String,
    pub entity_id: String,
    pub kind: EntityKind,
    pub node: NodeInfo,
    pub topic: Option<TopicInfo>,
}

fn mangle(name: &str) -> String {
    name.replace(KEY_SEPARATOR, MANGLED_SEPARATOR)
}

fn demangle(name: &str) -> String {
    name.replace(MANGLED_SEPARATOR, KEY_SEPARATOR)
}

impl Token {
    pub fn key(&self) -> String {
        let kind = match self.kind {
            EntityKind::Node => NODE,
            EntityKind::Publisher => PUBLISHER,
            EntityKind::Subscription => SUBSCRIPTION,
            EntityKind::ServiceServer => SERVICE_SERVER,
            EntityKind::ServiceClient => SERVICE_CLIENT,
        };
        let mut parts = vec![
            LIVELINESS_ADMIN_SPACE.to_string(),
            self.node.domain_id.to_string(),
            self.zid.clone(),
            self.node_id.clone(),
            self.entity_id.clone(),
            kind.to_string(),
            mangle(&self.node.enclave),
            mangle(&self.node.namespace),
            mangle(&self.node.name),
        ];
        if let Some(topic) = &self.topic {
            parts.extend([
                mangle(&topic.name),
                mangle(&topic.type_name),
                mangle(&topic.type_hash),
                topic.qos.to_string(),
            ]);
        }
        parts.join(KEY_SEPARATOR)
    }

    pub fn parse(key: &str) -> Result<Self, TokenError> {
        let parts: Vec<&str> = key.split(KEY_SEPARATOR).collect();
        let [
            admin_space,
            domain_id,
            zid,
            node_id,
            entity_id,
            kind,
            enclave,
            namespace,
            node_name,
            entity @ ..,
        ] = parts.as_slice()
        else {
            return Err(TokenError::TooFewParts);
        };
        if parts.iter().any(|part| part.is_empty()) {
            return Err(TokenError::EmptyPart);
        }
        if *admin_space != LIVELINESS_ADMIN_SPACE {
            return Err(TokenError::WrongAdminSpace);
        }
        let domain_id = domain_id.parse().map_err(|_| TokenError::InvalidDomainId)?;
        let kind = match *kind {
            NODE => EntityKind::Node,
            PUBLISHER => EntityKind::Publisher,
            SUBSCRIPTION => EntityKind::Subscription,
            SERVICE_SERVER => EntityKind::ServiceServer,
            SERVICE_CLIENT => EntityKind::ServiceClient,
            _ => return Err(TokenError::UnknownKind),
        };
        let topic = match kind {
            EntityKind::Node => None,
            _ => {
                let [name, type_name, type_hash, qos, ..] = entity else {
                    return Err(TokenError::TooFewParts);
                };
                Some(TopicInfo {
                    name: demangle(name),
                    type_name: demangle(type_name),
                    type_hash: demangle(type_hash),
                    qos: qos.parse().map_err(|_| TokenError::InvalidQos)?,
                })
            }
        };
        Ok(Self {
            zid: zid.to_string(),
            node_id: node_id.to_string(),
            entity_id: entity_id.to_string(),
            kind,
            node: NodeInfo {
                domain_id,
                enclave: demangle(enclave),
                namespace: demangle(namespace),
                name: demangle(node_name),
            },
            topic,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TALKER_NODE: &str = "@ros2_lv/0/6eeb7dddf6026d46786c8f11706568af/0/0/NN/%/%/talker";
    const TALKER_PUBLISHER: &str = "@ros2_lv/0/6eeb7dddf6026d46786c8f11706568af/0/11/MP/%/%/talker/%chatter/std_msgs::msg::dds_::String_/RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18/::,7:,:,:,,";
    const SERVICE_SERVER_TOKEN: &str = "@ros2_lv/0/14cad03f5fb80bfe4d25373a53747a71/0/11/SS/%/%/add_two_ints_server/%add_two_ints/example_interfaces::srv::dds_::AddTwoInts_/RIHS01_e118de6bf5eeb66a2491b5bda11202e7b68f198d6f67922cf30364858239c81a/::,10:,:,:,,";
    const SERVICE_CLIENT_TOKEN: &str = "@ros2_lv/0/2844a391ab633d3c31cbee149a07e5f4/0/11/SC/%/%/add_two_ints_client/%add_two_ints/example_interfaces::srv::dds_::AddTwoInts_/RIHS01_e118de6bf5eeb66a2491b5bda11202e7b68f198d6f67922cf30364858239c81a/::,10:,:,:,,";
    const HUMBLE_SUBSCRIPTION: &str = "@ros2_lv/0/7e5bf6e0bcb319450f1355b75766254f/0/10/MS/%/%/listener/%chatter/std_msgs::msg::dds_::String_/TypeHashNotSupported/::,10:,:,:,,";
    const SHORT_ZID_ROSOUT_PUBLISHER: &str = "@ros2_lv/0/f3b66aa49ebbcca499054ed4e0a67c/0/1/MP/%/%/add_two_ints_client/%rosout/rcl_interfaces::msg::dds_::Log_/RIHS01_e28ce254ca8abc06abf92773b74602cdbf116ed34fbaf294fb9f81da9f318eac/:1:,1000:,:10,0:,,";
    const BACKENDS: &str = "/backends:cuda:meta%3Bdata;cpu:";
    const DOMAIN_ID: usize = 0;
    const OTHER_DOMAIN_ID: usize = 7;
    const DEPTH_SEVEN: usize = 7;
    const DEPTH_TEN: usize = 10;

    fn talker_node() -> NodeInfo {
        NodeInfo {
            domain_id: DOMAIN_ID,
            enclave: "/".to_string(),
            namespace: "/".to_string(),
            name: "talker".to_string(),
        }
    }

    #[test]
    fn parses_a_node_token() {
        assert_eq!(
            Token::parse(TALKER_NODE),
            Ok(Token {
                zid: "6eeb7dddf6026d46786c8f11706568af".to_string(),
                node_id: "0".to_string(),
                entity_id: "0".to_string(),
                kind: EntityKind::Node,
                node: talker_node(),
                topic: None,
            })
        );
    }

    #[test]
    fn parses_a_publisher_token() {
        assert_eq!(
            Token::parse(TALKER_PUBLISHER),
            Ok(Token {
                zid: "6eeb7dddf6026d46786c8f11706568af".to_string(),
                node_id: "0".to_string(),
                entity_id: "11".to_string(),
                kind: EntityKind::Publisher,
                node: talker_node(),
                topic: Some(TopicInfo {
                    name: "/chatter".to_string(),
                    type_name: "std_msgs::msg::dds_::String_".to_string(),
                    type_hash:
                        "RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18"
                            .to_string(),
                    qos: QosProfile {
                        depth: DEPTH_SEVEN,
                        ..QosProfile::default()
                    },
                }),
            })
        );
    }

    #[test]
    fn parses_service_tokens() {
        let server = Token::parse(SERVICE_SERVER_TOKEN);
        let client = Token::parse(SERVICE_CLIENT_TOKEN);

        assert_eq!(
            server.as_ref().map(|t| t.kind),
            Ok(EntityKind::ServiceServer)
        );
        assert_eq!(
            client.as_ref().map(|t| t.kind),
            Ok(EntityKind::ServiceClient)
        );
        assert_eq!(
            server.map(|t| t.topic.map(|topic| topic.type_name)),
            Ok(Some(
                "example_interfaces::srv::dds_::AddTwoInts_".to_string()
            ))
        );
    }

    #[test]
    fn parses_a_humble_subscription_token() {
        let token = Token::parse(HUMBLE_SUBSCRIPTION);

        assert_eq!(token.as_ref().map(|t| t.kind), Ok(EntityKind::Subscription));
        assert_eq!(
            token.map(|t| t.topic),
            Ok(Some(TopicInfo {
                name: "/chatter".to_string(),
                type_name: "std_msgs::msg::dds_::String_".to_string(),
                type_hash: "TypeHashNotSupported".to_string(),
                qos: QosProfile {
                    depth: DEPTH_TEN,
                    ..QosProfile::default()
                },
            }))
        );
    }

    #[test]
    fn captured_tokens_round_trip() {
        assert_eq!(
            Token::parse(TALKER_NODE).map(|t| t.key()),
            Ok(TALKER_NODE.to_string())
        );
        assert_eq!(
            Token::parse(TALKER_PUBLISHER).map(|t| t.key()),
            Ok(TALKER_PUBLISHER.to_string())
        );
        assert_eq!(
            Token::parse(SERVICE_SERVER_TOKEN).map(|t| t.key()),
            Ok(SERVICE_SERVER_TOKEN.to_string())
        );
        assert_eq!(
            Token::parse(SERVICE_CLIENT_TOKEN).map(|t| t.key()),
            Ok(SERVICE_CLIENT_TOKEN.to_string())
        );
        assert_eq!(
            Token::parse(HUMBLE_SUBSCRIPTION).map(|t| t.key()),
            Ok(HUMBLE_SUBSCRIPTION.to_string())
        );
        assert_eq!(
            Token::parse(SHORT_ZID_ROSOUT_PUBLISHER).map(|t| t.key()),
            Ok(SHORT_ZID_ROSOUT_PUBLISHER.to_string())
        );
    }

    #[test]
    fn mangles_nested_names() {
        let token = Token {
            zid: "6eeb7dddf6026d46786c8f11706568af".to_string(),
            node_id: "1".to_string(),
            entity_id: "2".to_string(),
            kind: EntityKind::Subscription,
            node: NodeInfo {
                domain_id: OTHER_DOMAIN_ID,
                enclave: "/secure".to_string(),
                namespace: "/robot/arm".to_string(),
                name: "listener".to_string(),
            },
            topic: Some(TopicInfo {
                name: "/robot/arm/state".to_string(),
                type_name: "std_msgs::msg::dds_::String_".to_string(),
                type_hash: "TypeHashNotSupported".to_string(),
                qos: QosProfile::default(),
            }),
        };
        let key = "@ros2_lv/7/6eeb7dddf6026d46786c8f11706568af/1/2/MS/%secure/%robot%arm/listener/%robot%arm%state/std_msgs::msg::dds_::String_/TypeHashNotSupported/::,:,:,:,,";

        assert_eq!(token.key(), key);
        assert_eq!(Token::parse(key), Ok(token));
    }

    #[test]
    fn backends_part_is_ignored() {
        assert_eq!(
            Token::parse(&format!("{TALKER_PUBLISHER}{BACKENDS}")),
            Token::parse(TALKER_PUBLISHER)
        );
    }

    #[test]
    fn token_with_too_few_parts_is_rejected() {
        for input in [
            "",
            "@ros2_lv/0/6eeb7dddf6026d46786c8f11706568af/0/0/NN/%/%",
            "@ros2_lv/0/6eeb7dddf6026d46786c8f11706568af/0/11/MP/%/%/talker",
            "@ros2_lv/0/6eeb7dddf6026d46786c8f11706568af/0/11/MP/%/%/talker/%chatter/std_msgs::msg::dds_::String_/TypeHashNotSupported",
        ] {
            assert_eq!(Token::parse(input), Err(TokenError::TooFewParts), "{input}");
        }
    }

    #[test]
    fn node_token_with_more_parts_is_accepted() {
        assert_eq!(
            Token::parse(&format!("{TALKER_NODE}/%chatter")),
            Token::parse(TALKER_NODE)
        );
    }

    #[test]
    fn entity_token_with_more_parts_is_accepted() {
        assert_eq!(
            Token::parse(&format!("{TALKER_PUBLISHER}/cuda{BACKENDS}")),
            Token::parse(TALKER_PUBLISHER)
        );
    }

    #[test]
    fn token_with_empty_part_is_rejected() {
        for input in [
            "@ros2_lv/0//0/0/NN/%/%/talker",
            "@ros2_lv/0/6eeb7dddf6026d46786c8f11706568af/0/0/NN/%/%/talker/",
            &format!("{TALKER_PUBLISHER}/"),
        ] {
            assert_eq!(Token::parse(input), Err(TokenError::EmptyPart), "{input}");
        }
    }

    #[test]
    fn token_outside_the_admin_space_is_rejected() {
        assert_eq!(
            Token::parse("@ros_lv/0/6eeb7dddf6026d46786c8f11706568af/0/0/NN/%/%/talker"),
            Err(TokenError::WrongAdminSpace)
        );
    }

    #[test]
    fn token_with_non_numeric_domain_id_is_rejected() {
        for input in [
            "@ros2_lv/x/6eeb7dddf6026d46786c8f11706568af/0/0/NN/%/%/talker",
            "@ros2_lv/-1/6eeb7dddf6026d46786c8f11706568af/0/0/NN/%/%/talker",
        ] {
            assert_eq!(
                Token::parse(input),
                Err(TokenError::InvalidDomainId),
                "{input}"
            );
        }
    }

    #[test]
    fn token_with_unknown_kind_is_rejected() {
        assert_eq!(
            Token::parse("@ros2_lv/0/6eeb7dddf6026d46786c8f11706568af/0/0/XX/%/%/talker"),
            Err(TokenError::UnknownKind)
        );
    }

    #[test]
    fn token_with_invalid_qos_is_rejected() {
        assert_eq!(
            Token::parse(
                "@ros2_lv/0/6eeb7dddf6026d46786c8f11706568af/0/11/MP/%/%/talker/%chatter/std_msgs::msg::dds_::String_/TypeHashNotSupported/::,x:,:,:,,"
            ),
            Err(TokenError::InvalidQos)
        );
    }
}
