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

// Custom service variant that uses the
// NamedPipeCountingBitSet event implementation for the Event concept,
// all other concepts use the recommended IPC defaults.

use std::fmt::Debug;

use iceoryx2::prelude::ZeroCopySend;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;

#[derive(Debug, Clone)]
pub struct NamedPipeEventService {}

impl iceoryx2::service::Service for NamedPipeEventService {
    type StaticStorage = iceoryx2_cal::static_storage::recommended::Ipc;
    type ConfigSerializer = iceoryx2_cal::serialize::recommended::Recommended;
    type PersistentDynamicStorage<T: Debug + Send + Sync + ZeroCopySend + 'static> =
        iceoryx2_cal::dynamic_storage::recommended::PersistentIpc<T>;
    type Bag = iceoryx2_cal::bag::recommended::Recommended;
    type DynamicStorage<T: Debug + Send + Sync + ZeroCopySend + 'static> =
        iceoryx2_cal::dynamic_storage::recommended::Ipc<T>;
    type ServiceNameHasher = iceoryx2_cal::hash::recommended::Recommended;
    type SharedMemory = iceoryx2_cal::shared_memory::recommended::Ipc<
        iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator,
    >;
    type ResizableSharedMemory = iceoryx2_cal::resizable_shared_memory::recommended::Ipc<
        iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator,
    >;
    type Connection = iceoryx2_cal::zero_copy_connection::recommended::Ipc;
    type Event = iceoryx2_cal::event::NamedPipeCountingBitSet;
    type Monitoring = iceoryx2_cal::monitoring::recommended::Ipc;
    type Reactor = iceoryx2_cal::reactor::recommended::Ipc;
    type ArcThreadSafetyPolicy<T: Send + Debug + Abandonable> =
        iceoryx2_cal::arc_sync_policy::single_threaded::SingleThreaded<T>;
    type BlackboardMgmt<KeyType: Send + Sync + Debug + ZeroCopySend + 'static> =
        iceoryx2_cal::dynamic_storage::recommended::Ipc<KeyType>;
    type BlackboardPayload = iceoryx2_cal::shared_memory::recommended::Ipc<
        iceoryx2_cal::shm_allocator::bump_allocator::BumpAllocator,
    >;
    type UniqueId = iceoryx2::unique_id_generator::unique_system_id::UniqueSystemIdGenerator<
        NamedPipeEventService,
    >;
    type NameGenerator = iceoryx2::name_generator::default_name_generator::DefaultNameGenerator;
    type Clock = iceoryx2_cal::clock::posix::PosixClock<Self::Event>;
}

impl iceoryx2::service::internal::ServiceInternal<NamedPipeEventService> for NamedPipeEventService {}
