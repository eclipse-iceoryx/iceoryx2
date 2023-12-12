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

use std::time::Duration;

use iceoryx2_bb_posix::{process::Process, unique_system_id::*};
use iceoryx2_bb_testing::assert_that;

#[test]
fn unique_system_id_is_unique() {
    let sut1 = UniqueSystemId::new().unwrap();
    std::thread::sleep(Duration::from_secs(1));
    let sut2 = UniqueSystemId::new().unwrap();
    std::thread::sleep(Duration::from_secs(1));
    let sut3 = UniqueSystemId::new().unwrap();

    assert_that!(sut1.value(), ne sut2.value());

    let pid = Process::from_self().id();

    assert_that!(sut1.pid(), eq pid);
    assert_that!(sut2.pid(), eq pid);

    assert_that!(sut2.creation_time().seconds(), gt sut1.creation_time().seconds());
    assert_that!(sut3.creation_time().seconds(), gt sut2.creation_time().seconds());
    assert_that!(sut1.creation_time().seconds() + 2, ge sut2.creation_time().seconds());
    assert_that!(sut1.creation_time().seconds() + 3, ge sut3.creation_time().seconds());
}
