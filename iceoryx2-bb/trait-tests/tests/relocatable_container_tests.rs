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

extern crate iceoryx2_bb_loggers;

#[generic_tests::define]
mod relocatable_container {
    use iceoryx2_bb_container::{queue::RelocatableQueue, vector::relocatable_vec::RelocatableVec};
    use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
    use iceoryx2_bb_lock_free::{
        mpmc::{container::Container, unique_index_set::UniqueIndexSet},
        spsc::{
            index_queue::RelocatableIndexQueue,
            safely_overflowing_index_queue::RelocatableSafelyOverflowingIndexQueue,
        },
    };
    use iceoryx2_cal::zero_copy_connection::used_chunk_list::RelocatableUsedChunkList;

    use iceoryx2_bb_trait_tests_common::relocatable_container_tests;

    #[test]
    fn init_acquires_less_or_equal_the_required_size_of_bytes<T: RelocatableContainer>() {
        relocatable_container_tests::init_acquires_less_or_equal_the_required_size_of_bytes::<T>();
    }

    #[test]
    fn init_acquires_less_or_equal_the_required_size_of_bytes_multiple_allocations<
        T: RelocatableContainer,
    >() {
        relocatable_container_tests::init_acquires_less_or_equal_the_required_size_of_bytes_multiple_allocations::<T>();
    }

    #[test]
    #[should_panic]
    fn init_twice_causes_panic<T: RelocatableContainer>() {
        relocatable_container_tests::init_twice_causes_panic::<T>();
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
