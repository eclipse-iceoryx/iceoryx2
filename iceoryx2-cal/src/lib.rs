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

extern crate alloc;

pub mod arc_sync_policy;
pub mod communication_channel;
pub mod dynamic_storage;
pub mod event;
pub mod hash;
pub mod monitoring;
pub mod named_concept;
pub mod reactor;
pub mod resizable_shared_memory;
pub mod serialize;
pub mod shared_memory;
pub mod shared_memory_directory;
pub mod shm_allocator;
pub mod static_storage;
pub mod zero_copy_connection;

#[doc(hidden)]
pub mod testing;
