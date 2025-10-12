// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_posix::{
    config::test_directory,
    file::{CreationMode, FileBuilder},
    file_descriptor::FileDescriptorBased,
    memory_mapping::*,
    unique_system_id::UniqueSystemId,
};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_testing::assert_that;

fn generate_file_name() -> FilePath {
    let mut file = FileName::new(b"mmap_tests").unwrap();
    file.push_bytes(
        UniqueSystemId::new()
            .unwrap()
            .value()
            .to_string()
            .as_bytes(),
    )
    .unwrap();

    FilePath::from_path_and_file(&test_directory(), &file).unwrap()
}

#[test]
fn mapping_anonymous_memory_works() {
    const MEMORY_SIZE: usize = 8192;
    let mut sut = MemoryMappingBuilder::from_anonymous()
        .initial_mapping_permission(MappingPermission::ReadWrite)
        .size(MEMORY_SIZE)
        .create()
        .unwrap();

    for i in 0..MEMORY_SIZE {
        unsafe { sut.base_address_mut().add(i).write((i % 255) as u8) };
        assert_that!(unsafe { *sut.base_address_mut().add(i) }, eq(i % 255) as u8);
    }

    assert_that!(sut.base_address() as usize, eq sut.base_address_mut() as usize);
    assert_that!(sut.size(), eq MEMORY_SIZE);
    assert_that!(sut.file_descriptor(), is_none);
    assert_that!(sut.file_path(), is_none);
}

#[test]
fn setting_permission_to_read_works() {
    const MEMORY_SIZE: usize = 8192;
    let mut sut = MemoryMappingBuilder::from_anonymous()
        .initial_mapping_permission(MappingPermission::ReadWrite)
        .size(MEMORY_SIZE)
        .create()
        .unwrap();

    for i in 0..MEMORY_SIZE {
        unsafe { sut.base_address_mut().add(i).write((i % 255) as u8) };
        assert_that!(unsafe { *sut.base_address_mut().add(i) }, eq(i % 255) as u8);
    }

    sut.set_permission(0)
        .size(MEMORY_SIZE / 2)
        .apply(MappingPermission::Read)
        .unwrap();

    for i in 0..MEMORY_SIZE / 2 {
        assert_that!(unsafe { *sut.base_address_mut().add(i) }, eq(i % 255) as u8);
    }

    for i in MEMORY_SIZE / 2..MEMORY_SIZE {
        assert_that!(unsafe { *sut.base_address_mut().add(i) }, eq(i % 255) as u8);
        unsafe { sut.base_address_mut().add(i).write(0) };
        assert_that!(unsafe { *sut.base_address_mut().add(i) }, eq 0);
    }
}

#[test]
fn mapping_file_works() {
    const MEMORY_SIZE: usize = 8192;
    let file_path = generate_file_name();
    let mut file = FileBuilder::new(&file_path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    for i in 0..MEMORY_SIZE {
        file.write_at(i as _, &[(i % 255) as u8]).unwrap();
    }

    let mut sut = MemoryMappingBuilder::from_file(&file_path)
        .file_access_mode(AccessMode::ReadWrite)
        .mapping_behavior(MappingBehavior::Shared)
        .initial_mapping_permission(MappingPermission::ReadWrite)
        .size(MEMORY_SIZE)
        .create()
        .unwrap();

    for i in 0..MEMORY_SIZE {
        unsafe { sut.base_address_mut().add(i).write((i % 255) as u8) };
        assert_that!(unsafe { *sut.base_address_mut().add(i) }, eq(i % 255) as u8);
    }

    assert_that!(sut.base_address() as usize, eq sut.base_address_mut() as usize);
    assert_that!(sut.size(), eq MEMORY_SIZE);
    assert_that!(sut.file_descriptor(), is_some);
    assert_that!(*sut.file_path(), eq Some(file_path));
}

#[test]
fn mapping_file_descriptor_works() {
    const MEMORY_SIZE: usize = 8192;
    let file_path = generate_file_name();
    let mut file = FileBuilder::new(&file_path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    for i in 0..MEMORY_SIZE {
        file.write_at(i as _, &[(i % 123) as u8]).unwrap();
    }

    let fd = file.file_descriptor().clone();
    drop(file);

    let mut sut = MemoryMappingBuilder::from_file_descriptor(fd)
        .mapping_behavior(MappingBehavior::Shared)
        .initial_mapping_permission(MappingPermission::ReadWrite)
        .size(MEMORY_SIZE)
        .create()
        .unwrap();

    for i in 0..MEMORY_SIZE {
        unsafe { sut.base_address_mut().add(i).write((i % 123) as u8) };
        assert_that!(unsafe { *sut.base_address_mut().add(i) }, eq(i % 123) as u8);
    }

    assert_that!(sut.base_address() as usize, eq sut.base_address_mut() as usize);
    assert_that!(sut.size(), eq MEMORY_SIZE);
    assert_that!(sut.file_descriptor(), is_some);
    assert_that!(sut.file_path(), is_none);
}

#[test]
fn mapping_size_of_zero_fails() {
    let sut = MemoryMappingBuilder::from_anonymous()
        .initial_mapping_permission(MappingPermission::ReadWrite)
        .create();

    assert_that!(sut.err(), eq Some(MemoryMappingCreationError::MappingSizeIsZero));
}

#[test]
fn update_permissions_offset_fails_when_offset_is_not_multiple_of_page_size() {
    const MEMORY_SIZE: usize = 8192;
    let mut sut = MemoryMappingBuilder::from_anonymous()
        .initial_mapping_permission(MappingPermission::ReadWrite)
        .size(MEMORY_SIZE)
        .create()
        .unwrap();

    let result = sut
        .set_permission(123)
        .size(4096)
        .apply(MappingPermission::Read);

    assert_that!(result.err(), eq Some(MemoryMappingPermissionUpdateError::AddressOffsetNotAlignedToPageSize));
}

#[test]
fn update_permissions_offset_fails_when_size_is_not_multiple_of_page_size() {
    const MEMORY_SIZE: usize = 8192;
    let mut sut = MemoryMappingBuilder::from_anonymous()
        .initial_mapping_permission(MappingPermission::ReadWrite)
        .size(MEMORY_SIZE)
        .create()
        .unwrap();

    let result = sut
        .set_permission(0)
        .size(456)
        .apply(MappingPermission::Read);

    assert_that!(result.err(), eq Some(MemoryMappingPermissionUpdateError::SizeNotAlignedToPageSize));
}

#[test]
fn update_permissions_offset_fails_when_size_is_zero() {
    const MEMORY_SIZE: usize = 8192;
    let mut sut = MemoryMappingBuilder::from_anonymous()
        .initial_mapping_permission(MappingPermission::ReadWrite)
        .size(MEMORY_SIZE)
        .create()
        .unwrap();

    let result = sut.set_permission(0).apply(MappingPermission::Read);

    assert_that!(result.err(), eq Some(MemoryMappingPermissionUpdateError::SizeIsZero));
}

#[test]
fn update_permissions_offset_fails_when_range_is_greater_than_mapped_range() {
    const MEMORY_SIZE: usize = 8192;
    let mut sut = MemoryMappingBuilder::from_anonymous()
        .initial_mapping_permission(MappingPermission::ReadWrite)
        .size(MEMORY_SIZE)
        .create()
        .unwrap();

    let result = sut
        .set_permission(0)
        .size(MEMORY_SIZE * 2)
        .apply(MappingPermission::Read);

    assert_that!(result.err(), eq Some(MemoryMappingPermissionUpdateError::InvalidAddressRange));
}

#[test]
fn fails_when_it_is_not_mapped_to_address_hint() {
    const MEMORY_SIZE: usize = 8192;
    let sut = MemoryMappingBuilder::from_anonymous()
        .initial_mapping_permission(MappingPermission::ReadWrite)
        .mapping_address_hint(1)
        .enforce_mapping_address_hint(true)
        .size(MEMORY_SIZE)
        .create();

    assert_that!(sut.err(), eq Some(MemoryMappingCreationError::FailedToEnforceAddressHint));
}
