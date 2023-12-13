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

use iceoryx2_bb_posix::permission::*;
use iceoryx2_bb_testing::assert_that;

#[test]
pub fn permission_setting_and_reading_works() {
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
