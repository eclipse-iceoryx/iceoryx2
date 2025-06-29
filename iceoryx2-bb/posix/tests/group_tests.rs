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
use iceoryx2_bb_posix::group::*;
use iceoryx2_bb_system_types::group_name::GroupName;
use iceoryx2_bb_testing::{assert_that, test_requires};
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_USERS_AND_GROUPS;

#[test]
fn group_works() {
    test_requires!(POSIX_SUPPORT_USERS_AND_GROUPS);

    let root = GroupName::new(b"root").unwrap();
    let wheel = GroupName::new(b"wheel").unwrap();

    let (group_from_name, group_name) = if let Ok(group) = Group::from_name(&root) {
        (group, root)
    } else if let Ok(group) = Group::from_name(&wheel) {
        (group, wheel)
    } else {
        unreachable!("Neither group 'root' not group 'wheel' is found!")
    };

    let group_from_gid = Group::from_gid(Gid::new_from_native(0)).unwrap();

    assert_that!(group_from_name.gid(), eq group_from_gid.gid());
    assert_that!(group_from_name.gid().value(), eq 0);

    let group_details = group_from_name.details().unwrap();
    let group_from_gid_details = group_from_gid.details().unwrap();
    assert_that!(group_details.name(), eq group_from_gid_details.name());
    assert_that!(*group_details.name(), eq group_name);
}

#[test]
fn group_as_works() {
    test_requires!(POSIX_SUPPORT_USERS_AND_GROUPS);

    let root_1 = 0u32.as_group().unwrap();
    let root_2 = root_1
        .details()
        .unwrap()
        .name()
        .to_string()
        .as_group()
        .unwrap();
    let root_3 = root_1
        .details()
        .unwrap()
        .name()
        .to_string()
        .as_group()
        .unwrap();

    assert_that!(root_2.gid().value(), eq 0);
    assert_that!(root_3.gid().value(), eq 0);

    assert_that!(root_1.details().unwrap().members().len(), ge 0);
}
