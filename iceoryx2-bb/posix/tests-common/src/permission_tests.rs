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

use iceoryx2_bb_posix::file::{CreationMode, FileBuilder};
use iceoryx2_bb_posix::metadata::Metadata;
use iceoryx2_bb_posix::permission::*;
use iceoryx2_bb_posix::testing::generate_file_path;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::test_requires;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_PERMISSIONS;

#[test]
pub fn setting_and_reading_works() {
    let mut v1 = Permission::OWNER_READ
        | Permission::OTHERS_WRITE
        | Permission::GROUP_EXEC
        | Permission::GROUP_READ;
    v1 |= Permission::SET_GID;

    assert_that!(v1.has(Permission::OWNER_READ), eq true);
    assert_that!(v1.has(Permission::OWNER_WRITE), eq false);
    assert_that!(v1.has(Permission::OWNER_EXEC), eq false);
    assert_that!(v1.has(Permission::GROUP_READ), eq true);
    assert_that!(v1.has(Permission::GROUP_WRITE), eq false);
    assert_that!(v1.has(Permission::GROUP_EXEC), eq true);
    assert_that!(v1.has(Permission::OTHERS_READ), eq false);
    assert_that!(v1.has(Permission::OTHERS_WRITE), eq true);
    assert_that!(v1.has(Permission::OTHERS_EXEC), eq false);
    assert_that!(v1.has(Permission::SET_GID), eq true);
    assert_that!(v1.has(Permission::SET_UID), eq false);
    assert_that!(v1.has(Permission::STICKY_BIT), eq false);
}

#[test]
pub fn set_permissions_on_non_existing_path_fails() {
    test_requires!(POSIX_SUPPORT_PERMISSIONS);

    let file_path = generate_file_path();
    let path = file_path.into();

    assert_that!(
        Permission::OWNER_ALL.apply_to(&path),
        eq(Err(SetPermissionError::DoesNotExist))
    );
}

#[test]
pub fn set_permissions_on_existings_path_works() {
    test_requires!(POSIX_SUPPORT_PERMISSIONS);

    let file_path = generate_file_path();

    let initial_permission =
        Permission::OWNER_ALL | Permission::GROUP_READ_WRITE | Permission::OTHERS_EXEC;
    let _file = FileBuilder::new(&file_path)
        .creation_mode(CreationMode::CreateExclusive)
        .permission(initial_permission)
        .create()
        .expect("creating file exclusive works");

    let path = file_path.into();
    let metadata = Metadata::from_path(&path).unwrap();
    assert_that!(metadata.permission(), eq(initial_permission));

    let final_permission =
        Permission::OWNER_READ_WRITE | Permission::GROUP_EXEC | Permission::OTHERS_READ;
    final_permission
        .apply_to(&path)
        .expect("setting final permission work");

    let metadata = Metadata::from_path(&path).unwrap();
    assert_that!(metadata.permission(), eq(final_permission));
}
