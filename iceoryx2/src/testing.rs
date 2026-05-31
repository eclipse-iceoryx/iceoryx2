// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_elementary::math::ToB64;
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_bb_posix::{config::TEST_DIRECTORY, testing::*};
use iceoryx2_bb_system_types::file_name::*;
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::event::NamedConceptMgmt;
use iceoryx2_cal::named_concept::{NamedConceptDoesExistError, NamedConceptRemoveError};
use iceoryx2_cal::static_storage::StaticStorageCreateError;

use crate::identifiers::{UniqueNodeId, UniqueServiceId};
use crate::node::global_management_segment::GlobalManagementSegment;
use crate::node::{Node, NodeListFailure, NodeState};
use crate::prelude::MessagingPattern;
use crate::service::config_scheme::{
    dynamic_config_storage_config, port_tag_config, service_tag_config,
};
use crate::service::dynamic_config::DynamicConfig;
use crate::service::naming_scheme::dynamic_config_name;
use crate::service::service_hash::ServiceHash;
use crate::service::static_config;
use crate::{
    config::Config,
    prelude::{NodeName, ServiceName},
    service::static_config::message_type_details::{TypeDetail, TypeName, TypeVariant},
};
use iceoryx2_bb_container::string::String;

pub fn generate_service_name() -> ServiceName {
    ServiceName::new(&format!("tests_{}", UniqueSystemId::new().unwrap().value())).unwrap()
}

pub fn generate_node_name() -> NodeName {
    NodeName::new(&format!("tests_{}", UniqueSystemId::new().unwrap().value())).unwrap()
}

pub fn generate_isolated_config() -> Config {
    create_test_directory();

    let mut prefix = FileName::new(b"test_prefix_").unwrap();
    prefix
        .push_bytes(
            UniqueSystemId::new()
                .unwrap()
                .value()
                .to_b64()
                .to_lowercase()
                .as_bytes(),
        )
        .unwrap();

    let mut config = Config::default();
    config.global.set_root_path(&TEST_DIRECTORY);
    config.global.prefix = prefix;

    config
}

pub fn create_custom_type_detail(
    variant: TypeVariant,
    type_name: TypeName,
    size: usize,
    alignment: usize,
) -> TypeDetail {
    TypeDetail {
        variant,
        type_name,
        size,
        alignment,
    }
}

pub fn type_detail_set_size(v: &mut TypeDetail, value: usize) {
    v.size = value;
}

pub fn type_detail_set_alignment(v: &mut TypeDetail, value: usize) {
    v.alignment = value;
}

pub fn type_detail_set_name(v: &mut TypeDetail, value: TypeName) {
    v.type_name = value;
}

pub fn type_detail_set_variant(v: &mut TypeDetail, value: TypeVariant) {
    v.variant = value;
}

pub fn create_service_tag<S: crate::service::Service>(
    node: &Node<S>,
    service_hash: &ServiceHash,
) -> Result<Option<S::StaticStorage>, StaticStorageCreateError> {
    node.shared
        .create_service_tag("Testing", "Failed to create test service tag", service_hash)
}

pub fn does_service_tag_exist<S: crate::service::Service>(
    service_hash: &ServiceHash,
    config: &Config,
    node_id: &UniqueNodeId,
) -> Result<bool, NamedConceptDoesExistError> {
    <S::StaticStorage as NamedConceptMgmt>::does_exist_cfg(
        &service_hash.0.into(),
        &service_tag_config::<S>(config, node_id),
    )
}

pub fn create_port_tag<S: crate::service::Service>(
    node: &Node<S>,
    port_id: u128,
) -> Result<S::StaticStorage, StaticStorageCreateError> {
    node.shared
        .create_port_tag("Testing", "Failed to create test port tag", port_id)
}

pub fn does_port_tag_exist<S: crate::service::Service>(
    port_id: u128,
    config: &Config,
    node_id: &UniqueNodeId,
) -> Result<bool, NamedConceptDoesExistError> {
    let name = FileName::new(port_id.to_string().as_bytes())
        .expect("A number is always a valid file name.");

    <S::StaticStorage as NamedConceptMgmt>::does_exist_cfg(
        &name,
        &port_tag_config::<S>(config, node_id),
    )
}

pub fn get_node_state<S: crate::service::Service>(
    node_id: &UniqueNodeId,
    config: &Config,
) -> Result<Option<NodeState<S>>, NodeListFailure> {
    NodeState::new(node_id, config)
}

pub fn generate_service_hash<S: crate::service::Service>(
    service_name: &ServiceName,
    messaging_pattern: MessagingPattern,
) -> ServiceHash {
    ServiceHash::new::<S::ServiceNameHasher>(service_name, messaging_pattern)
}

/// # Safety
///
/// * It must be ensured that !NO! other process is running currently using the
///   same iceoryx2 domain.
///
pub unsafe fn remove_global_mgmt_segment<S: crate::service::Service>(
    config: &Config,
) -> Result<bool, NamedConceptRemoveError> {
    unsafe { GlobalManagementSegment::<S>::remove(config) }
}

pub fn do_blackboard_resources_exist<S: crate::service::Service>(
    config: &Config,
    service_id: UniqueServiceId,
    static_config: &static_config::blackboard::StaticConfig,
) -> bool {
    let blackboard_name = crate::service::naming_scheme::blackboard_name(service_id);
    let blackboard_payload_config =
        crate::service::config_scheme::blackboard_data_config::<S>(config);
    if <S::BlackboardPayload as NamedConceptMgmt>::does_exist_cfg(
        &blackboard_name,
        &blackboard_payload_config,
    )
    .unwrap()
    {
        return true;
    }

    let mut blackboard_mgmt_config =
        crate::service::config_scheme::blackboard_mgmt_config::<S, u64>(config);
    // Safe since the same type name is set when creating the BlackboardMgmt in
    // Creator::create_impl so we can safely remove the concept.
    unsafe {
        <S::BlackboardMgmt<u64> as DynamicStorage<u64>>::__internal_set_type_name_in_config(
            &mut blackboard_mgmt_config,
            static_config.type_details.type_name.as_str(),
        )
    };
    <S::BlackboardMgmt<u64> as NamedConceptMgmt>::does_exist_cfg(
        &blackboard_name,
        &blackboard_mgmt_config,
    )
    .unwrap()
}

pub unsafe fn remove_dynamic_config<S: crate::service::Service>(
    config: &Config,
    service_id: UniqueServiceId,
) {
    let segment_name = dynamic_config_name(service_id);
    let dyn_conf = dynamic_config_storage_config::<S>(config);

    unsafe {
        <S::DynamicStorage<DynamicConfig> as NamedConceptMgmt>::remove_cfg(&segment_name, &dyn_conf)
            .unwrap()
    };
}
