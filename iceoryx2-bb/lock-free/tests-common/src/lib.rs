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

pub mod bitset_tests;
pub mod mpmc_container_tests;
pub mod mpmc_unique_index_set_tests;
pub mod spmc_unrestricted_atomic_tests;
pub mod spsc_index_queue_tests;
pub mod spsc_queue_tests;
pub mod spsc_safely_overflowing_index_queue_tests;
