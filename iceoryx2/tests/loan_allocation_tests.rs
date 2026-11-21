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

extern crate iceoryx2_bb_loggers;

use core::alloc::{GlobalAlloc, Layout};
use std::alloc::System;

use iceoryx2_bb_concurrency::cell::Cell;
use iceoryx2_bb_testing_macros::tests;

iceoryx2_bb_testing::test_harness!();

struct CountingAllocator;

std::thread_local! {
    static IS_COUNTING: Cell<bool> = const { Cell::new(false) };
    static NUMBER_OF_ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if IS_COUNTING.try_with(Cell::get).unwrap_or(false) {
            let _ = NUMBER_OF_ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

fn number_of_allocations_in<F: FnOnce()>(f: F) -> usize {
    NUMBER_OF_ALLOCATIONS.with(|n| n.set(0));
    IS_COUNTING.with(|c| c.set(true));
    f();
    IS_COUNTING.with(|c| c.set(false));
    NUMBER_OF_ALLOCATIONS.with(Cell::get)
}

#[tests(Ipc, IpcThreadsafe)]
mod loan_allocation {
    use iceoryx2::prelude::*;
    use iceoryx2::testing::*;
    use iceoryx2_bb_testing::assert_that;

    use super::number_of_allocations_in;

    type Ipc = ipc::Service;
    type IpcThreadsafe = ipc_threadsafe::Service;

    const SLICE_LEN: usize = 8;
    const MAX_LOANED_SAMPLES: usize = 1;

    fn create_node<S: Service>() -> Node<S> {
        let config = generate_isolated_config();
        NodeBuilder::new().config(&config).create::<S>().unwrap()
    }

    #[test]
    pub fn publisher_loan_does_not_allocate<S: Service>() {
        let node = create_node::<S>();
        let service = node
            .service_builder(&generate_service_name())
            .publish_subscribe::<u64>()
            .create()
            .unwrap();
        let publisher = service.publisher_builder().create().unwrap();
        drop(publisher.loan().unwrap());

        let allocations = number_of_allocations_in(|| drop(publisher.loan().unwrap()));

        assert_that!(allocations, eq 0);
    }

    #[test]
    pub fn publisher_loan_uninit_does_not_allocate<S: Service>() {
        let node = create_node::<S>();
        let service = node
            .service_builder(&generate_service_name())
            .publish_subscribe::<u64>()
            .create()
            .unwrap();
        let publisher = service.publisher_builder().create().unwrap();
        drop(publisher.loan_uninit().unwrap());

        let allocations = number_of_allocations_in(|| drop(publisher.loan_uninit().unwrap()));

        assert_that!(allocations, eq 0);
    }

    #[test]
    pub fn publisher_loan_slice_uninit_does_not_allocate<S: Service>() {
        let node = create_node::<S>();
        let service = node
            .service_builder(&generate_service_name())
            .publish_subscribe::<[u64]>()
            .create()
            .unwrap();
        let publisher = service
            .publisher_builder()
            .initial_max_slice_len(SLICE_LEN)
            .create()
            .unwrap();
        drop(publisher.loan_slice_uninit(SLICE_LEN).unwrap());

        let allocations =
            number_of_allocations_in(|| drop(publisher.loan_slice_uninit(SLICE_LEN).unwrap()));

        assert_that!(allocations, eq 0);
    }

    #[test]
    pub fn client_loan_does_not_allocate<S: Service>() {
        let node = create_node::<S>();
        let service = node
            .service_builder(&generate_service_name())
            .request_response::<u64, u64>()
            .create()
            .unwrap();
        let client = service.client_builder().create().unwrap();
        drop(client.loan().unwrap());

        let allocations = number_of_allocations_in(|| drop(client.loan().unwrap()));

        assert_that!(allocations, eq 0);
    }

    #[test]
    pub fn active_request_loan_does_not_allocate<S: Service>() {
        let node = create_node::<S>();
        let service = node
            .service_builder(&generate_service_name())
            .request_response::<u64, u64>()
            .create()
            .unwrap();
        let client = service.client_builder().create().unwrap();
        let server = service.server_builder().create().unwrap();
        let _pending_response = client.send_copy(0).unwrap();
        let active_request = server.receive().unwrap().unwrap();
        drop(active_request.loan().unwrap());

        let allocations = number_of_allocations_in(|| drop(active_request.loan().unwrap()));

        assert_that!(allocations, eq 0);
    }

    #[test]
    pub fn loan_that_created_resizable_memory_is_returned_when_dropped<S: Service>() {
        let node = create_node::<S>();
        let service = node
            .service_builder(&generate_service_name())
            .publish_subscribe::<[u64]>()
            .create()
            .unwrap();
        let publisher = service
            .publisher_builder()
            .initial_max_slice_len(SLICE_LEN)
            .max_loaned_samples(MAX_LOANED_SAMPLES)
            .create()
            .unwrap();
        let sample = publisher.loan_slice_uninit(SLICE_LEN).unwrap();
        drop(sample.__internal_create_resizable_memory());
        drop(sample);

        assert_that!(publisher.loan_slice_uninit(SLICE_LEN), is_ok);
    }
}
