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
use iceoryx2_bb_posix::user::*;
use iceoryx2_bb_system_types::user_name::UserName;
use iceoryx2_bb_testing::{assert_that, test_requires};
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_USERS_AND_GROUPS;

#[test]
fn user_works() {
    test_requires!(POSIX_SUPPORT_USERS_AND_GROUPS);

    let root = User::from_name(&UserName::new(b"root").unwrap()).unwrap();
    let root_from_uid = User::from_uid(0).unwrap();

    assert_that!(root.uid(), eq root_from_uid.uid());
    assert_that!(root.uid(), eq 0);

    assert_that!(root.gid(), eq root_from_uid.gid());
    assert_that!(root.gid(), eq 0);

    assert_that!(root.name(), eq root_from_uid.name());
    assert_that!(root.name().as_bytes(), eq b"root");

    assert_that!(root.info(), eq root_from_uid.info());

    assert_that!(root.home_dir(), eq root_from_uid.home_dir());
    assert_that!(root.home_dir().to_string(), eq "/root");

    assert_that!(root.shell(), eq root_from_uid.shell());
    assert_that!(root.password(), eq root_from_uid.password());
}
