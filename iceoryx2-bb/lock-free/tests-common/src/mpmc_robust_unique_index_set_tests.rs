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

#[test]
pub fn acquire_and_release_works() {
    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();

    let owner_id = OwnerId::new(123).unwrap();
    let mut indices = vec![];
    for n in 0..CAPACITY {
        assert_that!(sut.borrowed_indices(), eq n);
        indices.push(sut.acquire(owner_id).unwrap());
        assert_that!(sut.borrowed_indices(), eq n + 1);
    }

    for n in (1..CAPACITY + 1).rev() {
        assert_that!(sut.borrowed_indices(), eq n);
        assert_that!(
            sut.release(indices.pop().unwrap(), owner_id, ReleaseMode::Default),
            is_ok
        );
        assert_that!(sut.borrowed_indices(), eq n - 1);
    }
}

#[test]
pub fn indices_are_unique() {
    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();

    let owner_id = OwnerId::new(123).unwrap();
    let mut indices = vec![];
    for _ in 0..CAPACITY {
        let index = sut.acquire(owner_id).unwrap();

        assert_that!(indices, not_contains index);
        indices.push(index);
    }
}

#[test]
pub fn indices_are_between_zero_and_capacity() {
    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();

    let owner_id = OwnerId::new(123).unwrap();
    for _ in 0..CAPACITY {
        let index = sut.acquire(owner_id).unwrap();

        assert_that!(index, lt CAPACITY);
    }
}

#[test]
pub fn when_full_acquire_returns_out_of_indices() {
    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();

    let owner_id = OwnerId::new(123).unwrap();
    for _ in 0..CAPACITY {
        sut.acquire(owner_id).unwrap();
    }

    assert_that!(sut.acquire(owner_id).err(), eq Some(UniqueIndexSetAcquireFailure::OutOfIndices));
}

#[test]
pub fn release_makes_space_for_more_indices() {
    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();

    let owner_id = OwnerId::new(123).unwrap();
    for _ in 0..CAPACITY {
        sut.acquire(owner_id).unwrap();
    }

    assert_that!(
        sut.release(CAPACITY / 2, owner_id, ReleaseMode::Default),
        is_ok
    );

    assert_that!(sut.acquire(owner_id), is_ok);
    assert_that!(sut.acquire(owner_id).err(), eq Some(UniqueIndexSetAcquireFailure::OutOfIndices));
}

#[test]
pub fn new_set_is_not_locked_and_empty() {
    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();

    assert_that!(sut.is_locked(), eq false);
    assert_that!(sut.borrowed_indices(), eq 0);
}

#[test]
pub fn release_last_index_and_set_release_mode_to_locked_locks_set() {
    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();

    let owner_id = OwnerId::new(123).unwrap();
    let index = sut.acquire(owner_id).unwrap();

    assert_that!(sut.release(index, owner_id, ReleaseMode::LockIfLastIndex), eq Ok(ReleaseState::Locked));
    assert_that!(sut.is_locked(), eq true);
}

#[test]
pub fn release_not_last_index_and_set_release_mode_to_locked_does_not_lock_the_set() {
    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();

    let owner_id = OwnerId::new(123).unwrap();
    let index_1 = sut.acquire(owner_id).unwrap();
    let _index_2 = sut.acquire(owner_id).unwrap();

    assert_that!(sut.release(index_1, owner_id, ReleaseMode::LockIfLastIndex), eq Ok(ReleaseState::Unlocked));
    assert_that!(sut.is_locked(), eq false);
}

#[test]
pub fn releasing_non_owned_index_fails() {
    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();

    let owner_id = OwnerId::new(123).unwrap();
    let bad_owner_id = OwnerId::new(7127).unwrap();
    let index = sut.acquire(owner_id).unwrap();

    assert_that!(sut.release(index, bad_owner_id, ReleaseMode::LockIfLastIndex), eq Err(RobustUniqueIndexSetReleaseError::IndexIsNotOwnedByProvidedOwner));
    assert_that!(sut.is_locked(), eq false);
}

#[test]
pub fn acquire_all_indices_and_release_with_release_mode_lock_locks_set_after_last_release() {
    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();

    let owner_id = OwnerId::new(123).unwrap();
    let mut indices = vec![];

    for _ in 0..CAPACITY {
        indices.push(sut.acquire(owner_id).unwrap());
    }

    for _ in 0..CAPACITY - 1 {
        assert_that!(sut.release(indices.pop().unwrap(), owner_id, ReleaseMode::LockIfLastIndex), eq Ok(ReleaseState::Unlocked));
    }

    assert_that!(sut.release(indices.pop().unwrap(), owner_id, ReleaseMode::LockIfLastIndex), eq Ok(ReleaseState::Locked));
    assert_that!(sut.is_locked(), eq true);
}

#[test]
pub fn new_indices_cannot_be_acquired_from_locked_set() {
    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();

    let owner_id = OwnerId::new(123).unwrap();
    let index = sut.acquire(owner_id).unwrap();
    assert_that!(
        sut.release(index, owner_id, ReleaseMode::LockIfLastIndex),
        is_ok
    );

    assert_that!(sut.acquire(owner_id).err(), eq Some(UniqueIndexSetAcquireFailure::IsLocked));
}

#[test]
pub fn zero_borrowed_indices_in_locked_set() {
    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();

    let owner_id = OwnerId::new(123).unwrap();
    let index = sut.acquire(owner_id).unwrap();
    assert_that!(
        sut.release(index, owner_id, ReleaseMode::LockIfLastIndex),
        is_ok
    );

    assert_that!(sut.borrowed_indices(), eq 0);
}

#[test]
pub fn recover_releases_indices_of_owner() {
    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();

    let mut indices = vec![];
    for n in 0..CAPACITY {
        let owner_id = OwnerId::new(n as u64 + 1).unwrap();
        let index = sut.acquire(owner_id).unwrap();
        indices.push(index);
    }

    for n in 0..CAPACITY {
        let id_to_remove = OwnerId::new(n as u64 + 1).unwrap();
        sut.recover(ReleaseMode::Default, |owner_id| owner_id == id_to_remove);

        assert_that!(sut.borrowed_indices(), eq CAPACITY - n - 1);
    }
}

#[test]
pub fn recover_locks_the_set_if_release_mode_is_lock() {
    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();

    let owner_id = OwnerId::new(558).unwrap();
    for _ in 0..CAPACITY {
        sut.acquire(owner_id).unwrap();
    }

    assert_that!(sut.recover(ReleaseMode::LockIfLastIndex, |id| id == owner_id), eq ReleaseState::Locked);
    assert_that!(sut.is_locked(), eq true);
}

#[test]
pub fn recover_does_not_lock_the_set_if_release_mode_is_lock_and_the_set_is_not_empty_after_recover()
 {
    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();

    let owner_id = OwnerId::new(558).unwrap();
    for _ in 0..CAPACITY / 2 {
        sut.acquire(owner_id).unwrap();
    }
    sut.acquire(OwnerId::new(912).unwrap()).unwrap();

    assert_that!(sut.recover(ReleaseMode::LockIfLastIndex, |id| id == owner_id), eq ReleaseState::Unlocked);
    assert_that!(sut.is_locked(), eq false);
}

#[test]
pub fn recover_of_locked_set_always_returns_locked() {
    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();

    let owner_id = OwnerId::new(123).unwrap();
    let index = sut.acquire(owner_id).unwrap();
    assert_that!(
        sut.release(index, owner_id, ReleaseMode::LockIfLastIndex),
        is_ok
    );

    assert_that!(sut.recover(ReleaseMode::Default, |id| id == owner_id), eq ReleaseState::Locked);
}

#[test]
pub fn acquire_and_release_works_with_uninitialized_memory() {
    let mut memory = [0u8; RobustUniqueIndexSet::const_memory_size(CAPACITY)];
    let allocator = BumpAllocator::new(memory.as_mut_ptr());
    let mut sut = unsafe { RobustUniqueIndexSet::new_uninit(CAPACITY) };
    unsafe { assert_that!(sut.init(&allocator), is_ok) };

    let owner_id = OwnerId::new(123).unwrap();
    let mut indices = vec![];
    for n in 0..CAPACITY {
        assert_that!(sut.borrowed_indices(), eq n);
        indices.push(unsafe { sut.acquire(owner_id).unwrap() });
        assert_that!(sut.borrowed_indices(), eq n + 1);
    }

    for n in (1..CAPACITY + 1).rev() {
        assert_that!(sut.borrowed_indices(), eq n);
        assert_that!(
            unsafe { sut.release(indices.pop().unwrap(), owner_id, ReleaseMode::Default) },
            is_ok
        );
        assert_that!(sut.borrowed_indices(), eq n - 1);
    }
}

#[test]
pub fn concurrent_acquire_release() {
    const REPETITIONS: i64 = 1000;
    let number_of_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, usize::MAX);

    let sut = StaticRobustUniqueIndexSet::<CAPACITY>::new();
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(number_of_threads as u32)
        .create(&barrier_handle)
        .unwrap();
    let owner_id = OwnerId::new(9191).unwrap();

    thread_scope(|s| {
        for _ in 0..number_of_threads {
            s.thread_builder()
                .spawn(|| {
                    let mut indices = vec![];
                    let mut repetition = 0;

                    barrier.wait();
                    loop {
                        match sut.acquire(owner_id) {
                            Ok(e) => {
                                indices.push(e);
                            }
                            Err(UniqueIndexSetAcquireFailure::OutOfIndices) => {
                                repetition += 1;
                                while let Some(index) = indices.pop() {
                                    assert_that!(
                                        sut.release(index, owner_id, ReleaseMode::Default),
                                        is_ok
                                    );
                                }
                                if repetition == REPETITIONS {
                                    break;
                                }
                            }
                            Err(UniqueIndexSetAcquireFailure::IsLocked) => {
                                assert_that!(true, eq false);
                            }
                        }
                    }
                })
                .expect("failed to spawn thread");
        }

        Ok(())
    })
    .expect("failed to run thread scope");

    // check if the sut is still in an consistent state
    let mut ids = vec![];
    let mut id_counter = [0u64; CAPACITY];

    for id in id_counter.iter_mut().take(CAPACITY) {
        let e = sut.acquire(owner_id);
        assert_that!(e, is_ok);
        *id += 1;
        ids.push(e.unwrap());
    }

    for id in id_counter.iter_mut().take(CAPACITY) {
        assert_that!(*id, eq 1);
    }

    let e = sut.acquire(owner_id);
    assert_that!(e, is_err);
    assert_that!(e.err().unwrap(), eq UniqueIndexSetAcquireFailure::OutOfIndices);
}
