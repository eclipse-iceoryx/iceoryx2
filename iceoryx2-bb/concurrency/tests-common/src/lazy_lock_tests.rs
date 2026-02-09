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

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use iceoryx2_bb_concurrency::atomic::{AtomicUsize, Ordering};
use iceoryx2_bb_concurrency::lazy_lock::LazyLock;
use iceoryx2_pal_testing_nostd_macros::requires_std;

pub fn lazy_lock_primitive_type() {
    static VALUE: LazyLock<u32> = LazyLock::new(|| 42);
    assert_eq!(*VALUE, 42);
}

pub fn lazy_lock_complex_type() {
    #[derive(Debug, PartialEq)]
    struct ComplexType {
        name: String,
        value: Vec<i32>,
    }

    static COMPLEX: LazyLock<ComplexType> = LazyLock::new(|| ComplexType {
        name: "test".to_string(),
        value: vec![1, 2, 3, 4, 5],
    });

    assert_eq!(COMPLEX.name, "test");
    assert_eq!(COMPLEX.value.len(), 5);
    assert_eq!(COMPLEX.value[2], 3);
}

pub fn lazy_lock_zero_sized_type() {
    #[derive(Debug, PartialEq)]
    struct ZeroSized;

    static VALUE: LazyLock<ZeroSized> = LazyLock::new(|| ZeroSized);
    assert_eq!(*VALUE, ZeroSized);
}

pub fn lazy_lock_closure() {
    let multiplier = 10;
    let lazy = LazyLock::new(move || multiplier * 5);
    assert_eq!(*lazy, 50);
}

pub fn lazy_lock_non_static() {
    let lazy = LazyLock::new(|| vec![1, 2, 3]);
    assert_eq!(lazy.len(), 3);
    assert_eq!(lazy[1], 2);
}

pub fn lazy_lock_deref() {
    static VALUE: LazyLock<String> = LazyLock::new(|| "hello".to_string());
    assert_eq!(VALUE.len(), 5);
    assert_eq!(&*VALUE, "hello");
}

pub fn lazy_lock_initialization_occurs_once() {
    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);
    static VALUE: LazyLock<u32> = LazyLock::new(|| {
        CALL_COUNT.fetch_add(1, Ordering::SeqCst);
        42
    });

    assert_eq!(*VALUE, 42);
    assert_eq!(*VALUE, 42);
    assert_eq!(*VALUE, 42);

    assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);
}

pub fn lazy_lock_force_initialization() {
    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);
    static VALUE: LazyLock<u32> = LazyLock::new(|| {
        CALL_COUNT.fetch_add(1, Ordering::SeqCst);
        100
    });

    let val = LazyLock::force(&VALUE);
    assert_eq!(*val, 100);
    assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);

    // Subsequent force should not re-initialize
    let val2 = LazyLock::force(&VALUE);
    assert_eq!(*val2, 100);
    assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);
}

pub fn lazy_lock_returns_same_reference() {
    static VALUE: LazyLock<String> = LazyLock::new(|| "hello".to_string());

    let ref1 = &*VALUE;
    let ref2 = &*VALUE;

    assert!(core::ptr::eq(ref1, ref2));
}

pub fn lazy_lock_dependent_initialization() {
    static FIRST: LazyLock<u32> = LazyLock::new(|| 10);
    static SECOND: LazyLock<u32> = LazyLock::new(|| *FIRST * 2);

    assert_eq!(*SECOND, 20);
    assert_eq!(*FIRST, 10);
}

#[requires_std("threading")]
pub fn lazy_lock_access_concurrent_access_from_multiple_threads() {
    use iceoryx2_bb_concurrency::internal::strategy::barrier::Barrier;

    const NUMBER_OF_THREADS: u32 = 10;

    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);
    static VALUE: LazyLock<u32> = LazyLock::new(|| {
        CALL_COUNT.fetch_add(1, Ordering::Relaxed);
        std::thread::sleep(std::time::Duration::from_millis(10));
        123
    });

    let barrier = Barrier::new(NUMBER_OF_THREADS + 1);

    std::thread::scope(|s| {
        for _ in 0..NUMBER_OF_THREADS {
            s.spawn(|| {
                barrier.wait(|_, _| {}, |_| {});
                assert_eq!(*VALUE, 123);
            });
        }

        barrier.wait(|_, _| {}, |_| {});
    });

    assert_eq!(CALL_COUNT.load(Ordering::Relaxed), 1);
}
