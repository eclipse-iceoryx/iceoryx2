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

#![allow(clippy::disallowed_types)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate iceoryx2_bb_loggers;

pub mod dynamic_storage_posix_shared_memory_tests;
pub mod pointer_offset_tests;
pub mod shared_memory_posix_shared_memory_tests;
pub mod shm_allocator_bump_allocator_tests;
pub mod shm_allocator_pool_allocator_tests;
pub mod static_storage_file_tests;
pub mod used_chunk_list_tests;
pub mod zero_copy_connection_posix_shared_memory_tests;
