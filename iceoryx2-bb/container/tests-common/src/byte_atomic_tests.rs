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

use iceoryx2_bb_container::byte_atomic::*;
use iceoryx2_bb_container::string::StaticString;
use iceoryx2_bb_derive_macros::AtomicCopy;
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary_traits::atomic_copy::AtomicCopy;
use iceoryx2_bb_posix::barrier::*;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

// TODO: refactor tests

#[repr(C)]
#[derive(AtomicCopy, Clone, Copy)]
struct Foo(u8, u64, u32, u16);

#[repr(C)]
#[derive(AtomicCopy, Clone, Copy)]
struct ComplexType {
    a: u8,
    b: StaticString<6>,
    c: f64,
    d: Foo,
}

#[test]
pub fn fixed_size_byte_atomic_cannot_be_created_when_sizes_do_not_match() {
    const SIZE: usize = size_of::<u64>();
    let value: u8 = 0;
    let sut = FixedSizeByteAtomic::<u8, SIZE>::new(value);
    assert_that!(sut, is_err);
    assert_that!(sut.err().unwrap(), eq ByteAtomicError::SizesDoNotMatch);
}

#[test]
pub fn new_creates_byte_atomic_containing_passed_value() {
    let value = 963;

    const SIZE: usize = size_of::<u64>();
    let fixed_size_sut = FixedSizeByteAtomic::<u64, SIZE>::new(value);
    assert_that!(fixed_size_sut, is_ok);
    let read_value = unsafe { fixed_size_sut.unwrap().read().assume_init() };
    assert_that!(read_value, eq value);

    const MEM_SIZE: usize = RelocatableByteAtomic::<u64>::const_memory_size();
    let mut memory = [0u8; MEM_SIZE];
    let allocator = BumpAllocator::new(memory.as_mut_ptr());
    unsafe {
        let mut relocatable_sut = RelocatableByteAtomic::new_uninit();
        relocatable_sut
            .init(&allocator, value)
            .expect("RelocatableByteAtomic initialized.");
        assert_that!(relocatable_sut.read().assume_init(), eq value);
    }
}

#[test]
pub fn new_creates_fixed_size_byte_atomic_containing_passed_complex_value() {
    let value = ComplexType {
        a: 5,
        b: StaticString::<6>::try_from("ato").unwrap(),
        c: -9.3,
        d: Foo(1, 111111, 444, 99),
    };

    const SIZE: usize = size_of::<ComplexType>();
    let fixed_size_sut = FixedSizeByteAtomic::<ComplexType, SIZE>::new(value);
    assert_that!(fixed_size_sut, is_ok);
    let read_value = unsafe { fixed_size_sut.unwrap().read().assume_init() };
    assert_that!(read_value.a, eq value.a);
    assert_that!(read_value.b, eq value.b);
    assert_that!(read_value.c, eq value.c);
    assert_that!(read_value.d.0, eq value.d.0);
    assert_that!(read_value.d.1, eq value.d.1);
    assert_that!(read_value.d.2, eq value.d.2);
    assert_that!(read_value.d.3, eq value.d.3);

    const MEM_SIZE: usize = RelocatableByteAtomic::<ComplexType>::const_memory_size();
    let mut memory = [0u8; MEM_SIZE];
    let allocator = BumpAllocator::new(memory.as_mut_ptr());
    unsafe {
        let mut relocatable_sut = RelocatableByteAtomic::new_uninit();
        relocatable_sut
            .init(&allocator, value)
            .expect("RelocatableByteAtomic initialized.");
        let read_value = relocatable_sut.read().assume_init();
        assert_that!(read_value.a, eq value.a);
        assert_that!(read_value.b, eq value.b);
        assert_that!(read_value.c, eq value.c);
        assert_that!(read_value.d.0, eq value.d.0);
        assert_that!(read_value.d.1, eq value.d.1);
        assert_that!(read_value.d.2, eq value.d.2);
        assert_that!(read_value.d.3, eq value.d.3);
    }
}

#[test]
pub fn byte_atomic_contains_passed_value_after_write() {
    let new_value: u64 = 752389;

    const SIZE: usize = size_of::<u64>();
    let fixed_size_sut = FixedSizeByteAtomic::<u64, SIZE>::new(0).unwrap();
    unsafe {
        fixed_size_sut.write(new_value);
        assert_that!(fixed_size_sut.read().assume_init(), eq new_value);
    }

    const MEM_SIZE: usize = RelocatableByteAtomic::<u64>::const_memory_size();
    let mut memory = [0u8; MEM_SIZE];
    let allocator = BumpAllocator::new(memory.as_mut_ptr());
    unsafe {
        let mut relocatable_sut = RelocatableByteAtomic::new_uninit();
        relocatable_sut
            .init(&allocator, 0)
            .expect("RelocatableByteAtomic initialized.");
        relocatable_sut.write(new_value);
        assert_that!(relocatable_sut.read().assume_init(), eq new_value);
    }
}

#[test]
pub fn byte_atomic_contains_passed_complex_value_after_write() {
    let new_value = ComplexType {
        a: 22,
        b: StaticString::<6>::try_from("smeik").unwrap(),
        c: 7.53,
        d: Foo(6, 734567, 5234, 132),
    };

    const SIZE: usize = size_of::<ComplexType>();
    let init_value = ComplexType {
        a: 0,
        b: StaticString::<6>::new(),
        c: 0.0,
        d: Foo(0, 0, 0, 0),
    };
    let fixed_size_sut = FixedSizeByteAtomic::<ComplexType, SIZE>::new(init_value).unwrap();
    unsafe {
        fixed_size_sut.write(new_value);
        let read_value = fixed_size_sut.read().assume_init();
        assert_that!(read_value.a, eq new_value.a);
        assert_that!(read_value.b, eq new_value.b);
        assert_that!(read_value.c, eq new_value.c);
        assert_that!(read_value.d.0, eq new_value.d.0);
        assert_that!(read_value.d.1, eq new_value.d.1);
        assert_that!(read_value.d.2, eq new_value.d.2);
        assert_that!(read_value.d.3, eq new_value.d.3);
    }

    const MEM_SIZE: usize = RelocatableByteAtomic::<ComplexType>::const_memory_size();
    let mut memory = [0u8; MEM_SIZE];
    let allocator = BumpAllocator::new(memory.as_mut_ptr());
    unsafe {
        let mut relocatable_sut = RelocatableByteAtomic::new_uninit();
        relocatable_sut
            .init(&allocator, init_value)
            .expect("RelocatableByteAtomic initialized.");
        relocatable_sut.write(new_value);
        let read_value = relocatable_sut.read().assume_init();
        assert_that!(read_value.a, eq new_value.a);
        assert_that!(read_value.b, eq new_value.b);
        assert_that!(read_value.c, eq new_value.c);
        assert_that!(read_value.d.0, eq new_value.d.0);
        assert_that!(read_value.d.1, eq new_value.d.1);
        assert_that!(read_value.d.2, eq new_value.d.2);
        assert_that!(read_value.d.3, eq new_value.d.3);
    }
}

// TODO #1601: The following tests should be run with Miri but using
// iceoryx2_bb_posix::barrier/thread::* causes the error: "unsupported operation: can't call
// foreign function ... this means the program tried to do something Miri does not support; it does
// not indicate a bug in the program" for `pthread_attr_init` and `pthread_barrieratrr_init`. The
// tests pass when std::sync::Barrier and std::thread are used.
#[test]
pub fn concurrent_read_without_write_always_returns_correct_data() {
    let value = 481935403;
    let number_of_threads = 16;
    const REPETITIONS: usize = 500;
    let barrier = std::sync::Barrier::new(number_of_threads);
    // let barrier_handle = BarrierHandle::new();
    // let barrier = BarrierBuilder::new(number_of_threads)
    //     .create(&barrier_handle)
    //     .unwrap();

    const SIZE: usize = size_of::<u64>();
    let fixed_size_sut = FixedSizeByteAtomic::<u64, SIZE>::new(value).unwrap();

    const MEM_SIZE: usize = RelocatableByteAtomic::<u64>::const_memory_size();
    let mut memory = [0u8; MEM_SIZE];
    let allocator = BumpAllocator::new(memory.as_mut_ptr());
    let mut relocatable_sut = unsafe { RelocatableByteAtomic::new_uninit() };
    unsafe {
        relocatable_sut
            .init(&allocator, value)
            .expect("RelocatableByteAtomic initialized.");
        relocatable_sut.write(value);
    }

    std::thread::scope(|s| {
        // thread_scope(|s| {
        for _ in 0..number_of_threads {
            s //.thread_builder()
                .spawn(|| {
                    barrier.wait();
                    for _ in 0..REPETITIONS {
                        unsafe {
                            let read_value_fixed_size = fixed_size_sut.read();
                            assert_that!(read_value_fixed_size.assume_init(), eq value);

                            let read_value_relocatable = relocatable_sut.read();
                            assert_that!(read_value_relocatable.assume_init(), eq value);
                        }
                    }
                });
            // .expect("failed to spawn thread");
        }
        // Ok(())
    });
    // .expect("failed to create scoped thread");
}

#[test]
pub fn concurrent_write_does_not_trigger_ub() {
    let value = u64::MAX;
    let number_of_threads = 16;
    const REPETITIONS: usize = 500;
    let barrier = std::sync::Barrier::new(number_of_threads);
    // let barrier_handle = BarrierHandle::new();
    // let barrier = BarrierBuilder::new(number_of_threads)
    //     .create(&barrier_handle)
    //     .unwrap();

    const SIZE: usize = size_of::<u64>();
    let fixed_size_sut = FixedSizeByteAtomic::<u64, SIZE>::new(value).unwrap();

    const MEM_SIZE: usize = RelocatableByteAtomic::<u64>::const_memory_size();
    let mut memory = [0u8; MEM_SIZE];
    let allocator = BumpAllocator::new(memory.as_mut_ptr());
    let mut relocatable_sut = unsafe { RelocatableByteAtomic::new_uninit() };
    unsafe {
        relocatable_sut
            .init(&allocator, value)
            .expect("RelocatableByteAtomic initialized.");
        relocatable_sut.write(value);
    }

    std::thread::scope(|s| {
        // thread_scope(|s| {
        for _ in 0..number_of_threads {
            s //.thread_builder()
                .spawn(|| {
                    barrier.wait();
                    for _ in 0..REPETITIONS {
                        unsafe {
                            fixed_size_sut.write(value);
                            relocatable_sut.write(value);
                        }
                    }
                });
            //.expect("failed to spawn thread");
        }
        //Ok(())
    });
    //.expect("failed to create scoped thread");

    unsafe {
        let read_value_fixed_size = fixed_size_sut.read();
        let read_value_relocatable = relocatable_sut.read();
        // safe because the value is a u64
        assert_that!(read_value_fixed_size.assume_init(), eq value);
        assert_that!(read_value_relocatable.assume_init(), eq value);
    }
}

#[test]
pub fn concurrent_read_and_write_does_not_trigger_ub() {
    let value = 3249780;
    let number_of_threads = 16;
    const REPETITIONS: usize = 500;
    let barrier = std::sync::Barrier::new(number_of_threads);
    // let barrier_handle = BarrierHandle::new();
    // let barrier = BarrierBuilder::new(number_of_threads)
    //     .create(&barrier_handle)
    //     .unwrap();

    const SIZE: usize = size_of::<u64>();
    let fixed_size_sut = FixedSizeByteAtomic::<u64, SIZE>::new(value).unwrap();

    const MEM_SIZE: usize = RelocatableByteAtomic::<u64>::const_memory_size();
    let mut memory = [0u8; MEM_SIZE];
    let allocator = BumpAllocator::new(memory.as_mut_ptr());
    let mut relocatable_sut = unsafe { RelocatableByteAtomic::new_uninit() };
    unsafe {
        relocatable_sut
            .init(&allocator, value)
            .expect("RelocatableByteAtomic initialized.");
        relocatable_sut.write(value);
    }

    std::thread::scope(|s| {
        for _ in 0..number_of_threads / 2 {
            s //.thread_builder()
                .spawn(|| {
                    barrier.wait();
                    for _ in 0..REPETITIONS {
                        unsafe {
                            let read_value_fixed_size = fixed_size_sut.read();
                            let read_value_relocatable = relocatable_sut.read();
                            // dummy assert to prevent the read operation from being optimized away
                            assert_that!(core::mem::size_of_val(&read_value_fixed_size), eq SIZE);
                            assert_that!(core::mem::size_of_val(&read_value_relocatable), eq SIZE);
                        }
                    }
                });
            // .expect("failed to spawn thread");
        }

        for _ in 0..number_of_threads / 2 {
            s //.thread_builder()
                .spawn(|| {
                    barrier.wait();
                    for _ in 0..REPETITIONS {
                        unsafe {
                            fixed_size_sut.write(value);
                            relocatable_sut.write(value);
                        }
                    }
                });
            // .expect("failed to spawn thread");
        }

        // Ok(())
    });
    // .expect("failed to create scoped thread");
}

#[test]
#[should_panic]
pub fn double_init_call_causes_panic() {
    const MEM_SIZE: usize = RelocatableByteAtomic::<u64>::const_memory_size();
    let mut memory = [0u8; MEM_SIZE];
    let bump_allocator = BumpAllocator::new(memory.as_mut_ptr());

    unsafe {
        let mut sut = RelocatableByteAtomic::<u64>::new_uninit();
        sut.init(&bump_allocator, 0).expect("first init succeeds");

        sut.init(&bump_allocator, 0).expect("double init failed");
    }
}

#[test]
#[cfg(debug_assertions)]
#[should_panic]
pub fn panic_is_called_in_debug_mode_if_map_is_not_initialized() {
    unsafe {
        let sut = RelocatableByteAtomic::<u8>::new_uninit();
        sut.write(9);
    }
}
