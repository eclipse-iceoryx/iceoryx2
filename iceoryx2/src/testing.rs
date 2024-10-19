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
use iceoryx2_bb_posix::{config::test_directory, unique_system_id::UniqueSystemId};
use iceoryx2_bb_system_types::file_name::*;

use crate::{
    config::Config,
    prelude::{NodeName, ServiceName},
};

pub fn generate_service_name() -> ServiceName {
    ServiceName::new(&format!("tests_{}", UniqueSystemId::new().unwrap().value())).unwrap()
}

pub fn generate_node_name() -> NodeName {
    NodeName::new(&format!("tests_{}", UniqueSystemId::new().unwrap().value())).unwrap()
}

pub fn generate_isolated_config() -> Config {
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
