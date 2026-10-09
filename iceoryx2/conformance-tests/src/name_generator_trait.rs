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
pub mod name_generator_trait {
    use iceoryx2::config::Config;
    use iceoryx2::name_generator::NameGenerator;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_testing::generate_isolated_config;

    pub trait SutFactory {
        fn cleanup(config: &Config);
    }

    pub struct DefaultNameGeneratorTests {}
    impl SutFactory for DefaultNameGeneratorTests {
        fn cleanup(_: &Config) {}
    }

    #[conformance_test]
    pub fn open_or_create_returns_name_generator_for_valid_config<
        Sut: NameGenerator,
        Factory: SutFactory,
    >() {
        let config = generate_isolated_config();
        assert_that!(Sut::open_or_create(&config), is_ok);

        Factory::cleanup(&config);
    }

    #[conformance_test]
    pub fn calling_open_or_create_twice_succeeds<Sut: NameGenerator, Factory: SutFactory>() {
        let config = generate_isolated_config();
        let _ = Sut::open_or_create(&config).unwrap();
        assert_that!(Sut::open_or_create(&config), is_ok);

        Factory::cleanup(&config);
    }
}
