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
    use iceoryx2::port::LoanError;
    use iceoryx2::prelude::*;
    use iceoryx2::testing::*;
    use iceoryx2_bb_testing::assert_that;

    use super::number_of_allocations_in;

    type Ipc = ipc::Service;
    type IpcThreadsafe = ipc_threadsafe::Service;

    const SLICE_LEN: usize = 8;
    const PAYLOAD: [u64; SLICE_LEN] = [0, 1, 2, 3, 4, 5, 6, 7];
    const MAX_LOANS: usize = 1;
    const MAX_CLIENTS: usize = 1;
    const MAX_ACTIVE_REQUESTS: usize = 1;
    const CHUNKS_A_CLIENT_BORROWS: usize = MAX_LOANS + 2 * MAX_ACTIVE_REQUESTS;

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
    pub fn sample_chunk_returns_once_after_its_last_holder_is_dropped<S: Service>() {
        let node = create_node::<S>();
        let service = node
            .service_builder(&generate_service_name())
            .publish_subscribe::<[u64]>()
            .create()
            .unwrap();
        let publisher = service
            .publisher_builder()
            .initial_max_slice_len(SLICE_LEN)
            .max_loaned_samples(MAX_LOANS)
            .create()
            .unwrap();
        let subscriber = service.subscriber_builder().create().unwrap();

        let sample = publisher.loan_slice_uninit(SLICE_LEN).unwrap();
        drop(sample.__internal_create_resizable_memory());
        assert_that!(publisher.loan_slice_uninit(SLICE_LEN).err(), eq Some(LoanError::ExceedsMaxLoans));
        drop(sample);

        let sample = publisher.loan_slice_uninit(SLICE_LEN).unwrap();
        let memory = sample.__internal_create_resizable_memory();
        drop(sample);
        assert_that!(publisher.loan_slice_uninit(SLICE_LEN).err(), eq Some(LoanError::ExceedsMaxLoans));
        drop(memory);

        let sample = publisher.loan_slice_uninit(SLICE_LEN).unwrap();
        let memory = sample.__internal_create_resizable_memory();
        sample.write_from_fn(|n| n as u64).send().unwrap();
        assert_that!(*subscriber.receive().unwrap().unwrap(), eq PAYLOAD);
        assert_that!(publisher.loan_slice_uninit(SLICE_LEN).err(), eq Some(LoanError::ExceedsMaxLoans));
        drop(memory);

        let _sample = publisher.loan_slice_uninit(SLICE_LEN).unwrap();
        assert_that!(publisher.loan_slice_uninit(SLICE_LEN).err(), eq Some(LoanError::ExceedsMaxLoans));
    }

    #[test]
    pub fn request_chunk_returns_once_after_its_last_holder_is_dropped<S: Service>() {
        let node = create_node::<S>();
        let service = node
            .service_builder(&generate_service_name())
            .request_response::<[u64], u64>()
            .max_loaned_requests(MAX_LOANS)
            .max_active_requests_per_client(MAX_ACTIVE_REQUESTS)
            .create()
            .unwrap();
        let client = service
            .client_builder()
            .initial_max_slice_len(SLICE_LEN)
            .create()
            .unwrap();
        let server = service.server_builder().create().unwrap();

        let request = client.loan_slice_uninit(SLICE_LEN).unwrap();
        drop(request.__internal_create_resizable_memory());
        drop(request);

        let request = client.loan_slice_uninit(SLICE_LEN).unwrap();
        let memory = request.__internal_create_resizable_memory();
        let pending_response = request.write_from_fn(|n| n as u64).send().unwrap();
        assert_that!(*server.receive().unwrap().unwrap(), eq PAYLOAD);
        drop(pending_response);

        let memories: Vec<_> = (1..CHUNKS_A_CLIENT_BORROWS)
            .map(|_| {
                let request = client.loan_slice_uninit(SLICE_LEN).unwrap();
                request.__internal_create_resizable_memory()
            })
            .collect();
        assert_that!(client.loan_slice_uninit(SLICE_LEN).err(), eq Some(LoanError::ExceedsMaxLoans));
        drop(memory);

        let request = client.loan_slice_uninit(SLICE_LEN).unwrap();
        let _memory = request.__internal_create_resizable_memory();
        drop(request);
        assert_that!(client.loan_slice_uninit(SLICE_LEN).err(), eq Some(LoanError::ExceedsMaxLoans));
        drop(memories);
    }

    #[test]
    pub fn response_chunk_returns_once_after_its_last_holder_is_dropped<S: Service>() {
        let node = create_node::<S>();
        let service = node
            .service_builder(&generate_service_name())
            .request_response::<u64, [u64]>()
            .max_clients(MAX_CLIENTS)
            .max_active_requests_per_client(MAX_ACTIVE_REQUESTS)
            .create()
            .unwrap();
        let client = service.client_builder().create().unwrap();
        let server = service
            .server_builder()
            .initial_max_slice_len(SLICE_LEN)
            .max_loaned_responses_per_request(MAX_LOANS)
            .create()
            .unwrap();
        let pending_response = client.send_copy(0).unwrap();
        let active_request = server.receive().unwrap().unwrap();

        let response = active_request.loan_slice_uninit(SLICE_LEN).unwrap();
        drop(response.__internal_create_resizable_memory());
        drop(response);

        let response = active_request.loan_slice_uninit(SLICE_LEN).unwrap();
        let memory = response.__internal_create_resizable_memory();
        response.write_from_fn(|n| n as u64).send().unwrap();
        assert_that!(*pending_response.receive().unwrap().unwrap(), eq PAYLOAD);
        assert_that!(active_request.loan_slice_uninit(SLICE_LEN).err(), eq Some(LoanError::ExceedsMaxLoans));
        drop(memory);
        drop(active_request);
        drop(pending_response);

        let _pending_response = client.send_copy(0).unwrap();
        let active_request = server.receive().unwrap().unwrap();
        assert_that!(active_request.loan_slice_uninit(SLICE_LEN), is_ok);
    }
}
