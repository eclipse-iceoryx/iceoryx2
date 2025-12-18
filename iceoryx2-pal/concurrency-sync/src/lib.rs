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

#![cfg_attr(not(test), no_std)]

extern crate alloc;

const SPIN_REPETITIONS: u64 = 10000;

pub mod atomic;
pub mod barrier;
pub mod cell;
pub mod condition_variable;
pub mod mutex;
pub mod once;
pub mod rwlock;
pub mod semaphore;

#[derive(Debug, PartialEq, Eq)]
pub enum WaitAction {
    Continue,
    Abort,
}

#[derive(Debug, PartialEq, Eq)]
pub enum WaitResult {
    Interrupted,
    Success,
}
