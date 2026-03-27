// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

#![allow(clippy::disallowed_types)]

use iceoryx2_bb_container::atomic_memcpy::*;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::inventory_test;

#[inventory_test]
pub fn atomic_memcpy_cannot_be_created_when_sizes_do_not_match() {
    const SIZE: usize = size_of::<u64>();
    let value: u8 = 0;
    let sut = AtomicMemcpy::<u8, SIZE>::new(value);
    assert_that!(sut, is_err);
    assert_that!(sut.err().unwrap(), eq AtomicMemcpyError::AtomicMemcpyCreateError);
}

// TODO: test other types
#[inventory_test]
pub fn new_creates_atomic_memcpy_containing_passed_value() {
    const SIZE: usize = size_of::<u64>();
    let value = 963;
    let sut = AtomicMemcpy::<u64, SIZE>::new(value);
    assert_that!(sut, is_ok);

    let read_value = unsafe { sut.unwrap().read().assume_init() };
    assert_that!(read_value, eq value);
}

#[inventory_test]
pub fn atomic_memcpy_contains_passed_value_after_write() {
    const SIZE: usize = size_of::<u64>();
    let mut sut = AtomicMemcpy::<u64, SIZE>::new(0).unwrap();

    let new_value: u64 = 752389;
    unsafe {
        sut.write(new_value);
        assert_that!(sut.read().assume_init(), eq new_value);
    }
}
