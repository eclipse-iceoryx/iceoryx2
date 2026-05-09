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

use alloc::vec;
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
use iceoryx2_bb_lock_free::mpmc::robust_unique_index_set::*;
use iceoryx2_bb_lock_free::mpmc::unique_index_set_enums::{
    ReleaseMode, ReleaseState, UniqueIndexSetAcquireFailure,
};
use iceoryx2_bb_posix::barrier::{BarrierBuilder, BarrierHandle, Handle};
use iceoryx2_bb_posix::system_configuration::SystemInfo;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

const CAPACITY: usize = 128;

#[test]
pub fn capacity_is_set_correctly() {
    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();
    assert_that!(sut.capacity(), eq CAPACITY);

    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new_with_reduced_capacity(CAPACITY * 2);
    assert_that!(sut, is_err);

    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new_with_reduced_capacity(CAPACITY / 2);
    assert_that!(sut, is_ok);
    assert_that!(sut.unwrap().capacity(), eq CAPACITY / 2);

    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new_with_reduced_capacity(0);
    assert_that!(sut, is_err);
}
