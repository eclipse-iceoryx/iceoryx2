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
//! let service_name = ServiceName::new("My/Funk/ServiceName")?;
//!
//! // use `process_local` as communication variant
//! let service = process_local::Service::new(&service_name)
//!     .publish_subscribe()
//!     .open_or_create::<u64>()?;
//!
//! let publisher = service.publisher().create()?;
//! let subscriber = service.subscriber().create()?;
//!
//! # Ok(())
//! # }
//! ```

use crate::service::dynamic_config::DynamicConfig;
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;
use iceoryx2_cal::*;

use super::ServiceState;

/// Defines a process local or single address space communication setup.
#[derive(Debug)]
pub struct Service {
    state: ServiceState<
        static_storage::process_local::Storage,
        dynamic_storage::process_local::Storage<DynamicConfig>,
    >,
}

impl crate::service::Service for Service {
    type StaticStorage = static_storage::process_local::Storage;
    type ConfigSerializer = serialize::toml::Toml;
    type DynamicStorage = dynamic_storage::process_local::Storage<DynamicConfig>;
    type ServiceNameHasher = hash::sha1::Sha1;
    type SharedMemory = shared_memory::process_local::Memory<PoolAllocator>;
    type Connection = zero_copy_connection::process_local::Connection;
    type Event = event::sem_bitset_process_local::Event;

    fn from_state(state: ServiceState<Self::StaticStorage, Self::DynamicStorage>) -> Self {
        Self { state }
    }

    fn state(&self) -> &ServiceState<Self::StaticStorage, Self::DynamicStorage> {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ServiceState<Self::StaticStorage, Self::DynamicStorage> {
        &mut self.state
    }
}
