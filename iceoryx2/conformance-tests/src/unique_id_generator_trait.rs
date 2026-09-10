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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod unique_id_generator_trait {
    use iceoryx2::config::Config;
    use iceoryx2::port::port_name::PortName;
    use iceoryx2::service::ipc;
    use iceoryx2::unique_id_generator::{Entity, UniqueIdGenerator};
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;

    #[conformance_test]
    pub fn generate_works_with_valid_arguments<Sut: UniqueIdGenerator>() {
        let sut = Sut::generate::<ipc::Service>(
            Entity::Client(PortName::new_empty()),
            Config::global_config(),
        );
        assert_that!(sut, is_ok);
    }

    #[conformance_test]
    pub fn generate_returns_unique_ids<Sut: UniqueIdGenerator>() {
        let config = Config::global_config();
        let sut1 =
            Sut::generate::<ipc::Service>(Entity::Client(PortName::new_empty()), config).unwrap();
        let sut2 =
            Sut::generate::<ipc::Service>(Entity::Client(PortName::new("Name").unwrap()), config)
                .unwrap();
        let sut3 =
            Sut::generate::<ipc::Service>(Entity::Server(PortName::new("Name").unwrap()), config)
                .unwrap();

        assert_that!(sut1, ne sut2);
        assert_that!(sut1, ne sut3);
        assert_that!(sut2, ne sut3);
    }
}
