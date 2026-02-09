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

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod atomic_tests;
pub mod lazy_lock_tests;
pub mod once_tests;
pub mod spin_lock_tests;
pub mod strategy_barrier_tests;
pub mod strategy_condition_variable_tests;
pub mod strategy_mutex_tests;
pub mod strategy_rwlock_tests;
pub mod strategy_semaphore_tests;
