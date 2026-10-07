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

use iceoryx2::name_generator::{default_name_generator::DefaultNameGenerator, *};
use iceoryx2::node::node_name::NodeName;
use iceoryx2::port::port_name::PortName;
use iceoryx2::testing::*;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
fn generate_returns_default_name() {
    let config = generate_isolated_config();
    let sut = DefaultNameGenerator::open_or_create(&config).unwrap();

    assert_that!(sut.generate_node_name(), eq NodeName::default());
    assert_that!(sut.generate_port_name(), eq PortName::default());
}
