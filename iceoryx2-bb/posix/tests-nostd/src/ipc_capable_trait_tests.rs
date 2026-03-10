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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_posix_tests_common::ipc_capable_trait_tests;
use iceoryx2_bb_posix_tests_common::ipc_capable_trait_tests::BarrierTest;
use iceoryx2_bb_posix_tests_common::ipc_capable_trait_tests::MutexTest;
use iceoryx2_bb_posix_tests_common::ipc_capable_trait_tests::ReadWriteMutexTest;
use iceoryx2_bb_posix_tests_common::ipc_capable_trait_tests::TestSut;
use iceoryx2_bb_posix_tests_common::ipc_capable_trait_tests::UnnamedSemaphoreTest;
use iceoryx2_bb_testing_nostd_macros::inventory_test_generic;

#[inventory_test_generic(BarrierTest, UnnamedSemaphoreTest, MutexTest, ReadWriteMutexTest)]
fn ipc_capable_trait_new_handle_is_not_initialized<Sut: TestSut>() {
    ipc_capable_trait_tests::ipc_capable_trait_new_handle_is_not_initialized::<Sut>();
}

#[inventory_test_generic(BarrierTest, UnnamedSemaphoreTest, MutexTest, ReadWriteMutexTest)]
fn ipc_capable_trait_creating_ipc_construct_from_uninitialized_handle_panics<Sut: TestSut>() {
    ipc_capable_trait_tests::ipc_capable_trait_creating_ipc_construct_from_uninitialized_handle_panics::<Sut>();
}

#[inventory_test_generic(BarrierTest, UnnamedSemaphoreTest, MutexTest, ReadWriteMutexTest)]
fn ipc_capable_trait_creating_ipc_construct_from_process_local_handle_panics<Sut: TestSut>() {
    ipc_capable_trait_tests::ipc_capable_trait_creating_ipc_construct_from_process_local_handle_panics::<Sut>();
}

#[inventory_test_generic(BarrierTest, UnnamedSemaphoreTest, MutexTest, ReadWriteMutexTest)]
fn ipc_capable_trait_creating_ipc_construct_from_ipc_handle_works<Sut: TestSut>() {
    ipc_capable_trait_tests::ipc_capable_trait_creating_ipc_construct_from_ipc_handle_works::<Sut>(
    );
}

#[inventory_test_generic(BarrierTest, UnnamedSemaphoreTest, MutexTest, ReadWriteMutexTest)]
fn ipc_capable_trait_init_handle_twice_panics<Sut: TestSut>() {
    ipc_capable_trait_tests::ipc_capable_trait_init_handle_twice_panics::<Sut>();
}

#[inventory_test_generic(BarrierTest, UnnamedSemaphoreTest, MutexTest, ReadWriteMutexTest)]
fn ipc_capable_trait_initialized_handle_is_initialized<Sut: TestSut>() {
    ipc_capable_trait_tests::ipc_capable_trait_initialized_handle_is_initialized::<Sut>();
}

#[inventory_test_generic(BarrierTest, UnnamedSemaphoreTest, MutexTest, ReadWriteMutexTest)]
fn ipc_capable_trait_inter_process_capability_is_set_correctly<Sut: TestSut>() {
    ipc_capable_trait_tests::ipc_capable_trait_inter_process_capability_is_set_correctly::<Sut>();
}
