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

pub mod access_mode_tests;
pub mod adaptive_wait_tests;
pub mod barrier_tests;
pub mod clock_tests;
pub mod creation_mode_tests;
pub mod deadline_queue_tests;
pub mod directory_tests;
pub mod file_descriptor_set_tests;
pub mod file_descriptor_tests;
pub mod file_lock_tests;
pub mod file_tests;
pub mod file_type_tests;
pub mod group_tests;
pub mod ipc_capable_trait_tests;
pub mod memory_lock_tests;
pub mod memory_mapping_tests;
pub mod memory_tests;
pub mod metadata_tests;
pub mod mutex_tests;
pub mod ownership_tests;
pub mod permission_tests;
pub mod process_state_tests;
pub mod process_tests;
pub mod read_write_mutex_tests;
pub mod scheduler_tests;
pub mod semaphore_tests;
pub mod shared_memory_tests;
pub mod signal_set_tests;
pub mod signal_tests;
pub mod socket_ancillary_tests;
pub mod socket_pair_tests;
pub mod thread_tests;
pub mod udp_socket_tests;
pub mod unique_system_id_tests;
pub mod unix_datagram_socket_tests;
pub mod users_tests;
