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

use iceoryx2_bb_concurrency_tests_common::strategy_semaphore_tests;

#[test]
fn strategy_semaphore_post_and_try_wait_works() {
    strategy_semaphore_tests::strategy_semaphore_post_and_try_wait_works();
}

#[test]
fn strategy_semaphore_post_and_wait_works() {
    strategy_semaphore_tests::strategy_semaphore_post_and_wait_works();
}

#[test]
fn strategy_semaphore_wait_blocks() {
    strategy_semaphore_tests::strategy_semaphore_wait_blocks();
}
