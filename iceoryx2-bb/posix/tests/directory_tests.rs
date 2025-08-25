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

use std::sync::Barrier;

use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_posix::config::*;
use iceoryx2_bb_posix::directory::*;
use iceoryx2_bb_posix::file::*;
use iceoryx2_bb_posix::file_descriptor::FileDescriptorBased;
use iceoryx2_bb_posix::file_type::*;
use iceoryx2_bb_posix::testing::create_test_directory;
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::test_fail;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_pal_configuration::PATH_SEPARATOR;

struct TestFixture {
    files: Vec<FilePath>,
    directories: Vec<Path>,
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        for file in &self.files {
            File::remove(file).expect("failed to cleanup test file");
        }

        for dir in self.directories.iter().rev() {
            Directory::remove(dir).expect("failed to cleanup test directory");
        }
    }
}

impl TestFixture {
    fn new() -> Self {
        Self {
            files: vec![],
            directories: vec![],
        }
    }
    fn create_test_file_at_path(&mut self, directory: &Path) -> File {
        let mut file = FileName::new(b"dir_tests_file").unwrap();
        file.push_bytes(
            UniqueSystemId::new()
                .unwrap()
                .value()
                .to_string()
                .as_bytes(),
        )
        .unwrap();

        let file = FilePath::from_path_and_file(directory, &file).unwrap();

        self.files.push(file.clone());

        FileBuilder::new(&file)
            .creation_mode(CreationMode::PurgeAndCreate)
            .create()
            .unwrap()
    }

    fn create_test_directory_at_path(&mut self, directory: &Path) -> Directory {
        let mut directory = directory.clone();
        let mut file = FileName::new(b"dir_tests_").unwrap();
        file.push_bytes(
            UniqueSystemId::new()
                .unwrap()
                .value()
                .to_string()
                .as_bytes(),
        )
        .unwrap();
        directory.add_path_entry(&file.into()).unwrap();

        self.directories.push(directory.clone());

        Directory::create(&directory, Permission::OWNER_ALL).unwrap()
    }

    fn generate_path_in_test_directory(&mut self) -> Path {
        let mut directory = test_directory();
        directory.push(PATH_SEPARATOR).unwrap();
        directory.push_bytes(b"dir_tests_").unwrap();
        directory
            .push_bytes(
                UniqueSystemId::new()
                    .unwrap()
                    .value()
                    .to_string()
                    .as_bytes(),
            )
            .unwrap();
        self.directories.push(directory.clone());

        directory
    }
}

#[test]
fn directory_test_directory_does_exist() {
    create_test_directory();
    assert_that!(Directory::does_exist(&test_directory()).unwrap(), eq true);
}

#[test]
fn directory_non_existing_directory_does_not_exist() {
    create_test_directory();
    let mut non_existant_path = Path::new(&test_directory()).unwrap();
    non_existant_path.push_bytes(b"i_do_not_exist").unwrap();
    assert_that!(!Directory::does_exist(&non_existant_path).unwrap(), eq true);
}

#[test]
fn directory_file_is_not_a_directory() {
    create_test_directory();

    let not_a_directory_entry = FileName::new(b"not_a_directory").unwrap();
    let not_a_directory_path =
        FilePath::from_path_and_file(&test_directory(), &not_a_directory_entry).unwrap();

    FileBuilder::new(&not_a_directory_path)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();
    assert_that!(Directory::does_exist(&not_a_directory_path.clone().into()).unwrap(), eq false);
    File::remove(&not_a_directory_path).unwrap();
}

#[test]
fn directory_create_from_path_works() {
    let mut test = TestFixture::new();

    create_test_directory();
    let sut_name = test.generate_path_in_test_directory();

    assert_that!(Directory::does_exist(&sut_name).unwrap(), eq false);
    let sut_create = Directory::create(&sut_name, Permission::OWNER_ALL);
    assert_that!(sut_create, is_ok);
    assert_that!(Directory::does_exist(&sut_name).unwrap(), eq true);
    assert_that!(unsafe { sut_create.unwrap().file_descriptor().native_handle() }, ge 0);
}

#[test]
fn directory_create_from_path_works_recursively() {
    let mut test = TestFixture::new();

    create_test_directory();
    let mut sut_name = test.generate_path_in_test_directory();
    sut_name
        .add_path_entry(&Path::new(b"all").unwrap())
        .unwrap();
    sut_name
        .add_path_entry(&Path::new(b"glory").unwrap())
        .unwrap();
    sut_name.add_path_entry(&Path::new(b"to").unwrap()).unwrap();
    sut_name
        .add_path_entry(&Path::new(b"the").unwrap())
        .unwrap();
    sut_name
        .add_path_entry(&Path::new(b"hypnotoad").unwrap())
        .unwrap();

    assert_that!(Directory::does_exist(&sut_name).unwrap(), eq false);
    let sut_create = Directory::create(&sut_name, Permission::OWNER_ALL);
    assert_that!(sut_create, is_ok);
    assert_that!(Directory::does_exist(&sut_name).unwrap(), eq true);
}

#[test]
fn directory_create_from_path_is_thread_safe() {
    const NUMBER_OF_THREADS: usize = 4;
    let _watchdog = Watchdog::new();
    let mut test = TestFixture::new();

    create_test_directory();
    let mut sut_name = test.generate_path_in_test_directory();
    sut_name
        .add_path_entry(&Path::new(b"all").unwrap())
        .unwrap();
    sut_name
        .add_path_entry(&Path::new(b"glory").unwrap())
        .unwrap();
    sut_name.add_path_entry(&Path::new(b"to").unwrap()).unwrap();
    sut_name
        .add_path_entry(&Path::new(b"the").unwrap())
        .unwrap();
    sut_name
        .add_path_entry(&Path::new(b"hypnotoad").unwrap())
        .unwrap();

    let barrier = Barrier::new(NUMBER_OF_THREADS + 1);
    std::thread::scope(|s| {
        for _ in 0..NUMBER_OF_THREADS {
            s.spawn(|| {
                barrier.wait();
                let sut_create = Directory::create(&sut_name, Permission::OWNER_ALL);
                assert_that!(sut_create, is_ok);
            });
        }

        assert_that!(Directory::does_exist(&sut_name).unwrap(), eq false);
        barrier.wait();
    });

    assert_that!(Directory::does_exist(&sut_name).unwrap(), eq true);
}

#[test]
fn directory_open_from_path_works() {
    let mut test = TestFixture::new();

    create_test_directory();
    let sut_name = test.generate_path_in_test_directory();

    Directory::create(&sut_name, Permission::OWNER_ALL).unwrap();

    let sut_open = Directory::new(&sut_name);
    assert_that!(sut_open, is_ok);
}

#[test]
fn directory_list_contents_works() {
    let mut test = TestFixture::new();

    create_test_directory();
    let sut_name = test.generate_path_in_test_directory();

    let sut = Directory::create(&sut_name, Permission::OWNER_ALL);
    assert_that!(sut, is_ok);
    let sut = sut.unwrap();

    let mut dir_vec = vec![];
    const NUMBER_OF_DIRECTORIES: usize = 10;
    for _i in 0..NUMBER_OF_DIRECTORIES {
        let dir = test.create_test_directory_at_path(sut.path());
        dir_vec.push(dir.path().to_string());
    }

    let mut file_vec = vec![];
    const NUMBER_OF_FILES: usize = 10;
    for _i in 0..NUMBER_OF_FILES {
        let file = test.create_test_file_at_path(sut.path());
        file_vec.push(file.path().unwrap().to_string());
    }

    let content = sut.contents().unwrap();
    assert_that!(content, len NUMBER_OF_DIRECTORIES + NUMBER_OF_FILES);

    let is_part_of_dir = |name: String| -> bool {
        for dir in &dir_vec {
            let separator = String::from_utf8_lossy(&[PATH_SEPARATOR; 1]);
            if *dir == sut.path().to_string() + &separator + &name {
                return true;
            }
        }
        false
    };

    let is_part_of_files = |name: String| -> bool {
        for file in &file_vec {
            let separator = String::from_utf8_lossy(&[PATH_SEPARATOR; 1]);
            if *file == sut.path().to_string() + &separator + &name {
                return true;
            }
        }
        false
    };

    for entry in content {
        match entry.metadata().file_type() {
            FileType::File => assert_that!(is_part_of_files(entry.name().to_string()), eq true),
            FileType::Directory => assert_that!(is_part_of_dir(entry.name().to_string()), eq true),
            _ => test_fail!("The directory shall only contain files and directories."),
        }
    }
}
