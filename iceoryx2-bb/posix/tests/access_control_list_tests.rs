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

use iceoryx2_bb_posix::access_control_list::*;
use iceoryx2_bb_posix::config::TEST_DIRECTORY;
use iceoryx2_bb_posix::directory::*;
use iceoryx2_bb_posix::file::*;
use iceoryx2_bb_posix::file_descriptor::FileDescriptorBased;
use iceoryx2_bb_posix::group::*;
use iceoryx2_bb_posix::user::*;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::test_requires;
use iceoryx2_pal_posix::*;

// TODO: [#40]
#[ignore]
#[test]
fn access_control_list_string_conversion_works() {
    test_requires!(posix::POSIX_SUPPORT_ACL);

    let mut sut = AccessControlList::new().unwrap();
    sut.add_user(0, AclPermission::Execute).unwrap();
    sut.add_group(0, AclPermission::WriteExecute).unwrap();

    let sut_string = sut.as_string().unwrap();
    let new_sut = AccessControlList::from_string(&sut_string).unwrap();

    assert_that!(sut.as_string().unwrap(), eq new_sut.as_string().unwrap());

    let entries = sut.get().unwrap();
    let new_entries = new_sut.get().unwrap();

    assert_that!(entries, len 6);
    let new_entries_len = new_entries.len();
    assert_that!(entries, len new_entries_len);

    for i in 0..6 {
        assert_that!(entries[i].id(), eq new_entries[i].id());
        assert_that!(entries[i].permission(), eq new_entries[i].permission());
        assert_that!(entries[i].tag(), eq new_entries[i].tag());
    }
}

#[test]
fn access_control_list_apply_to_file_works() {
    test_requires!(posix::POSIX_SUPPORT_ACL);

    Directory::create(&TEST_DIRECTORY, Permission::OWNER_ALL).unwrap();
    let file_path = FilePath::from_path_and_file(&TEST_DIRECTORY, unsafe {
        &FileName::new_unchecked(b"access_control_list_test")
    })
    .unwrap();

    let file = FileBuilder::new(&file_path)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let mut sut = AccessControlList::new().unwrap();
    sut.set(Acl::OwningUser, AclPermission::ReadExecute)
        .unwrap();
    sut.set(Acl::OwningGroup, AclPermission::Execute).unwrap();
    sut.set(Acl::Other, AclPermission::None).unwrap();
    sut.set(
        Acl::MaxAccessRightsForNonOwners,
        AclPermission::ReadWriteExecute,
    )
    .unwrap();

    // apply basic settings
    sut.apply_to_file_descriptor(unsafe { file.file_descriptor().native_handle() })
        .unwrap();

    //  // acquire acl from fd and extend it
    let mut sut =
        AccessControlList::from_file_descriptor(unsafe { file.file_descriptor().native_handle() })
            .unwrap();

    let testuser1_uid = "testuser1".as_user().unwrap().uid();
    let testuser2_uid = "testuser2".as_user().unwrap().uid();
    let testgroup1_gid = "testgroup1".as_group().unwrap().gid();
    let testgroup2_gid = "testgroup2".as_group().unwrap().gid();

    sut.add_user(testuser1_uid, AclPermission::Read).unwrap();
    sut.add_user(testuser2_uid, AclPermission::Write).unwrap();
    sut.add_group(testgroup1_gid, AclPermission::ReadWrite)
        .unwrap();
    sut.add_group(testgroup2_gid, AclPermission::WriteExecute)
        .unwrap();
    sut.apply_to_file_descriptor(unsafe { file.file_descriptor().native_handle() })
        .unwrap();

    let sut =
        AccessControlList::from_file_descriptor(unsafe { file.file_descriptor().native_handle() })
            .unwrap();
    let entries = sut.get().unwrap();

    for entry in entries {
        match entry.tag() {
            AclTag::OwningUser => {
                assert_that!(entry.permission(), eq AclPermission::ReadExecute)
            }
            AclTag::OwningGroup => {
                assert_that!(entry.permission(), eq AclPermission::Execute)
            }
            AclTag::Other => {
                assert_that!(entry.permission(), eq AclPermission::None)
            }
            AclTag::MaxAccessRightsForNonOwners => {
                assert_that!(entry.permission(), eq AclPermission::ReadWriteExecute)
            }
            AclTag::User => {
                if entry.id() == Some(testuser1_uid) {
                    assert_that!(entry.permission(), eq AclPermission::Read);
                } else if entry.id() == Some(testuser2_uid) {
                    assert_that!(entry.permission(), eq AclPermission::Write);
                } else {
                    assert_that!(true, eq false);
                }
            }
            AclTag::Group => {
                if entry.id() == Some(testgroup1_gid) {
                    assert_that!(entry.permission(), eq AclPermission::ReadWrite);
                } else if entry.id() == Some(testgroup2_gid) {
                    assert_that!(entry.permission(), eq AclPermission::WriteExecute);
                } else {
                    assert_that!(true, eq false);
                }
            }
            _ => {
                assert_that!(true, eq false);
            }
        }
    }
}
