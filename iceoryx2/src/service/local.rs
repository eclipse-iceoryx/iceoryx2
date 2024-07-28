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
//! let node = NodeBuilder::new().create::<local::Service>()?;
//!
//! // use `local` as communication variant
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

/// Defines a process local or single address space communication setup.
#[derive(Debug)]
pub struct Service {
    state: Arc<ServiceState<Self>>,
}

impl crate::service::Service for Service {
    type StaticStorage = static_storage::process_local::Storage;
    type ConfigSerializer = serialize::toml::Toml;
    type DynamicStorage = dynamic_storage::process_local::Storage<DynamicConfig>;
    type ServiceNameHasher = hash::sha1::Sha1;
    type SharedMemory = shared_memory::process_local::Memory<PoolAllocator>;
    type Connection = zero_copy_connection::process_local::Connection;
    type Event = event::process_local::EventImpl;
    type Monitoring = monitoring::process_local::ProcessLocalMonitoring;
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
