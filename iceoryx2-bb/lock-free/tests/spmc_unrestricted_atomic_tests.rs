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

use core::sync::atomic::{AtomicBool, Ordering};
use std::{
    alloc::{alloc, dealloc, Layout},
    ptr::addr_of,
    sync::Mutex,
    thread,
};

use iceoryx2_bb_lock_free::spmc::unrestricted_atomic::*;
use iceoryx2_bb_posix::{barrier::*, system_configuration::SystemInfo};
use iceoryx2_bb_testing::assert_that;

const NUMBER_OF_RUNS: usize = 100000;
const DATA_SIZE: usize = 1024;

static TEST_LOCK: Mutex<bool> = Mutex::new(false);

fn verify(value: u8, rhs: &[u8; DATA_SIZE]) -> bool {
    for i in 0..DATA_SIZE {
        if value != rhs[i] {
            return false;
        }
    }

    true
}

fn verify_no_data_race(rhs: &[u8; DATA_SIZE]) -> bool {
    let value = rhs[0];
    for i in 0..DATA_SIZE {
        if value != rhs[i] {
            return false;
        }
    }

    true
}

#[test]
fn spmc_unrestricted_atomic_acquire_multiple_producer_fails() {
    let _test_lock = TEST_LOCK.lock().unwrap();
    let sut = UnrestrictedAtomic::<[u8; DATA_SIZE]>::new([0xff; DATA_SIZE]);

    let p1 = sut.acquire_producer();
    assert_that!(p1, is_some);
    let p2 = sut.acquire_producer();
    assert_that!(p2, is_none);

    drop(p1);

    let p3 = sut.acquire_producer();
    assert_that!(p3, is_some);
}

#[test]
fn spmc_unrestricted_atomic_load_store_works() {
    let _test_lock = TEST_LOCK.lock().unwrap();
    let sut = UnrestrictedAtomic::<[u8; DATA_SIZE]>::new([0xff; DATA_SIZE]);
    assert_that!(verify(0xff, &sut.load()), eq true);

    for i in 0..NUMBER_OF_RUNS {
        let idx = i % 255;
        sut.acquire_producer()
            .unwrap()
            .store([(idx) as u8; DATA_SIZE]);
        assert_that!(verify((idx) as u8, &sut.load()), eq true);
    }
}

#[test]
fn spmc_unrestricted_atomic_load_store_works_concurrently() {
    let _test_lock = TEST_LOCK.lock().unwrap();
    let number_of_threads = SystemInfo::NumberOfCpuCores.value();
    let store_finished = AtomicBool::new(false);
    let sut = UnrestrictedAtomic::<[u8; DATA_SIZE]>::new([0xff; DATA_SIZE]);
    let handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(number_of_threads as u32 + 1)
        .create(&handle)
        .unwrap();

    thread::scope(|s| {
        for _ in 0..number_of_threads {
            s.spawn(|| {
                barrier.wait();

                while !store_finished.load(Ordering::Relaxed) {
                    assert_that!(verify_no_data_race(&sut.load()), eq true);
                }
            });
        }

        s.spawn(|| {
            barrier.wait();
            let producer = sut.acquire_producer().unwrap();

            for i in 0..NUMBER_OF_RUNS {
                producer.store([(i % 255) as u8; DATA_SIZE]);
            }

            store_finished.store(true, Ordering::Relaxed);
        });
    });
}

#[test]
fn spmc_unrestricted_atomic_get_ptr_write_and_update_works() {
    let _test_lock = TEST_LOCK.lock().unwrap();
    let sut = UnrestrictedAtomic::<u32>::new(0);

    let p = sut.acquire_producer().unwrap();
    let entry = unsafe { p.__internal_get_ptr_to_write_cell() };
    assert_that!(sut.load(), eq 0);

    unsafe { *entry = 1 };
    assert_that!(sut.load(), eq 0);

    unsafe { p.__internal_update_write_cell() };
    assert_that!(sut.load(), eq 1);
}

#[test]
fn spmc_unrestricted_atomic_get_ptr_write_and_update_works_concurrently() {
    let _test_lock = TEST_LOCK.lock().unwrap();
    let store_finished = AtomicBool::new(false);
    let sut = UnrestrictedAtomic::<u128>::new(0);
    let handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&handle).unwrap();

    let mut values = vec![];
    thread::scope(|s| {
        s.spawn(|| {
            barrier.wait();

            while !store_finished.load(Ordering::Relaxed) {
                values.push(sut.load());
            }
        });

        s.spawn(|| {
            let producer = sut.acquire_producer().unwrap();
            barrier.wait();

            for i in 0..NUMBER_OF_RUNS as u128 {
                let entry = unsafe { producer.__internal_get_ptr_to_write_cell() };
                unsafe { *entry = i };
                unsafe { producer.__internal_update_write_cell() };
            }

            store_finished.store(true, Ordering::Relaxed);
        });
    });

    let mut pred = 0;
    for v in values {
        assert_that!(v, le NUMBER_OF_RUNS as u128);
        assert_that!(v, ge pred);
        pred = v;
    }
}

#[test]
fn spmc_unrestricted_atomic_mgmt_release_producer_allows_new_acquire() {
    let _test_lock = TEST_LOCK.lock().unwrap();
    let sut = UnrestrictedAtomicMgmt::new();

    let p1 = unsafe { sut.__internal_acquire_producer() };
    assert_that!(p1, is_ok);
    assert_that!(unsafe { sut.__internal_acquire_producer() }, is_err);

    unsafe {
        sut.__internal_release_producer();
    }

    let p2 = unsafe { sut.__internal_acquire_producer() };
    assert_that!(p2, is_ok);
}

#[test]
fn spmc_unrestricted_atomic_mgmt_get_ptr_write_and_update_works() {
    let _test_lock = TEST_LOCK.lock().unwrap();

    let value: u64 = 3;
    let value_ptr: *const u64 = &value;
    let value_size = core::mem::size_of::<u64>();
    let value_alignment = core::mem::align_of::<u64>();

    let mut read_value: u64 = 0;
    let read_value_ptr: *mut u64 = &mut read_value;

    let atomic = UnrestrictedAtomic::<u64>::new(0);
    let data_ptr = atomic.__internal_get_data_ptr();
    let mgmt = atomic.__internal_get_mgmt();

    unsafe {
        assert_that!(mgmt.__internal_acquire_producer(), is_ok);
        let write_cell_ptr =
            mgmt.__internal_get_ptr_to_write_cell(value_size, value_alignment, data_ptr);
        core::ptr::copy_nonoverlapping(value_ptr as *const u8, write_cell_ptr, value_size);

        // new value not read before update
        mgmt.load(
            read_value_ptr as *mut u8,
            value_size,
            value_alignment,
            data_ptr,
        );
        assert_that!(read_value, eq 0);

        mgmt.__internal_update_write_cell();

        // new value can be read
        mgmt.load(
            read_value_ptr as *mut u8,
            value_size,
            value_alignment,
            data_ptr,
        );
        assert_that!(read_value, eq value);

        mgmt.__internal_release_producer();
    }
}

fn internal_pointer_calculation_works<ValueType: Copy + Default>() {
    let layout = Layout::new::<UnrestrictedAtomic<ValueType>>();

    unsafe {
        let random_ptr = alloc(layout) as *mut UnrestrictedAtomic<ValueType>;
        *(random_ptr) = UnrestrictedAtomic::<ValueType>::new(ValueType::default());

        let mgmt_ptr = addr_of!(*(&*random_ptr).__internal_get_mgmt());
        let data_ptr = addr_of!(*(&*random_ptr).__internal_get_data_ptr());

        for i in -(align_of::<UnrestrictedAtomic<ValueType>>() as isize) + 1..0 {
            let mut ptr = random_ptr;
            ptr = ptr.byte_offset(i);
            let ptrs = __internal_calculate_atomic_mgmt_and_payload_ptr(
                ptr as *mut u8,
                align_of::<ValueType>(),
            );

            assert_that!(mgmt_ptr as *mut u8, eq ptrs.atomic_mgmt_ptr);
            assert_that!(data_ptr as *mut u8, eq ptrs.atomic_payload_ptr);
        }

        dealloc(random_ptr as *mut u8, layout);
    }
}

#[test]
fn spmc_unrestricted_atomic_internal_ptr_calculation_works_with_integers() {
    internal_pointer_calculation_works::<u8>();
    internal_pointer_calculation_works::<u16>();
    internal_pointer_calculation_works::<u32>();
    internal_pointer_calculation_works::<u64>();
    internal_pointer_calculation_works::<i8>();
    internal_pointer_calculation_works::<i16>();
    internal_pointer_calculation_works::<i32>();
    internal_pointer_calculation_works::<i64>();
}
