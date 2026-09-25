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
    use iceoryx2_testing::generate_isolated_config;

    pub trait SutFactory {
        fn setup(entities: &[Entity], config: &Config);
        fn cleanup(config: &Config);
    }

    pub struct UniqueSystemIdTests {}
    impl SutFactory for UniqueSystemIdTests {
        fn setup(_: &[Entity], _: &Config) {}
        fn cleanup(_: &Config) {}
    }

    #[conformance_test]
    pub fn generate_works_with_valid_arguments<Sut: UniqueIdGenerator, Factory: SutFactory>() {
        let config = generate_isolated_config();
        let entities = [Entity::Client(PortName::new_empty())];
        Factory::setup(&entities, &config);

        let sut = Sut::generate::<ipc::Service>(&entities[0], &config);
        assert_that!(sut, is_ok);

        Factory::cleanup(&config);
    }

    #[conformance_test]
    pub fn generate_returns_unique_ids<Sut: UniqueIdGenerator, Factory: SutFactory>() {
        let config = generate_isolated_config();
        let entities = [
            Entity::Client(PortName::new_empty()),
            Entity::Client(PortName::new("Name").unwrap()),
            Entity::Server(PortName::new("Name").unwrap()),
        ];
        Factory::setup(&entities, &config);

        let sut1 = Sut::generate::<ipc::Service>(&entities[0], &config).unwrap();
        let sut2 = Sut::generate::<ipc::Service>(&entities[1], &config).unwrap();
        let sut3 = Sut::generate::<ipc::Service>(&entities[2], &config).unwrap();

        assert_that!(sut1.unique_value(), ne sut2.unique_value());
        assert_that!(sut1.unique_value(), ne sut3.unique_value());
        assert_that!(sut2.unique_value(), ne sut3.unique_value());

        Factory::cleanup(&config);
    }
}
