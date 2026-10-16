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

use core::fmt::Debug;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_cal::shm_allocator::bump_allocator::BumpAllocator;
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;
use iceoryx2_cal::*;

/// Defines a zero copy inter-process communication setup with a simulated clock that can be
/// acquired via [`Node::time_controller()`](crate::node::Node::time_controller()).
#[derive(Debug, Clone)]
pub struct Service {}

impl crate::service::Service for Service {
    type StaticStorage = static_storage::recommended::Ipc;
    type ConfigSerializer = serialize::recommended::Recommended;
    type PersistentDynamicStorage<T: Debug + Send + Sync + ZeroCopySend + 'static> =
        dynamic_storage::recommended::PersistentIpc<T>;
    type Bag = bag::recommended::Recommended;
    type DynamicStorage<T: Debug + Send + Sync + ZeroCopySend + 'static> =
        dynamic_storage::recommended::Ipc<T>;
    type ServiceNameHasher = hash::recommended::Recommended;
    type SharedMemory = shared_memory::recommended::Ipc<PoolAllocator>;
    type ResizableSharedMemory = resizable_shared_memory::recommended::Ipc<PoolAllocator>;
    type Connection = zero_copy_connection::recommended::Ipc;
    type Event = event::recommended::Ipc;
    type Monitoring = monitoring::recommended::Ipc;
    type Reactor = reactor::recommended::Ipc;
    type ArcThreadSafetyPolicy<T: Send + Debug + Abandonable> =
        arc_sync_policy::single_threaded::SingleThreaded<T>;
    type BlackboardMgmt<KeyType: Send + Sync + Debug + ZeroCopySend + 'static> =
        dynamic_storage::recommended::Ipc<KeyType>;
    type BlackboardPayload = shared_memory::recommended::Ipc<BumpAllocator>;
    type UniqueId = crate::unique_id_generator::unique_system_id::UniqueSystemIdGenerator<Service>;
    type NameGenerator = crate::name_generator::default_name_generator::DefaultNameGenerator;
    type Clock = clock::simulation_clock::SimulationClock<Self::Event>;
}

impl crate::service::internal::ServiceInternal<Service> for Service {}
