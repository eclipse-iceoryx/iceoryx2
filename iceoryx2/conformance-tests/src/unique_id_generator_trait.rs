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
    use iceoryx2::port::port_name::PortName;
    use iceoryx2::service::{self, ipc};
    use iceoryx2::unique_id_generator::{
        Entity, UniqueId, UniqueIdBuilder, UniqueIdGenerator, UniqueIdGeneratorError,
    };
    use iceoryx2_bb_concurrency::atomic::{AtomicU64, Ordering};
    use iceoryx2_bb_container::string::StaticString;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_bb_testing_macros::test;

    struct TestUniqueId {
        id: u64,
    }
    impl TestUniqueId {
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            Self {
                id: COUNTER.fetch_add(1, Ordering::Relaxed),
            }
        }
        fn value(&self) -> u64 {
            self.id
        }
    }
    impl From<UniqueId> for TestUniqueId {
        fn from(value: UniqueId) -> Self {
            Self {
                id: value.value() as u64,
            }
        }
    }
    impl UniqueIdGenerator for TestUniqueId {
        fn generate<Service: service::Service>(
            _builder: UniqueIdBuilder,
        ) -> Result<UniqueId, UniqueIdGeneratorError> {
            Ok(unsafe { UniqueId::from_value(TestUniqueId::new().value() as u128) })
        }
    }

    #[test]
    fn unique_id_can_be_created_from_value() {
        let value: u128 = 743243817103481069312485843209;
        let id = unsafe { UniqueId::from_value(value) };
        assert_that!(id.value(), eq value);
    }

    #[test]
    fn pid_returns_error_when_not_implemented() {
        let sut = TestUniqueId::new();
        let pid = sut.pid();
        assert_that!(pid, is_err);
        assert_that!(pid.err().unwrap(), eq UniqueIdGeneratorError::NotImplemented);
    }

    #[test]
    fn creation_time_returns_error_when_not_implemented() {
        let sut = TestUniqueId::new();
        let time = sut.creation_time();
        assert_that!(time, is_err);
        assert_that!(time.err().unwrap(), eq UniqueIdGeneratorError::NotImplemented);
    }

    #[conformance_test]
    pub fn generate_works_with_valid_arguments<Sut: UniqueIdGenerator>() {
        let sut =
            UniqueIdBuilder::new(Entity::Client(PortName::new_empty())).create::<ipc::Service>();
        assert_that!(sut, is_ok);
    }

    #[conformance_test]
    pub fn generate_returns_unique_ids<Sut: UniqueIdGenerator>() {
        let sut1 = UniqueIdBuilder::new(Entity::Client(PortName::new_empty()))
            .create::<ipc::Service>()
            .unwrap();
        let sut2 = UniqueIdBuilder::new(Entity::Client(PortName::new_empty()))
            .create::<ipc::Service>()
            .unwrap();
        let sut3 = UniqueIdBuilder::new(Entity::Client(PortName::new_empty()))
            .create::<ipc::Service>()
            .unwrap();

        assert_that!(sut1, ne sut2);
        assert_that!(sut1, ne sut3);
        assert_that!(sut2, ne sut3);
    }
}
