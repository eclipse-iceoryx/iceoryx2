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
#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

extern crate alloc;
extern crate iceoryx2_bb_loggers;

mod arc_sync_policy_trait_tests;
mod communication_channel_trait_tests;
mod dynamic_storage_trait_tests;
mod event_id_tracker_trait_tests;
mod event_signal_mechanism_trait_tests;
mod event_trait_tests;
mod monitoring_trait_tests;
mod reactor_trait_tests;
mod resizable_shared_memory_trait_tests;
mod serialize_trait_tests;
mod shared_memory_trait_tests;
mod shm_allocator_trait_tests;
mod static_storage_trait_tests;
mod zero_copy_connection_trait_tests;
