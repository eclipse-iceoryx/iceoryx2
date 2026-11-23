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

use core::time::Duration;

use iceoryx2_bb_concurrency::atomic::{AtomicUsize, Ordering};
use iceoryx2_bb_elementary::generation::Generation;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_integrations_ros2_protocol_link_adapter_zenoh::codec::{
    EntityKind, NodeInfo, QosProfile, Token, TopicInfo,
};
use iceoryx2_integrations_ros2_protocol_link_adapter_zenoh::{
    Config, EndpointDescription, EndpointName, EndpointSettings, MessageType, OpenError,
    RmwZenohAdapter,
};
use iceoryx2_link_adapter::{Adapter, EndpointTypes};
use iceoryx2_link_conformance_tests::testing::retry;
use zenoh::liveliness::LivelinessToken;
use zenoh::{Session, Wait};

const TIMEOUT: Duration = Duration::from_secs(10);
const SCOUTING_DELAY_MS: u64 = 50;
const DOMAIN_IDS_PER_PROCESS: usize = 65_536;
const ZID_PREFIX: &str = "1ce";
const ZID_KEY: &str = "id";
const ENCLAVE: &str = "/";
const NAMESPACE: &str = "/";
const NODE_NAME: &str = "talker";
const NODE_ID: &str = "0";
const FIRST_ENTITY_ID: usize = 10;
const DEPTH: usize = 7;
const TWO_TOKENS: u64 = 2;
const THREE_TOKENS: u64 = 3;
const CHATTER: &str = "/chatter";
const STATUS: &str = "/status";
const ROSOUT: &str = "/rosout";
const ADD_TWO_INTS: &str = "/add_two_ints";
const GET_PARAMETERS: &str = "/talker/get_parameters";
const STRING_TYPE: &str = "std_msgs::msg::dds_::String_";
const STRING_ROS_TYPE: &str = "std_msgs/msg/String";
const STRING_HASH: &str = "RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18";
const INT_TYPE: &str = "std_msgs::msg::dds_::Int64_";
const LOG_TYPE: &str = "rcl_interfaces::msg::dds_::Log_";
const GET_PARAMETERS_TYPE: &str = "rcl_interfaces::srv::dds_::GetParameters_";
const NO_HASH: &str = "TypeHashNotSupported";
const ADD_TWO_INTS_TYPE: &str = "example_interfaces::srv::dds_::AddTwoInts_";
const ADD_TWO_INTS_REQUEST: &str = "example_interfaces/srv/AddTwoInts_Request";
const ADD_TWO_INTS_RESPONSE: &str = "example_interfaces/srv/AddTwoInts_Response";
const ADD_TWO_INTS_HASH: &str =
    "RIHS01_e118de6bf5eeb66a2491b5bda11202e7b68f198d6f67922cf30364858239c81a";
const PARAMETER_EVENTS: &str = "/parameter_events";
const PARAMETER_EVENT_TYPE: &str = "rcl_interfaces::msg::dds_::ParameterEvent_";
const GET_TYPE_DESCRIPTION: &str = "/talker/get_type_description";
const GET_TYPE_DESCRIPTION_TYPE: &str =
    "type_description_interfaces::srv::dds_::GetTypeDescription_";
const UNKNOWN_KIND: &str = "XX";

static NEXT_DOMAIN_OFFSET: AtomicUsize = AtomicUsize::new(0);

fn domain_id() -> usize {
    let process = usize::try_from(std::process::id()).expect("the process id fits");
    process * DOMAIN_IDS_PER_PROCESS + NEXT_DOMAIN_OFFSET.fetch_add(1, Ordering::Relaxed)
}

fn zenoh_config() -> zenoh::Config {
    let mut config = zenoh::Config::default();
    config
        .scouting
        .set_delay(Some(SCOUTING_DELAY_MS))
        .expect("the scouting delay is set");
    config
}

fn adapter(domain_id: usize, zenoh_config: zenoh::Config) -> RmwZenohAdapter {
    RmwZenohAdapter::create(
        zenoh_config,
        &Config {
            domain_id,
            ..Config::default()
        },
    )
    .expect("the adapter is created")
}

fn qos() -> QosProfile {
    QosProfile {
        depth: DEPTH,
        ..QosProfile::default()
    }
}

fn string_topic(name: &str) -> EndpointDescription {
    EndpointDescription {
        settings: EndpointSettings {
            name: EndpointName::Topic(name.to_string()),
            qos: qos(),
        },
        types: EndpointTypes::PublishSubscribe(MessageType {
            type_name: STRING_ROS_TYPE.to_string(),
            type_hash: STRING_HASH.to_string(),
        }),
    }
}

fn listed(adapter: &RmwZenohAdapter) -> Vec<EndpointDescription> {
    let mut descriptions = Vec::new();
    adapter
        .endpoints(&mut |description| descriptions.push(description.clone()))
        .expect("listing succeeds");
    descriptions
}

fn wait_for_listing(adapter: &RmwZenohAdapter, expected: &[EndpointDescription]) {
    retry(
        || match listed(adapter) == expected {
            true => Ok(()),
            false => Err("the listing differs"),
        },
        TIMEOUT,
    )
    .expect("the expected endpoints are listed");
}

fn wait_for_changes(adapter: &RmwZenohAdapter, changes: u64) {
    retry(
        || match adapter.generation() == Generation::At(changes) {
            true => Ok(()),
            false => Err("a change has not arrived"),
        },
        TIMEOUT,
    )
    .expect("every change arrives");
}

struct Emulator {
    session: Session,
    domain_id: usize,
    next_entity_id: usize,
}

impl Emulator {
    fn new(domain_id: usize) -> Self {
        Self {
            session: zenoh::open(zenoh_config())
                .wait()
                .expect("the emulator's session opens"),
            domain_id,
            next_entity_id: FIRST_ENTITY_ID,
        }
    }

    fn zid(&self) -> String {
        self.session.zid().to_string()
    }

    fn declare(
        &mut self,
        zid: &str,
        kind: EntityKind,
        name: &str,
        type_name: &str,
        type_hash: &str,
    ) -> LivelinessToken {
        let token = Token {
            zid: zid.to_string(),
            node_id: NODE_ID.to_string(),
            entity_id: self.next_entity_id.to_string(),
            kind,
            node: NodeInfo {
                domain_id: self.domain_id,
                enclave: ENCLAVE.to_string(),
                namespace: NAMESPACE.to_string(),
                name: NODE_NAME.to_string(),
            },
            topic: Some(TopicInfo {
                name: name.to_string(),
                type_name: type_name.to_string(),
                type_hash: type_hash.to_string(),
                qos: qos(),
            }),
        };
        self.next_entity_id += 1;
        self.session
            .liveliness()
            .declare_token(token.key())
            .wait()
            .expect("the token is declared")
    }
}

#[test]
fn remote_publishers_topic_is_listed() {
    let domain_id = domain_id();
    let adapter = adapter(domain_id, zenoh_config());
    let mut emulator = Emulator::new(domain_id);
    let zid = emulator.zid();

    let _publisher = emulator.declare(
        &zid,
        EntityKind::Publisher,
        CHATTER,
        STRING_TYPE,
        STRING_HASH,
    );

    wait_for_listing(&adapter, &[string_topic(CHATTER)]);
}

#[test]
fn remote_service_is_listed_with_request_and_response_types() {
    let domain_id = domain_id();
    let adapter = adapter(domain_id, zenoh_config());
    let mut emulator = Emulator::new(domain_id);
    let zid = emulator.zid();

    let _server = emulator.declare(
        &zid,
        EntityKind::ServiceServer,
        ADD_TWO_INTS,
        ADD_TWO_INTS_TYPE,
        ADD_TWO_INTS_HASH,
    );

    wait_for_listing(
        &adapter,
        &[EndpointDescription {
            settings: EndpointSettings {
                name: EndpointName::Service(ADD_TWO_INTS.to_string()),
                qos: qos(),
            },
            types: EndpointTypes::RequestResponse {
                request: MessageType {
                    type_name: ADD_TWO_INTS_REQUEST.to_string(),
                    type_hash: ADD_TWO_INTS_HASH.to_string(),
                },
                response: MessageType {
                    type_name: ADD_TWO_INTS_RESPONSE.to_string(),
                    type_hash: ADD_TWO_INTS_HASH.to_string(),
                },
            },
        }],
    );
}

#[test]
fn generation_moves_on_arrival_and_departure_and_holds_still_while_listing() {
    let domain_id = domain_id();
    let adapter = adapter(domain_id, zenoh_config());
    let mut emulator = Emulator::new(domain_id);
    let zid = emulator.zid();

    let initial = adapter.generation();
    listed(&adapter);
    assert_that!(adapter.generation(), eq initial);

    let publisher = emulator.declare(
        &zid,
        EntityKind::Publisher,
        CHATTER,
        STRING_TYPE,
        STRING_HASH,
    );
    retry(
        || match adapter.generation() != initial {
            true => Ok(()),
            false => Err("the generation has not moved"),
        },
        TIMEOUT,
    )
    .expect("the arrival moves the generation");

    let arrived = adapter.generation();
    listed(&adapter);
    assert_that!(adapter.generation(), eq arrived);

    drop(publisher);
    retry(
        || match adapter.generation() != arrived {
            true => Ok(()),
            false => Err("the generation has not moved"),
        },
        TIMEOUT,
    )
    .expect("the departure moves the generation");
}

#[test]
fn topic_that_left_is_not_listed() {
    let domain_id = domain_id();
    let adapter = adapter(domain_id, zenoh_config());
    let mut emulator = Emulator::new(domain_id);
    let zid = emulator.zid();

    let publisher = emulator.declare(
        &zid,
        EntityKind::Publisher,
        CHATTER,
        STRING_TYPE,
        STRING_HASH,
    );
    wait_for_listing(&adapter, &[string_topic(CHATTER)]);

    drop(publisher);
    wait_for_listing(&adapter, &[]);
}

#[test]
fn token_of_the_adapters_own_zid_is_not_listed() {
    let domain_id = domain_id();
    let own = format!("{ZID_PREFIX}{domain_id:x}");
    let mut config = zenoh_config();
    config
        .insert_json5(ZID_KEY, &format!("\"{own}\""))
        .expect("the zid is set");
    let adapter = adapter(domain_id, config);
    let mut emulator = Emulator::new(domain_id);
    let zid = emulator.zid();

    let _own = emulator.declare(
        &own,
        EntityKind::Publisher,
        STATUS,
        STRING_TYPE,
        STRING_HASH,
    );
    let _remote = emulator.declare(
        &zid,
        EntityKind::Publisher,
        CHATTER,
        STRING_TYPE,
        STRING_HASH,
    );
    wait_for_changes(&adapter, TWO_TOKENS);

    assert_that!(listed(&adapter), eq vec![string_topic(CHATTER)]);
}

#[test]
fn ros_internal_topics_and_services_are_not_listed() {
    let domain_id = domain_id();
    let adapter = adapter(domain_id, zenoh_config());
    let mut emulator = Emulator::new(domain_id);
    let zid = emulator.zid();

    let _rosout = emulator.declare(&zid, EntityKind::Publisher, ROSOUT, LOG_TYPE, NO_HASH);
    let _parameters = emulator.declare(
        &zid,
        EntityKind::ServiceServer,
        GET_PARAMETERS,
        GET_PARAMETERS_TYPE,
        NO_HASH,
    );
    let _chatter = emulator.declare(
        &zid,
        EntityKind::Publisher,
        CHATTER,
        STRING_TYPE,
        STRING_HASH,
    );
    wait_for_changes(&adapter, THREE_TOKENS);

    assert_that!(listed(&adapter), eq vec![string_topic(CHATTER)]);
}

#[test]
fn topic_with_two_types_is_not_listed() {
    let domain_id = domain_id();
    let adapter = adapter(domain_id, zenoh_config());
    let mut emulator = Emulator::new(domain_id);
    let zid = emulator.zid();

    let _string = emulator.declare(
        &zid,
        EntityKind::Publisher,
        CHATTER,
        STRING_TYPE,
        STRING_HASH,
    );
    let _int = emulator.declare(&zid, EntityKind::Publisher, CHATTER, INT_TYPE, NO_HASH);
    wait_for_changes(&adapter, TWO_TOKENS);

    assert_that!(listed(&adapter), len 0);
}

#[test]
fn two_entities_on_one_topic_list_it_once() {
    let domain_id = domain_id();
    let adapter = adapter(domain_id, zenoh_config());
    let mut emulator = Emulator::new(domain_id);
    let zid = emulator.zid();

    let _publisher = emulator.declare(
        &zid,
        EntityKind::Publisher,
        CHATTER,
        STRING_TYPE,
        STRING_HASH,
    );
    let _subscription = emulator.declare(
        &zid,
        EntityKind::Subscription,
        CHATTER,
        STRING_TYPE,
        STRING_HASH,
    );
    wait_for_changes(&adapter, TWO_TOKENS);

    assert_that!(listed(&adapter), eq vec![string_topic(CHATTER)]);
}

#[test]
fn parameter_events_topic_is_not_listed() {
    let domain_id = domain_id();
    let adapter = adapter(domain_id, zenoh_config());
    let mut emulator = Emulator::new(domain_id);
    let zid = emulator.zid();

    let _parameter_events = emulator.declare(
        &zid,
        EntityKind::Publisher,
        PARAMETER_EVENTS,
        PARAMETER_EVENT_TYPE,
        NO_HASH,
    );
    let _chatter = emulator.declare(
        &zid,
        EntityKind::Publisher,
        CHATTER,
        STRING_TYPE,
        STRING_HASH,
    );
    wait_for_changes(&adapter, TWO_TOKENS);

    assert_that!(listed(&adapter), eq vec![string_topic(CHATTER)]);
}

#[test]
fn type_description_service_is_not_listed() {
    let domain_id = domain_id();
    let adapter = adapter(domain_id, zenoh_config());
    let mut emulator = Emulator::new(domain_id);
    let zid = emulator.zid();

    let _type_description = emulator.declare(
        &zid,
        EntityKind::ServiceServer,
        GET_TYPE_DESCRIPTION,
        GET_TYPE_DESCRIPTION_TYPE,
        NO_HASH,
    );
    let _chatter = emulator.declare(
        &zid,
        EntityKind::Publisher,
        CHATTER,
        STRING_TYPE,
        STRING_HASH,
    );
    wait_for_changes(&adapter, TWO_TOKENS);

    assert_that!(listed(&adapter), eq vec![string_topic(CHATTER)]);
}

#[test]
fn topic_with_two_type_hashes_is_not_listed() {
    let domain_id = domain_id();
    let adapter = adapter(domain_id, zenoh_config());
    let mut emulator = Emulator::new(domain_id);
    let zid = emulator.zid();

    let _hashed = emulator.declare(
        &zid,
        EntityKind::Publisher,
        CHATTER,
        STRING_TYPE,
        STRING_HASH,
    );
    let _unhashed = emulator.declare(
        &zid,
        EntityKind::Subscription,
        CHATTER,
        STRING_TYPE,
        NO_HASH,
    );
    wait_for_changes(&adapter, TWO_TOKENS);

    assert_that!(listed(&adapter), len 0);
}

#[test]
fn endpoint_whose_type_name_is_no_rmw_name_is_not_listed() {
    let domain_id = domain_id();
    let adapter = adapter(domain_id, zenoh_config());
    let mut emulator = Emulator::new(domain_id);
    let zid = emulator.zid();

    let _status = emulator.declare(
        &zid,
        EntityKind::Publisher,
        STATUS,
        STRING_ROS_TYPE,
        STRING_HASH,
    );
    let _chatter = emulator.declare(
        &zid,
        EntityKind::Publisher,
        CHATTER,
        STRING_TYPE,
        STRING_HASH,
    );
    wait_for_changes(&adapter, TWO_TOKENS);

    assert_that!(listed(&adapter), eq vec![string_topic(CHATTER)]);
}

#[test]
fn malformed_token_leaves_the_generation_and_the_listing_unchanged() {
    let domain_id = domain_id();
    let adapter = adapter(domain_id, zenoh_config());
    let mut emulator = Emulator::new(domain_id);
    let zid = emulator.zid();

    let _chatter = emulator.declare(
        &zid,
        EntityKind::Publisher,
        CHATTER,
        STRING_TYPE,
        STRING_HASH,
    );
    wait_for_listing(&adapter, &[string_topic(CHATTER)]);

    let _malformed = emulator
        .session
        .liveliness()
        .declare_token(format!(
            "@ros2_lv/{domain_id}/{zid}/{NODE_ID}/{FIRST_ENTITY_ID}/{UNKNOWN_KIND}/%/%/{NODE_NAME}"
        ))
        .wait()
        .expect("the malformed token is declared");
    let _status = emulator.declare(
        &zid,
        EntityKind::Publisher,
        STATUS,
        STRING_TYPE,
        STRING_HASH,
    );
    wait_for_listing(&adapter, &[string_topic(CHATTER), string_topic(STATUS)]);

    wait_for_changes(&adapter, TWO_TOKENS);
}

#[test]
fn endpoints_of_every_pattern_are_unsupported() {
    let domain_id = domain_id();
    let mut adapter = adapter(domain_id, zenoh_config());
    let description = string_topic(CHATTER);

    assert_that!(adapter.publish_subscribe(&description).err(), eq Some(OpenError::UnsupportedPattern));
    assert_that!(adapter.request_response(&description).err(), eq Some(OpenError::UnsupportedPattern));
    assert_that!(adapter.event(&description).err(), eq Some(OpenError::UnsupportedPattern));
}
