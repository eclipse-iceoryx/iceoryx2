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
use iceoryx2_bb_posix::file::{CreationMode, FileBuilder};
use iceoryx2_bb_posix::group::*;
use iceoryx2_bb_posix::metadata::Metadata;
use iceoryx2_bb_posix::ownership::*;
use iceoryx2_bb_posix::permission::*;
use iceoryx2_bb_posix::testing::generate_file_path;
use iceoryx2_bb_posix::user::*;
use iceoryx2_bb_system_types::group_name::GroupName;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::test_requires;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_USERS_AND_GROUPS;

#[test]
pub fn ownership_builder_works() {
    test_requires!(POSIX_SUPPORT_USERS_AND_GROUPS);

    let root = GroupName::new(b"root").unwrap();
    let wheel = GroupName::new(b"wheel").unwrap();

    let group = if let Ok(group) = Group::from_name(&root) {
        group
    } else if let Ok(group) = Group::from_name(&wheel) {
        group
    } else {
        unreachable!("Neither group 'root' not group 'wheel' is found!")
    };

    let ownership = OwnershipBuilder::new()
        .uid("root".as_user().expect("no such user").uid())
        .gid(group.gid())
        .create();

    assert_that!(ownership.uid().value(), eq 0);
    assert_that!(ownership.gid().value(), eq 0);
}

#[test]
pub fn set_ownership_on_non_existing_path_fails() {
    test_requires!(POSIX_SUPPORT_USERS_AND_GROUPS);

    let file_path = generate_file_path();
    let path = file_path.into();

    let me = User::from_self().unwrap();
    let uid = me.uid();
    let gid = me.details().unwrap().gid();

    let ownership = OwnershipBuilder::new().uid(uid).gid(gid).create();

    assert_that!(
        ownership.apply_to(&path),
        eq(Err(SetOwnershipError::DoesNotExist))
    );
}

#[test]
pub fn set_ownership_on_existings_path_works() {
    test_requires!(POSIX_SUPPORT_USERS_AND_GROUPS);

    let file_path = generate_file_path();

    let _file = FileBuilder::new(&file_path)
        .creation_mode(CreationMode::CreateExclusive)
        .permission(Permission::OWNER_READ_WRITE)
        .create()
        .expect("creating file exclusive works");

    let path = file_path.into();

    let me = User::from_self().unwrap();
    let uid = me.uid();
    let gid = me.details().unwrap().gid();

    let ownership = OwnershipBuilder::new().uid(uid).gid(gid).create();
    ownership.apply_to(&path).expect("setting ownership work");

    let metadata = Metadata::from_path(&path).unwrap();
    assert_that!(metadata.uid(), eq(uid));
    assert_that!(metadata.gid(), eq(gid));
}
