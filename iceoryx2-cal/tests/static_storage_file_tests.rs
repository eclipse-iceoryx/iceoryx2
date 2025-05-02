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

use core::time::Duration;
use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_posix::config::*;
use iceoryx2_bb_posix::directory::Directory;
use iceoryx2_bb_posix::file::*;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_cal::static_storage::file::*;
use iceoryx2_cal::testing::*;

#[test]
fn static_storage_file_custom_suffix_works() {
    let storage_name = generate_name();
    let config = generate_isolated_config::<Storage>()
        .suffix(unsafe { &FileName::new_unchecked(b".blubbme") });

    let content = "some storage content".to_string();

    let storage_guard = Builder::new(&storage_name)
        .config(&config)
        .create(content.as_bytes())
        .unwrap();
    assert_that!(*storage_guard.name(), eq storage_name);

    let storage_reader = Builder::new(&storage_name)
        .config(&config)
        .open(Duration::ZERO)
        .unwrap();
    assert_that!(*storage_reader.name(), eq storage_name);

    let content_len = content.len() as u64;
    assert_that!(storage_reader, len content_len);

    let mut read_content = String::from_utf8(vec![b' '; content.len()]).unwrap();
    storage_reader
        .read(unsafe { read_content.as_mut_vec() }.as_mut_slice())
        .unwrap();
    assert_that!(read_content, eq content);
}

#[test]
fn static_storage_file_path_is_created_when_it_does_not_exist() {
    let storage_name = generate_name();
    let config = generate_isolated_config::<Storage>();
    let content = "some more funky content".to_string();
    let non_existing_path = FilePath::from_path_and_file(&test_directory(), &generate_name())
        .unwrap()
        .clone();

    Directory::remove(&non_existing_path.clone().into()).ok();
    let config = config.path_hint(&non_existing_path.into());

    let storage_guard = Builder::new(&storage_name)
        .config(&config)
        .create(content.as_bytes());
    assert_that!(storage_guard, is_ok);

    let storage_reader = Builder::new(&storage_name)
        .config(&config)
        .open(Duration::ZERO)
        .unwrap();
    assert_that!(*storage_reader.name(), eq storage_name);

    let content_len = content.len() as u64;
    assert_that!(storage_reader, len content_len);

    let mut read_content = String::from_utf8(vec![b' '; content.len()]).unwrap();
    storage_reader
        .read(unsafe { read_content.as_mut_vec() }.as_mut_slice())
        .unwrap();
    assert_that!(read_content, eq content);
}

#[test]
fn static_storage_file_custom_path_and_suffix_list_storage_works() {
    const NUMBER_OF_STORAGES: u64 = 12;
    let config = generate_isolated_config::<Storage>()
        .suffix(unsafe { &FileName::new_unchecked(b".blubbme") })
        .path_hint(
            &FilePath::from_path_and_file(
                &test_directory(),
                &FileName::new(b"non_existing").unwrap(),
            )
            .unwrap()
            .into(),
        );

    let content = "some storage content".to_string();

    let mut storages = vec![];
    for _i in 0..NUMBER_OF_STORAGES {
        let storage_name = generate_name();
        storages.push(
            Builder::new(&storage_name)
                .config(&config)
                .create(content.as_bytes())
                .unwrap(),
        );
    }

    let mut some_files = vec![];
    for _i in 0..NUMBER_OF_STORAGES {
        let storage_name =
            FilePath::from_path_and_file(&test_directory(), &generate_name()).unwrap();
        FileBuilder::new(&storage_name)
            .creation_mode(CreationMode::PurgeAndCreate)
            .create()
            .unwrap();
        some_files.push(storage_name);
    }

    let contents = Storage::list_cfg(&config).unwrap();
    assert_that!(contents, len NUMBER_OF_STORAGES as usize);

    let contains = |s| {
        for entry in &storages {
            if *entry.name() == s {
                return true;
            }
        }
        false
    };

    for entry in contents {
        assert_that!(contains(entry), eq true);
    }

    for file in &some_files {
        File::remove(file).unwrap();
    }
}
