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

#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

//! Library of lock-free constructs.
//!
//! From C++ Concurrency in Action - Anthony Williams
//!
//! Obstruction-Free: If all other threads are paused, then any given thread will complete its
//!                     operation in a bounded number of steps.
//! Lock-Free: If multiple threads are operating on a data structure, then after a bounded number
//!             of steps one of them will complete its operation.
//! Wait-Free: Every thread operating on a data structure will complete its operation in a bounded
//!             number of steps, even if other threads are also operating on the data structure.
//!
//! Lock-Free guarantees that a misbehaving thread cannot block any other thread.

pub mod mpmc;
pub mod spmc;
pub mod spsc;
