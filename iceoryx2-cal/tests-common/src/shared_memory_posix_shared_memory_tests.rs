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

use core::time::Duration;
use iceoryx2_bb_posix::clock::Time;
use iceoryx2_bb_posix::creation_mode::CreationMode;
use iceoryx2_bb_posix::permission::Permission;
use iceoryx2_bb_posix::testing::generate_file_path;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_cal::named_concept::*;
use iceoryx2_cal::shared_memory::*;
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;

const TIMEOUT: Duration = Duration::from_millis(100);

#[test]
fn waiting_for_initialization_works() {
    type Sut = iceoryx2_cal::shared_memory::posix::Memory<PoolAllocator>;
    let storage_name = generate_file_path().file_name();
    let file_name = <Sut as NamedConceptMgmt>::Configuration::default()
        .path_for(&storage_name)
        .file_name();

    let _raw_shm = iceoryx2_bb_posix::shared_memory::SharedMemoryBuilder::new(&file_name)
        .creation_mode(CreationMode::PurgeAndCreate)
        .size(1234)
        .has_ownership(true)
        .permission(Permission::OWNER_WRITE)
        .create()
        .unwrap();

    let start = Time::now().expect("failed to get current time");
    let sut = <Sut as SharedMemory<PoolAllocator>>::Builder::new(&storage_name)
        .timeout(TIMEOUT)
        .open();

    assert_that!(sut, is_err);
    assert_that!(sut.err().unwrap(), eq SharedMemoryOpenError::InitializationNotYetFinalized);
    assert_that!(start.elapsed().expect("failed to get elapsed time"), ge TIMEOUT);
}
