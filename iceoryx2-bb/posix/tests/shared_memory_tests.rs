// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_elementary::math::ToB64;
use iceoryx2_bb_posix::{shared_memory::*, unique_system_id::UniqueSystemId};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_testing::{assert_that, test_requires};
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_PERSISTENT_SHARED_MEMORY;

fn generate_shm_name() -> FileName {
    let mut file_name = FileName::new(b"shared_memory_tests_").unwrap();
    file_name
        .push_bytes(UniqueSystemId::new().unwrap().value().to_b64().as_bytes())
        .unwrap();
    file_name
}

#[test]
fn shared_memory_create_and_open_works() {
    let shm_name = generate_shm_name();
    let mut sut_create = SharedMemoryBuilder::new(&shm_name)
        .creation_mode(CreationMode::PurgeAndCreate)
        .size(1024)
        .permission(Permission::OWNER_ALL)
        .zero_memory(true)
        .create()
        .unwrap();

    let sut_open = SharedMemoryBuilder::new(&shm_name)
        .open_existing(AccessMode::Read)
        .unwrap();

    assert_that!(sut_create.size(), eq sut_open.size());
    assert_that!(sut_create.size(), ge 1024);

    assert_that!(sut_create.name(), eq sut_open.name());
    assert_that!(*sut_create.name(), eq shm_name);

    assert_that!(sut_create.base_address(), ne sut_open.base_address());

    for e in sut_create.as_mut_slice().iter_mut() {
        *e = 255;
    }

    for e in sut_open.as_slice().iter() {
        assert_that!(*e, eq 255);
    }
}

#[test]
fn shared_memory_create_and_modify_open_works() {
    let shm_name = generate_shm_name();
    let sut_create = SharedMemoryBuilder::new(&shm_name)
        .creation_mode(CreationMode::PurgeAndCreate)
        .size(1024)
        .permission(Permission::OWNER_ALL)
        .zero_memory(true)
        .create()
        .unwrap();

    let mut sut_open = SharedMemoryBuilder::new(&shm_name)
        .open_existing(AccessMode::ReadWrite)
        .unwrap();

    assert_that!(sut_create.size(), eq sut_open.size());
    assert_that!(sut_create.size(), ge 1024);

    assert_that!(sut_create.name(), eq sut_open.name());
    assert_that!(*sut_create.name(), eq shm_name);

    assert_that!(sut_create.base_address(), ne sut_open.base_address());

    for e in sut_open.as_mut_slice().iter_mut() {
        *e = 170;
    }

    for e in sut_create.as_slice().iter() {
        assert_that!(*e, eq 170);
    }
}

#[test]
fn shared_memory_opening_with_non_fitting_size_fails() {
    let shm_name = generate_shm_name();
    let sut_create = SharedMemoryBuilder::new(&shm_name)
        .creation_mode(CreationMode::PurgeAndCreate)
        .size(1024)
        .permission(Permission::OWNER_ALL)
        .zero_memory(true)
        .create()
        .unwrap();

    let sut_open1 = SharedMemoryBuilder::new(&shm_name)
        .creation_mode(CreationMode::OpenOrCreate)
        .size(sut_create.size() + 1)
        .permission(Permission::OWNER_ALL)
        .zero_memory(true)
        .create();

    let sut_open2 = SharedMemoryBuilder::new(&shm_name)
        .creation_mode(CreationMode::OpenOrCreate)
        .size(sut_create.size() * 2)
        .permission(Permission::OWNER_ALL)
        .zero_memory(true)
        .create();

    assert_that!(sut_open1, is_err);
    assert_that!(sut_open2, is_err);

    assert_that!(
        sut_open1.err().unwrap(), eq
        SharedMemoryCreationError::SizeDoesNotFit
    );
    assert_that!(
        sut_open2.err().unwrap(), eq
        SharedMemoryCreationError::SizeDoesNotFit
    );
}

#[test]
fn shared_memory_release_ownership_works() {
    test_requires!(POSIX_SUPPORT_PERSISTENT_SHARED_MEMORY);

    let shm_name = generate_shm_name();
    let mut sut_create = SharedMemoryBuilder::new(&shm_name)
        .creation_mode(CreationMode::PurgeAndCreate)
        .size(1024)
        .permission(Permission::OWNER_ALL)
        .zero_memory(true)
        .create()
        .unwrap();

    for e in sut_create.as_mut_slice().iter_mut() {
        *e = 170;
    }

    assert_that!(sut_create.has_ownership(), eq true);
    sut_create.release_ownership();
    assert_that!(sut_create.has_ownership(), eq false);
    drop(sut_create);

    let sut_open = SharedMemoryBuilder::new(&shm_name)
        .open_existing(AccessMode::ReadWrite)
        .unwrap();

    assert_that!(sut_open.size(), eq 1024);
    assert_that!(*sut_open.name(), eq shm_name);

    for e in sut_open.as_slice().iter() {
        assert_that!(*e, eq 170);
    }

    assert_that!(SharedMemory::remove(&shm_name), is_ok);
    drop(sut_open);

    let sut_open = SharedMemoryBuilder::new(&shm_name).open_existing(AccessMode::ReadWrite);
    assert_that!(sut_open, is_err);
    assert_that!(
        sut_open.err().unwrap(), eq
        SharedMemoryCreationError::DoesNotExist
    );
}

#[test]
fn shared_memory_create_without_ownership_works() {
    test_requires!(POSIX_SUPPORT_PERSISTENT_SHARED_MEMORY);

    let shm_name = generate_shm_name();
    let mut sut_create = SharedMemoryBuilder::new(&shm_name)
        .creation_mode(CreationMode::PurgeAndCreate)
        .size(1024)
        .permission(Permission::OWNER_ALL)
        .zero_memory(true)
        .has_ownership(false)
        .create()
        .unwrap();

    for e in sut_create.as_mut_slice().iter_mut() {
        *e = 170;
    }

    assert_that!(sut_create.has_ownership(), eq false);
    drop(sut_create);

    let sut_open = SharedMemoryBuilder::new(&shm_name)
        .open_existing(AccessMode::ReadWrite)
        .unwrap();

    assert_that!(sut_open.size(), eq 1024);
    assert_that!(*sut_open.name(), eq shm_name);

    for e in sut_open.as_slice().iter() {
        assert_that!(*e, eq 170);
    }

    assert_that!(SharedMemory::remove(&shm_name), is_ok);
    drop(sut_open);

    let sut_open = SharedMemoryBuilder::new(&shm_name).open_existing(AccessMode::ReadWrite);
    assert_that!(sut_open, is_err);
    assert_that!(
        sut_open.err().unwrap(), eq
        SharedMemoryCreationError::DoesNotExist
    );
}

#[test]
fn shared_memory_acquire_ownership_works() {
    test_requires!(POSIX_SUPPORT_PERSISTENT_SHARED_MEMORY);

    let shm_name = generate_shm_name();
    let sut_create = SharedMemoryBuilder::new(&shm_name)
        .creation_mode(CreationMode::PurgeAndCreate)
        .size(1024)
        .permission(Permission::OWNER_ALL)
        .zero_memory(true)
        .has_ownership(false)
        .create()
        .unwrap();

    sut_create.acquire_ownership();

    assert_that!(sut_create.has_ownership(), eq true);
    drop(sut_create);
    assert_that!(SharedMemory::does_exist(&shm_name), eq false);
}

#[test]
fn shared_memory_existing_shm_can_be_listed() {
    const NUMBER_OF_SHM: usize = 32;

    let mut shms = vec![];
    for _ in 0..NUMBER_OF_SHM {
        let shm_name = generate_shm_name();
        shms.push(
            SharedMemoryBuilder::new(&shm_name)
                .creation_mode(CreationMode::PurgeAndCreate)
                .size(1024)
                .permission(Permission::OWNER_ALL)
                .zero_memory(true)
                .create()
                .unwrap(),
        );
    }

    let shm_list = SharedMemory::list();

    assert_that!(shm_list.len(), ge NUMBER_OF_SHM);
    for shm in &shms {
        assert_that!(shm_list, contains * shm.name());
    }
}

#[test]
fn shared_memory_can_be_mapped_with_a_custom_offset() {
    const MAPPING_OFFSET: isize = 0; // only zero works reliably
    let shm_name = generate_shm_name();
    let sut = SharedMemoryBuilder::new(&shm_name)
        .mapping_offset(MAPPING_OFFSET)
        .creation_mode(CreationMode::PurgeAndCreate)
        .size(1024 * 1024)
        .create()
        .unwrap();

    assert_that!(sut.mapping_offset(), eq MAPPING_OFFSET);
}
