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

use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_posix::config::*;
use iceoryx2_bb_posix::creation_mode::*;
use iceoryx2_bb_posix::file::*;
use iceoryx2_bb_posix::file_descriptor::*;
use iceoryx2_bb_posix::file_type::*;
use iceoryx2_bb_posix::group::*;
use iceoryx2_bb_posix::testing::create_test_directory;
use iceoryx2_bb_posix::user::*;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::test_requires;
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_PERMISSIONS;
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_USERS_AND_GROUPS;

#[test]
fn metadata_reads_basic_stats_correctly() {
    create_test_directory();
    let file_name =
        FilePath::from_path_and_file(&test_directory(), &FileName::new(b"meta_test").unwrap())
            .unwrap();

    let mut file = FileBuilder::new(&file_name)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();
    file.write(b"hello world").unwrap();

    let sut = file.metadata().unwrap();

    assert_that!(sut.file_type(), eq FileType::File);
    assert_that!(sut.size(), eq 11);

    File::remove_self(file).unwrap();
}

#[test]
fn metadata_reads_owner_and_permission_stats_correctly() {
    test_requires!(POSIX_SUPPORT_USERS_AND_GROUPS && POSIX_SUPPORT_PERMISSIONS);

    create_test_directory();
    let file_name =
        FilePath::from_path_and_file(&test_directory(), &FileName::new(b"meta_test_123").unwrap())
            .unwrap();

    let mut file = FileBuilder::new(&file_name)
        .creation_mode(CreationMode::PurgeAndCreate)
        .permission(Permission::ALL)
        .owner(User::from_self().unwrap().uid())
        .group(Group::from_self().unwrap().gid())
        .create()
        .unwrap();
    file.write(b"hello world!!").unwrap();

    let sut = file.metadata().unwrap();

    assert_that!(sut.file_type(), eq FileType::File);
    assert_that!(sut.size(), eq 13);
    assert_that!(sut.permission(), eq Permission::ALL);
    assert_that!(sut.uid(), eq User::from_self().expect("").uid());
    assert_that!(sut.gid(), eq Group::from_self().expect("").gid());

    File::remove_self(file).unwrap();
}
