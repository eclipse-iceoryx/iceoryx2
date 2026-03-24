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

mod active_request_tests;
mod client_tests;
mod listener_tests;
mod node_death_tests;
mod node_tests;
mod notifier_tests;
mod pending_response_tests;
mod publisher_tests;
mod reader_tests;
mod sample_mut_tests;
mod sample_tests;
mod server_tests;
mod service_blackboard_tests;
mod service_event_tests;
mod service_publish_subscribe_tests;
mod service_request_response_builder_tests;
mod service_request_response_tests;
mod service_tests;
mod subscriber_tests;
mod waitset_tests;
mod writer_tests;
