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

use iceoryx2_bb_elementary::math::ToB64;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_posix::{
    config::test_directory,
    directory::{Directory, DirectoryCreateError},
    file::Permission,
    unique_system_id::UniqueSystemId,
};
use iceoryx2_bb_system_types::file_name::*;

use crate::{
    config::Config,
    prelude::{NodeName, ServiceName},
    service::static_config::message_type_details::{TypeDetail, TypeNameString, TypeVariant},
};

pub fn generate_service_name() -> ServiceName {
    ServiceName::new(&format!("tests_{}", UniqueSystemId::new().unwrap().value())).unwrap()
}

pub fn generate_node_name() -> NodeName {
    NodeName::new(&format!("tests_{}", UniqueSystemId::new().unwrap().value())).unwrap()
}

pub fn generate_isolated_config() -> Config {
    match Directory::create(&test_directory(), Permission::OWNER_ALL) {
        Ok(_) | Err(DirectoryCreateError::DirectoryAlreadyExists) => (),
        Err(e) => fatal_panic!(
            "Failed to create test directory {} due to {:?}.",
            test_directory(),
            e
        ),
    };

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
    config.global.set_root_path(&test_directory());
    config.global.prefix = prefix;

    config
}

pub fn create_custom_type_detail(
    variant: TypeVariant,
    type_name: TypeNameString,
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

pub fn type_detail_set_name(v: &mut TypeDetail, value: TypeNameString) {
    v.type_name = value;
}

pub fn type_detail_set_variant(v: &mut TypeDetail, value: TypeVariant) {
    v.variant = value;
}
