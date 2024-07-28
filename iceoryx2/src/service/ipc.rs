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

//! # Example
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//!
//! // use `ipc` as communication variant
//! let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .open_or_create()?;
//!
//! let publisher = service.publisher_builder().create()?;
//! let subscriber = service.subscriber_builder().create()?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! See [`Service`](crate::service) for more detailed examples.

use std::sync::Arc;

use crate::service::dynamic_config::DynamicConfig;
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;
use iceoryx2_cal::*;

use super::ServiceState;

/// Defines a zero copy inter-process communication setup based on posix mechanisms.
#[derive(Debug)]
pub struct Service {
    state: Arc<ServiceState<Self>>,
}

impl crate::service::Service for Service {
    type StaticStorage = static_storage::file::Storage;
    type ConfigSerializer = serialize::toml::Toml;
    type DynamicStorage = dynamic_storage::posix_shared_memory::Storage<DynamicConfig>;
    type ServiceNameHasher = hash::sha1::Sha1;
    type SharedMemory = shared_memory::posix::Memory<PoolAllocator>;
    type Connection = zero_copy_connection::posix_shared_memory::Connection;
    type Event = event::unix_datagram_socket::EventImpl;
    type Monitoring = monitoring::file_lock::FileLockMonitoring;
}

impl crate::service::internal::ServiceInternal<Service> for Service {
    fn __internal_from_state(state: ServiceState<Self>) -> Self {
        Self {
            state: Arc::new(state),
        }
    }

    fn __internal_state(&self) -> &Arc<ServiceState<Self>> {
        &self.state
    }
}
