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

use iceoryx2_bb_posix::memory_lock::*;
use iceoryx2_bb_testing::{assert_that, test_requires};
use iceoryx2_pal_posix::posix;
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_MEMORY_LOCK;

#[test]
fn memory_lock_works() {
    test_requires!(POSIX_SUPPORT_MEMORY_LOCK);

    let some_memory = [0u8; 1024];

    {
        let mem_lock = unsafe {
            MemoryLock::new(
                some_memory.as_ptr() as *const posix::void,
                some_memory.len(),
            )
        };
        assert_that!(mem_lock, is_ok);
    }
}

#[test]
fn memory_lock_all_works() {
    test_requires!(POSIX_SUPPORT_MEMORY_LOCK);

    assert_that!(
        MemoryLock::lock_all(LockMode::LockAllPagesThatBecomeMapped),
        is_ok
    );

    MemoryLock::unlock_all();
}
