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

use alloc::vec;
use core::alloc::Layout;
use iceoryx2_bb_elementary_traits::allocator::Deallocate;
use iceoryx2_bb_posix::file::AccessMode;
use iceoryx2_bb_posix::testing::generate_file_path;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::conformance_test;
use iceoryx2_bb_testing_macros::conformance_tests;
use iceoryx2_cal::named_concept::*;
use iceoryx2_cal::shared_memory::*;
use iceoryx2_cal::shm_allocator::AllocationError;
use iceoryx2_cal::shm_allocator::ShmAllocator;
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;
use iceoryx2_cal::testing::*;

pub type DefaultAllocator = PoolAllocator;

type AllocatorConfig = <DefaultAllocator as ShmAllocator>::Configuration;

const NUMBER_OF_CHUNKS: usize = 32;
const CHUNK_SIZE: usize = 8192;
const DEFAULT_SIZE: usize = CHUNK_SIZE * NUMBER_OF_CHUNKS;
const DEFAULT_LAYOUT: Layout = unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, 1) };
const SHM_CONFIG: AllocatorConfig = AllocatorConfig {
    bucket_layout: DEFAULT_LAYOUT,
};

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod shared_memory_with_dealloc_trait {
    use super::*;

    #[conformance_test]
    pub fn allocation_with_creator_works<
        Sut: SharedMemory<DefaultAllocator> + Deallocate<ShmPointer>,
    >() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut::Configuration>();
        let mut chunks = vec![];

        let sut_create = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
            .config(&config)
            .create(&SHM_CONFIG)
            .unwrap();

        for _ in 0..NUMBER_OF_CHUNKS {
            let chunk = sut_create.allocate(DEFAULT_LAYOUT);
            assert_that!(chunk, is_ok);
            chunks.push(chunk.unwrap());
        }

        let chunk = sut_create.allocate(DEFAULT_LAYOUT);
        assert_that!(chunk, is_err);
        assert_that!(
            chunk.err().unwrap(), eq
            AllocationError::OutOfMemory
        );

        unsafe {
            sut_create.deallocate(chunks[0], DEFAULT_LAYOUT);
        }

        let chunk = sut_create.allocate(DEFAULT_LAYOUT);
        assert_that!(chunk, is_ok);
    }

    #[conformance_test]
    pub fn allocation_with_creator_and_client_works<
        Sut: SharedMemory<DefaultAllocator> + Deallocate<ShmPointer>,
    >() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut::Configuration>();
        let mut chunks = vec![];

        let sut_create = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
            .config(&config)
            .create(&SHM_CONFIG)
            .unwrap();

        for _ in 0..NUMBER_OF_CHUNKS / 2 {
            let chunk = sut_create.allocate(DEFAULT_LAYOUT);
            assert_that!(chunk, is_ok);
            chunks.push(chunk.unwrap());
        }

        let sut_open = Sut::Builder::new(&name)
            .config(&config)
            .open(AccessMode::ReadWrite)
            .unwrap();
        for _ in 0..NUMBER_OF_CHUNKS / 2 {
            let chunk = sut_open.allocate(DEFAULT_LAYOUT);
            assert_that!(chunk, is_ok);
            chunks.push(chunk.unwrap());
        }

        let chunk = sut_open.allocate(DEFAULT_LAYOUT);
        assert_that!(chunk, is_err);
        assert_that!(
            chunk.err().unwrap(), eq
            AllocationError::OutOfMemory
        );

        unsafe {
            sut_create.deallocate(chunks[0], DEFAULT_LAYOUT);
        }

        let chunk = sut_open.allocate(DEFAULT_LAYOUT);
        assert_that!(chunk, is_ok);
    }
}
