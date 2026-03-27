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

#![allow(clippy::disallowed_types)]

use iceoryx2_bb_testing_macros::tests;

#[tests(
    RelocatableVec<u64>,
    RelocatableVec<u128>,
    RelocatableVec<[u8; 123]>,
    RelocatableQueue<u64>,
    RelocatableQueue<u128>,
    RelocatableQueue<[u8; 123]>,
    Container<u64>,
    Container<u128>,
    Container<[u8; 123]>,
    UniqueIndexSet,
    RelocatableIndexQueue,
    RelocatableSafelyOverflowingIndexQueue,
    RelocatableUsedChunkList
)]
pub mod generic {
    use alloc::boxed::Box;
    use core::ptr::NonNull;
    use iceoryx2_bb_container::{queue::RelocatableQueue, vector::relocatable_vec::RelocatableVec};
    use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
    use iceoryx2_bb_lock_free::{
        mpmc::{container::Container, unique_index_set::UniqueIndexSet},
        spsc::{
            index_queue::RelocatableIndexQueue,
            safely_overflowing_index_queue::RelocatableSafelyOverflowingIndexQueue,
        },
    };
    use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::requires_std;
    use iceoryx2_cal::zero_copy_connection::used_chunk_list::RelocatableUsedChunkList;

    const MEMORY_SIZE: usize = 1024 * 128;

    fn memory() -> Box<[u8; MEMORY_SIZE]> {
        Box::new([0u8; MEMORY_SIZE])
    }

    fn allocator(memory: &mut [u8]) -> BumpAllocator {
        BumpAllocator::new(NonNull::new(memory.as_mut_ptr()).unwrap(), memory.len())
    }

    #[test]
    pub fn init_acquires_less_or_equal_the_required_size_of_bytes<T: RelocatableContainer>() {
        const MAX_CAPACITY: usize = 128;

        for capacity in 1..MAX_CAPACITY {
            let mut memory = memory();
            let allocator = allocator(&mut *memory);

            let mut sut = unsafe { T::new_uninit(capacity) };
            let require_memory_size = T::memory_size(capacity);

            assert_that!(unsafe { sut.init(&allocator) }, is_ok);
            assert_that!(allocator.used_space(), le require_memory_size);
        }
    }

    #[test]
    pub fn init_acquires_less_or_equal_the_required_size_of_bytes_multiple_allocations<
        T: RelocatableContainer,
    >() {
        const MAX_CAPACITY: usize = 18;

        let mut memory = memory();
        let allocator = allocator(&mut *memory);

        let mut current_size = 0;
        for capacity in 1..MAX_CAPACITY {
            let mut sut = unsafe { T::new_uninit(capacity) };
            let require_memory_size = T::memory_size(capacity);

            assert_that!(unsafe { sut.init(&allocator) }, is_ok);
            assert_that!(allocator.used_space(), le current_size + require_memory_size);
            current_size = allocator.used_space();
        }
    }

    #[test]
    #[requires_std("panics")]
    #[should_panic]
    pub fn init_twice_causes_panic<T: RelocatableContainer>() {
        const MAX_CAPACITY: usize = 18;

        let mut memory = memory();
        let allocator = allocator(&mut *memory);

        let mut sut = unsafe { T::new_uninit(MAX_CAPACITY) };

        assert_that!(unsafe { sut.init(&allocator) }, is_ok);

        //panics
        assert_that!(unsafe { sut.init(&allocator) }, is_ok);
    }
}
