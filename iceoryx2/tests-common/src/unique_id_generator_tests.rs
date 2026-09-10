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

use iceoryx2::config::Config;
use iceoryx2::port::port_name::PortName;
use iceoryx2::service::{self, ipc};
use iceoryx2::unique_id_generator::*;
use iceoryx2_bb_concurrency::atomic::{AtomicU64, Ordering};
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_bb_testing::assert_that;
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
impl UniqueIdGenerator for TestUniqueId {
    fn generate<Service: service::Service>(
        _entity: Entity,
        _config: &Config,
    ) -> Result<UniqueId, UniqueIdGeneratorGenerateError> {
        Ok(unsafe { UniqueId::from_raw_id(TestUniqueId::new().value() as u128) })
    }
}

#[test]
fn unique_id_can_be_created_from_value() {
    let value: u128 = 743243817103481069312485843209;
    let id = unsafe { UniqueId::from_raw_id(value) };
    assert_that!(id.value(), eq value);
}

#[test]
fn pid_returns_error_when_not_implemented() {
    let config = Config::global_config();
    let id = TestUniqueId::generate::<ipc::Service>(Entity::Client(PortName::new_empty()), config)
        .unwrap();
    let pid = TestUniqueId::pid(id);
    assert_that!(pid, is_err);
    assert_that!(pid.err().unwrap(), eq UniqueIdGeneratorDetailsError::NotImplemented);
}

#[test]
fn creation_time_returns_error_when_not_implemented() {
    let config = Config::global_config();
    let id = TestUniqueId::generate::<ipc::Service>(Entity::Client(PortName::new_empty()), config)
        .unwrap();
    let time = TestUniqueId::creation_time(id);
    assert_that!(time, is_err);
    assert_that!(time.err().unwrap(), eq UniqueIdGeneratorDetailsError::NotImplemented);
}

#[test]
fn unique_system_id_is_correctly_converted() {
    let unique_system_id = UniqueSystemId::new().unwrap();
    let unique_id = unsafe { UniqueId::from_raw_id(unique_system_id.value()) };
    assert_that!(unique_id.unique_value() >> 32, eq unique_system_id.counter() as u64);
    assert_that!(unique_id.unique_value() as u32, eq unique_system_id.pid().value() as u32);
    assert_that!(unique_id.payload_value() >> 32, eq unique_system_id.creation_time().nanoseconds() as u64);
    assert_that!(unique_id.payload_value() as u32, eq unique_system_id.creation_time().seconds() as u32);
}
