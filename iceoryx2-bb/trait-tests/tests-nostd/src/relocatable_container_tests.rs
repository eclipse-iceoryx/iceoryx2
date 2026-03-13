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

extern crate iceoryx2_bb_loggers;

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

use iceoryx2_bb_testing_nostd_macros::inventory_test_generic;
use iceoryx2_bb_trait_tests_common::relocatable_container_tests;

#[inventory_test_generic(
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
fn init_acquires_less_or_equal_the_required_size_of_bytes<T: RelocatableContainer>() {
    relocatable_container_tests::init_acquires_less_or_equal_the_required_size_of_bytes::<T>();
}

#[inventory_test_generic(
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
fn init_acquires_less_or_equal_the_required_size_of_bytes_multiple_allocations<
    T: RelocatableContainer,
>() {
    relocatable_container_tests::init_acquires_less_or_equal_the_required_size_of_bytes_multiple_allocations::<T>();
}

#[inventory_test_generic(
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
#[should_panic]
fn init_twice_causes_panic<T: RelocatableContainer>() {
    relocatable_container_tests::init_twice_causes_panic::<T>();
}
