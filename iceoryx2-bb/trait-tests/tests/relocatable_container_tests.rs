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

#[generic_tests::define]
mod relocatable_container {
    use core::ptr::NonNull;
    use iceoryx2_bb_container::{queue::RelocatableQueue, vec::RelocatableVec};
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
    use iceoryx2_cal::zero_copy_connection::used_chunk_list::RelocatableUsedChunkList;

    const MEMORY_SIZE: usize = 1024 * 128;

    fn memory() -> Box<[u8; MEMORY_SIZE]> {
        Box::new([0u8; MEMORY_SIZE])
    }

    fn allocator(memory: &mut [u8]) -> BumpAllocator {
        BumpAllocator::new(
            NonNull::new(memory.as_mut_ptr() as *mut u8).unwrap(),
            memory.len(),
        )
    }

    #[test]
    fn init_acquires_less_or_equal_the_required_size_of_bytes<T: RelocatableContainer>() {
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
    fn init_acquires_less_or_equal_the_required_size_of_bytes_multiple_allocations<
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
    #[should_panic]
    fn init_twice_causes_panic<T: RelocatableContainer>() {
        const MAX_CAPACITY: usize = 18;

        let mut memory = memory();
        let allocator = allocator(&mut *memory);

        let mut sut = unsafe { T::new_uninit(MAX_CAPACITY) };

        assert_that!(unsafe { sut.init(&allocator) }, is_ok);

        //panics
        assert_that!(unsafe { sut.init(&allocator) }, is_ok);
    }

    #[instantiate_tests(<RelocatableVec<u64>>)]
    mod vec_u64 {}

    #[instantiate_tests(<RelocatableVec<u128>>)]
    mod vec_u128 {}

    #[instantiate_tests(<RelocatableVec<[u8; 123]>>)]
    mod vec_special {}

    #[instantiate_tests(<RelocatableQueue<u64>>)]
    mod queue_u64 {}

    #[instantiate_tests(<RelocatableQueue<u128>>)]
    mod queue_u128 {}

    #[instantiate_tests(<RelocatableQueue<[u8; 123]>>)]
    mod queue_special {}

    #[instantiate_tests(<Container<u64>>)]
    mod container_u64 {}

    #[instantiate_tests(<Container<u128>>)]
    mod container_u128 {}

    #[instantiate_tests(<Container<[u8; 123]>>)]
    mod container_special {}

    #[instantiate_tests(<UniqueIndexSet>)]
    mod unique_index_set {}

    #[instantiate_tests(<RelocatableIndexQueue>)]
    mod index_queue {}

    #[instantiate_tests(<RelocatableSafelyOverflowingIndexQueue>)]
    mod safely_overflowing_index_queue {}

    #[instantiate_tests(<RelocatableUsedChunkList>)]
    mod used_chunk_list {}
}
