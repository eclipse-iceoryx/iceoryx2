// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_testing_macros::tests;

#[tests(1, 2, 3, 128)]
pub mod generic {
    use alloc::vec;

    use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
    use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::zero_copy_connection::used_chunk_list::FixedSizeUsedChunkList;
    use iceoryx2_cal::zero_copy_connection::used_chunk_list::RelocatableUsedChunkList;

    #[test]
    fn insert_remove_all_works<const CAPACITY: usize>() {
        let mut sut = FixedSizeUsedChunkList::<CAPACITY>::new();

        for i in 0..sut.capacity() {
            assert_that!(sut.insert(i), eq true);
        }

        let mut removed_indices = vec![false; sut.capacity()];
        sut.remove_all(|index| {
            removed_indices[index] = true;
        });

        for index in removed_indices {
            assert_that!(index, eq true);
        }
    }

    #[test]
    fn insert_remove_works<const CAPACITY: usize>() {
        let sut = FixedSizeUsedChunkList::<CAPACITY>::new();

        for i in 0..sut.capacity() {
            assert_that!(sut.remove(i), eq false);
            assert_that!(sut.insert(i), eq true);
            assert_that!(sut.remove(i), eq true);
            assert_that!(sut.remove(i), eq false);

            assert_that!(sut.insert(i), eq true);
        }

        for i in (0..sut.capacity()).rev() {
            assert_that!(sut.remove(i), eq true);
            assert_that!(sut.remove(i), eq false);
        }
    }

    #[test]
    fn list_initialized_on_dirty_memory_starts_unused<const CAPACITY: usize>() {
        let mut memory = [u8::MAX; CAPACITY];
        let allocator = BumpAllocator::new(
            core::ptr::NonNull::new(memory.as_mut_ptr()).unwrap(),
            memory.len(),
        );
        let mut sut = unsafe { RelocatableUsedChunkList::new_uninit(CAPACITY) };
        assert_that!(unsafe { sut.init(&allocator) }, is_ok);

        for i in 0..sut.capacity() {
            assert_that!(sut.remove(i), eq false);
        }
    }

    #[test]
    fn list_initialized_on_zeroed_memory_starts_unused<const CAPACITY: usize>() {
        let mut memory = [0u8; CAPACITY];
        let allocator = BumpAllocator::new(
            core::ptr::NonNull::new(memory.as_mut_ptr()).unwrap(),
            memory.len(),
        );
        let mut sut = unsafe { RelocatableUsedChunkList::new_uninit(CAPACITY) };
        assert_that!(unsafe { sut.init_on_zeroed_memory(&allocator) }, is_ok);

        for i in 0..sut.capacity() {
            assert_that!(sut.remove(i), eq false);
            assert_that!(sut.insert(i), eq true);
            assert_that!(sut.insert(i), eq false);
        }

        let mut removed_indices = vec![false; sut.capacity()];
        sut.remove_all(|index| {
            removed_indices[index] = true;
        });
        for index in removed_indices {
            assert_that!(index, eq true);
        }
    }
}
