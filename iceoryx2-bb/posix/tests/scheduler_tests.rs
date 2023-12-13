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

use iceoryx2_bb_posix::config::DEFAULT_SCHEDULER;
use iceoryx2_bb_posix::scheduler::*;
use iceoryx2_bb_testing::assert_that;

#[test]
fn scheduler_default_scheduler_set_correctly() {
    assert_that!(Scheduler::default(), eq DEFAULT_SCHEDULER)
}
