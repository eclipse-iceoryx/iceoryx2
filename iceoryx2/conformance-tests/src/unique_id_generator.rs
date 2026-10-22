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
pub mod unique_id_generator {
    use iceoryx2::config::Config;
    use iceoryx2::port::port_name::PortName;
    use iceoryx2::service::ipc;
    use iceoryx2::unique_id_generator::{Entity, UniqueIdGenerator};
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_testing::{generate_isolated_config, remove_global_mgmt_segment};

    pub trait SutFactory {
        fn new(entities: &[Entity], config: &Config) -> Self;
    }

    pub struct UniqueSystemIdTests {
        config: Config,
    }
    impl SutFactory for UniqueSystemIdTests {
        fn new(_: &[Entity], config: &Config) -> Self {
            Self {
                config: config.clone(),
            }
        }
    }
    impl Drop for UniqueSystemIdTests {
        fn drop(&mut self) {
            unsafe {
                remove_global_mgmt_segment::<ipc::Service>(&self.config)
                    .expect("Removed global management segment");
            }
        }
    }

    #[conformance_test]
    pub fn generate_works_with_valid_arguments<Sut: UniqueIdGenerator, Factory: SutFactory>() {
        let config = generate_isolated_config();
        let entities = [Entity::Client(PortName::new_empty())];
        let _factory = Factory::new(&entities, &config);

        let id_generator = Sut::open_or_create(&config).unwrap();
        let sut = id_generator.generate(&entities[0]);
        assert_that!(sut, is_ok);
    }

    #[conformance_test]
    pub fn generate_returns_unique_ids<Sut: UniqueIdGenerator, Factory: SutFactory>() {
        let config = generate_isolated_config();
        let entities = [
            Entity::Client(PortName::new_empty()),
            Entity::Client(PortName::new("Name").unwrap()),
            Entity::Server(PortName::new("Name").unwrap()),
        ];
        let _factory = Factory::new(&entities, &config);

        let id_generator = Sut::open_or_create(&config).unwrap();
        let sut1 = id_generator.generate(&entities[0]).unwrap();
        let sut2 = id_generator.generate(&entities[1]).unwrap();
        let sut3 = id_generator.generate(&entities[2]).unwrap();

        assert_that!(sut1.unique_value(), ne sut2.unique_value());
        assert_that!(sut1.unique_value(), ne sut3.unique_value());
        assert_that!(sut2.unique_value(), ne sut3.unique_value());
    }
}
