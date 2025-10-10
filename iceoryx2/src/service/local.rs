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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
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

use core::fmt::Debug;

use crate::service::dynamic_config::DynamicConfig;
use iceoryx2_cal::shm_allocator::bump_allocator::BumpAllocator;
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;
use iceoryx2_cal::*;

/// Defines a process local or single address space communication setup.
#[derive(Debug, Clone)]
pub struct Service {}

impl crate::service::Service for Service {
    type StaticStorage = static_storage::recommended::Local;
    type ConfigSerializer = serialize::recommended::Recommended;
    type DynamicStorage = dynamic_storage::recommended::Local<DynamicConfig>;
    type ServiceNameHasher = hash::recommended::Recommended;
    type SharedMemory = shared_memory::recommended::Local<PoolAllocator>;
    type ResizableSharedMemory = resizable_shared_memory::recommended::Local<PoolAllocator>;
    type Connection = zero_copy_connection::recommended::Local;
    type Event = event::recommended::Local;
    type Monitoring = monitoring::recommended::Local;
    type Reactor = reactor::recommended::Local;
    type ArcThreadSafetyPolicy<T: Send + Debug> =
        arc_sync_policy::single_threaded::SingleThreaded<T>;
    type BlackboardMgmt<KeyType: Send + Sync + Debug + 'static> =
        dynamic_storage::recommended::Local<KeyType>;
    type BlackboardPayload = shared_memory::recommended::Local<BumpAllocator>;
}

impl crate::service::internal::ServiceInternal<Service> for Service {}
