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

#![allow(clippy::disallowed_types)]

use alloc::string::String;
use alloc::string::ToString;

use iceoryx2_bb_posix::file::*;
use iceoryx2_bb_posix::file_descriptor::*;
use iceoryx2_bb_posix::testing::create_test_directory;
use iceoryx2_bb_posix::testing::generate_file_path;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::test_requires;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_PERMISSIONS;
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_USERS_AND_GROUPS;

struct TestFixture {
    file: FilePath,
}

impl TestFixture {
    fn new() -> TestFixture {
        create_test_directory();
        let file = generate_file_path();
        File::remove(&file).ok();
        TestFixture { file }
    }

    fn file(&self) -> &FilePath {
        &self.file
    }

    fn create_file(&self, name: &FilePath) -> File {
        let file = FileBuilder::new(name)
            .creation_mode(CreationMode::PurgeAndCreate)
            .create();

        assert_that!(file, is_ok);
        file.unwrap()
    }

    fn open_file(&self, name: &FilePath) -> File {
        let file = FileBuilder::new(name).open_existing(AccessMode::ReadWrite);

        assert_that!(file, is_ok);
        file.unwrap()
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        File::remove(self.file()).expect("failed to cleanup test file");
    }
}

#[test]
pub fn opening_non_existing_file_fails() {
    let test = TestFixture::new();
    let result = FileBuilder::new(test.file()).open_existing(AccessMode::ReadWrite);

    assert_that!(result, is_err);
    assert_that!(result.err().unwrap(), eq FileOpenError::FileDoesNotExist);
}

#[test]
pub fn creating_non_existing_file_succeeds() {
    let test = TestFixture::new();
    let result = FileBuilder::new(test.file())
        .creation_mode(CreationMode::CreateExclusive)
        .create();

    assert_that!(result, is_ok);
}

#[test]
pub fn creating_existing_file_fails() {
    let test = TestFixture::new();

    test.create_file(test.file());

    let result = FileBuilder::new(test.file())
        .creation_mode(CreationMode::CreateExclusive)
        .create();

    assert_that!(result, is_err);
    assert_that!(result.err().unwrap(), eq FileCreationError::FileAlreadyExists);
}

#[test]
pub fn purge_and_create_non_existing_file_succeeds() {
    let test = TestFixture::new();

    let result = FileBuilder::new(test.file())
        .creation_mode(CreationMode::PurgeAndCreate)
        .create();

    assert_that!(result, is_ok);
}

#[test]
pub fn purge_and_create_existing_file_succeeds() {
    let test = TestFixture::new();
    test.create_file(test.file());

    let result = FileBuilder::new(test.file())
        .creation_mode(CreationMode::PurgeAndCreate)
        .create();

    assert_that!(result, is_ok);
}

#[test]
pub fn open_or_create_with_existing_file_succeeds() {
    let test = TestFixture::new();

    test.create_file(test.file());

    let result = FileBuilder::new(&test.file)
        .creation_mode(CreationMode::OpenOrCreate)
        .create();

    assert_that!(result, is_ok);
}

#[test]
pub fn open_or_create_with_non_existing_file_succeeds() {
    let test = TestFixture::new();

    let result = FileBuilder::new(&test.file)
        .creation_mode(CreationMode::OpenOrCreate)
        .create();

    assert_that!(result, is_ok);
}

#[test]
pub fn creating_file_applies_additional_settings() {
    test_requires!(POSIX_SUPPORT_PERMISSIONS && POSIX_SUPPORT_USERS_AND_GROUPS);

    let test = TestFixture::new();

    let file = FileBuilder::new(&test.file)
        .creation_mode(CreationMode::OpenOrCreate)
        .permission(Permission::OWNER_READ)
        .create();

    assert_that!(file, is_ok);

    let file = file.ok().unwrap();
    assert_that!(
        file.metadata().unwrap().permission(), eq
        Permission::OWNER_READ
    );
}

#[test]
pub fn simple_read_write_works() {
    let test = TestFixture::new();
    let mut file = test.create_file(&test.file);

    let mut content = "oh look what is in the file \n in in that line \t fuuu".to_string();
    let result = file.write(unsafe { content.as_mut_vec() }.as_slice());
    assert_that!(file.flush(), is_ok);

    assert_that!(result, is_ok);
    assert_that!(content, len result.ok().unwrap() as usize);

    let mut read_content = String::new();
    assert_that!(file.seek(0), is_ok);
    let result = file.read_to_string(&mut read_content);
    assert_that!(result, is_ok);
    assert_that!(content, len result.ok().unwrap() as usize);

    assert_that!(content, eq read_content);
}

#[test]
pub fn write_appends_content_to_file() {
    let test = TestFixture::new();
    let mut file = test.create_file(&test.file);

    assert_that!(file.write(b"another file bytes the dust\n"), is_ok);
    assert_that!(
        file.write(b"a horse with a blanket does not require shoes"),
        is_ok
    );
    assert_that!(file.flush(), is_ok);
    assert_that!(file.seek(0), is_ok);

    let mut read_content = String::new();
    let result = file.read_to_string(&mut read_content);
    assert_that!(result, is_ok);
    assert_that!(read_content.as_bytes(), eq b"another file bytes the dust\na horse with a blanket does not require shoes");
}

#[test]
pub fn multiple_read_calls_move_file_cursor() {
    let test = TestFixture::new();
    let mut file = test.create_file(&test.file);

    assert_that!(file.write(b"hakuna matata"), is_ok);
    assert_that!(file.flush(), is_ok);
    assert_that!(file.seek(0), is_ok);

    let mut buffer = [0u8; 1];
    assert_that!(file.read(&mut buffer), is_ok);
    assert_that!(buffer[0], eq b'h');

    assert_that!(file.read(&mut buffer), is_ok);
    assert_that!(buffer[0], eq b'a');

    assert_that!(file.read(&mut buffer), is_ok);
    assert_that!(buffer[0], eq b'k');
}

#[test]
pub fn read_line_works() {
    let test = TestFixture::new();
    let mut file = test.create_file(&test.file);

    assert_that!(
        file.write(b"whatever you do\nwherever you go\ndo not forget your towel!"),
        is_ok
    );
    assert_that!(file.flush(), is_ok);
    assert_that!(file.seek(0), is_ok);

    let mut buffer = String::new();
    assert_that!(file.read_line_to_string(&mut buffer), is_ok);
    assert_that!(buffer, eq "whatever you do");
    buffer.clear();

    assert_that!(file.read_line_to_string(&mut buffer), is_ok);
    assert_that!(buffer, eq "wherever you go");
    buffer.clear();

    assert_that!(file.read_line_to_string(&mut buffer), is_ok);
    assert_that!(buffer, eq "do not forget your towel!");
}

#[test]
pub fn two_file_objects_read_work_with_ranges_in_same_file() {
    let test = TestFixture::new();
    let mut file_a = test.create_file(&test.file);
    let mut file_b = test.open_file(&test.file);

    let mut content = "hello".to_string();
    let result = file_a.write(unsafe { content.as_mut_vec() }.as_slice());
    assert_that!(result, is_ok);
    assert_that!(content, len result.ok().unwrap() as usize);

    let mut content = "world".to_string();
    let result = file_b.write_at(2, unsafe { content.as_mut_vec() }.as_slice());
    assert_that!(result, is_ok);
    assert_that!(content, len result.ok().unwrap() as usize);

    let mut read_content = String::new();
    let result = file_a.read_range_to_string(1, 7, &mut read_content);
    assert_that!(result, is_ok);
    assert_that!(result.ok().unwrap(), eq 6);

    assert_that!("eworld", eq read_content);
}

#[test]
pub fn created_file_does_exist() -> Result<(), FileError> {
    let test = TestFixture::new();
    test.create_file(&test.file);

    assert_that!(File::does_exist(&test.file)?, eq true);
    Ok(())
}

#[test]
pub fn truncate_works() -> Result<(), FileError> {
    const NEW_SIZE: usize = 192;
    let test = TestFixture::new();
    let mut sut = test.create_file(&test.file);
    assert_that!(sut.truncate(NEW_SIZE), is_ok);
    assert_that!(sut.metadata().unwrap().size(), eq NEW_SIZE as _);

    Ok(())
}

#[test]
pub fn non_existing_file_does_not_exist() -> Result<(), FileError> {
    let test = TestFixture::new();

    assert_that!(!File::does_exist(&test.file)?, eq true);
    Ok(())
}

#[test]
pub fn remove_returns_true_when_file_exists() -> Result<(), FileError> {
    let test = TestFixture::new();
    test.create_file(&test.file);

    assert_that!(File::remove(&test.file)?, eq true);
    Ok(())
}

#[test]
pub fn remove_returns_false_when_file_not_exists() -> Result<(), FileError> {
    let test = TestFixture::new();

    assert_that!(!File::remove(&test.file)?, eq true);
    Ok(())
}

#[test]
pub fn newly_created_file_is_removed_when_it_has_ownership() -> Result<(), FileError> {
    create_test_directory();
    let file_name = generate_file_path();

    let file = FileBuilder::new(&file_name)
        .has_ownership(true)
        .creation_mode(CreationMode::OpenOrCreate)
        .create()
        .unwrap();

    assert_that!(File::does_exist(&file_name)?, eq true);
    drop(file);
    assert_that!(File::does_exist(&file_name)?, eq false);

    Ok(())
}

#[test]
pub fn newly_created_file_has_not_ownership_by_default() -> Result<(), FileError> {
    create_test_directory();
    let file_name = generate_file_path();

    let file = FileBuilder::new(&file_name)
        .creation_mode(CreationMode::OpenOrCreate)
        .create()
        .unwrap();

    assert_that!(File::does_exist(&file_name)?, eq true);
    drop(file);
    assert_that!(File::does_exist(&file_name)?, eq true);

    File::remove(&file_name).unwrap();

    Ok(())
}

#[test]
pub fn opened_file_is_removed_when_it_has_ownership() -> Result<(), FileError> {
    create_test_directory();
    let file_name = generate_file_path();

    FileBuilder::new(&file_name)
        .creation_mode(CreationMode::OpenOrCreate)
        .create()
        .unwrap();

    let file = FileBuilder::new(&file_name)
        .has_ownership(true)
        .open_existing(AccessMode::ReadWrite)
        .unwrap();

    assert_that!(File::does_exist(&file_name)?, eq true);
    drop(file);
    assert_that!(File::does_exist(&file_name)?, eq false);

    Ok(())
}

#[test]
pub fn opened_file_has_not_ownership_by_default() -> Result<(), FileError> {
    create_test_directory();
    let file_name = generate_file_path();

    FileBuilder::new(&file_name)
        .creation_mode(CreationMode::OpenOrCreate)
        .create()
        .unwrap();

    let file = FileBuilder::new(&file_name)
        .open_existing(AccessMode::ReadWrite)
        .unwrap();

    assert_that!(File::does_exist(&file_name)?, eq true);
    drop(file);
    assert_that!(File::does_exist(&file_name)?, eq true);

    File::remove(&file_name).unwrap();

    Ok(())
}

#[test]
pub fn acquire_ownership_works() -> Result<(), FileError> {
    create_test_directory();
    let file_name = generate_file_path();

    let file = FileBuilder::new(&file_name)
        .creation_mode(CreationMode::OpenOrCreate)
        .create()
        .unwrap();

    file.acquire_ownership();

    assert_that!(File::does_exist(&file_name)?, eq true);
    drop(file);
    assert_that!(File::does_exist(&file_name)?, eq false);

    Ok(())
}

#[test]
pub fn release_ownership_works() -> Result<(), FileError> {
    create_test_directory();
    let file_name = generate_file_path();

    let file = FileBuilder::new(&file_name)
        .has_ownership(true)
        .creation_mode(CreationMode::OpenOrCreate)
        .create()
        .unwrap();

    file.release_ownership();

    assert_that!(File::does_exist(&file_name)?, eq true);
    drop(file);
    assert_that!(File::does_exist(&file_name)?, eq true);

    File::remove(&file_name).unwrap();

    Ok(())
}
