// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use std::fmt::Debug;

use iceoryx2::prelude::ZeroCopySend;
use iceoryx2::service::Service;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;

#[derive(Debug, Clone)]
pub struct CustomService {}

#[derive(Debug, Clone)]
pub struct CustomServiceThreadSafe {}

impl Service for CustomService {
    // use a static storage based on a file
    type StaticStorage = iceoryx2_cal::static_storage::file::Storage;
    type ConfigSerializer = iceoryx2_cal::serialize::recommended::Recommended;
    // persistent dynamic storage is also file based
    type PersistentDynamicStorage<T: Debug + Send + Sync + ZeroCopySend + 'static> =
        iceoryx2_cal::dynamic_storage::file::Storage<T>;
    type Bag = iceoryx2_cal::bag::recommended::Recommended;
    // use a dynamic storage based on a file
    type DynamicStorage<T: Debug + Send + Sync + ZeroCopySend + 'static> =
        iceoryx2_cal::dynamic_storage::file::Storage<T>;
    type ServiceNameHasher = iceoryx2_cal::hash::recommended::Recommended;
    // instead of using POSIX shared memory, we use a file
    type SharedMemory = iceoryx2_cal::shared_memory::file::Memory<
        iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator,
    >;
    // instead of using resizable shared memory based on POSIX shared memory, we
    // use a variant based on a file
    type ResizableSharedMemory = iceoryx2_cal::resizable_shared_memory::dynamic::DynamicMemory<
        iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator,
        iceoryx2_cal::shared_memory::file::Memory<
            iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator,
        >,
    >;
    // instead of using a connection based on POSIX shared memory, we use a
    // variant based on a file
    type Connection = iceoryx2_cal::zero_copy_connection::file::Connection;
    type Event = iceoryx2_cal::event::recommended::Ipc;
    type Monitoring = iceoryx2_cal::monitoring::recommended::Ipc;
    type Reactor = iceoryx2_cal::reactor::recommended::Ipc;
    type ArcThreadSafetyPolicy<T: Send + Debug + Abandonable> =
        iceoryx2_cal::arc_sync_policy::mutex_protected::MutexProtected<T>;
    // the blackboard mgmt segment is replaced with a file based version
    type BlackboardMgmt<KeyType: Send + Sync + Debug + ZeroCopySend + 'static> =
        iceoryx2_cal::dynamic_storage::file::Storage<KeyType>;
    // the blackboard payload is replaced with a file based version
    type BlackboardPayload = iceoryx2_cal::shared_memory::file::Memory<
        iceoryx2_cal::shm_allocator::bump_allocator::BumpAllocator,
    >;
    type UniqueId = iceoryx2::unique_id_generator::unique_system_id::UniqueSystemIdGenerator<Self>;
    type NameGenerator = iceoryx2::name_generator::default_name_generator::DefaultNameGenerator;
    type Clock = iceoryx2_cal::clock::posix::PosixClock<Self::Event>;
}

// this will be set as IpcService for the ffi service types
impl Service for CustomServiceThreadSafe {
    type StaticStorage = <CustomService as Service>::StaticStorage;
    type ConfigSerializer = <CustomService as Service>::ConfigSerializer;
    type PersistentDynamicStorage<T: Debug + Send + Sync + ZeroCopySend + 'static> =
        <CustomService as Service>::PersistentDynamicStorage<T>;
    type Bag = <CustomService as Service>::Bag;
    type DynamicStorage<T: Debug + Send + Sync + ZeroCopySend + 'static> =
        <CustomService as Service>::DynamicStorage<T>;
    type ServiceNameHasher = <CustomService as Service>::ServiceNameHasher;
    type SharedMemory = <CustomService as Service>::SharedMemory;
    type ResizableSharedMemory = <CustomService as Service>::ResizableSharedMemory;
    type Connection = <CustomService as Service>::Connection;
    type Event = <CustomService as Service>::Event;
    type Monitoring = <CustomService as Service>::Monitoring;
    type Reactor = <CustomService as Service>::Reactor;
    // use a mutex for thread-safety
    type ArcThreadSafetyPolicy<T: Send + Debug + Abandonable> =
        iceoryx2_cal::arc_sync_policy::mutex_protected::MutexProtected<T>;
    type BlackboardMgmt<KeyType: Send + Sync + Debug + ZeroCopySend + 'static> =
        <CustomService as Service>::BlackboardMgmt<KeyType>;
    type BlackboardPayload = <CustomService as Service>::BlackboardPayload;
    type UniqueId = iceoryx2::unique_id_generator::unique_system_id::UniqueSystemIdGenerator<Self>;
    type NameGenerator = <CustomService as Service>::NameGenerator;
    type Clock = <CustomService as Service>::Clock;
}

impl iceoryx2::service::internal::ServiceInternal<CustomService> for CustomService {}
impl iceoryx2::service::internal::ServiceInternal<CustomServiceThreadSafe>
    for CustomServiceThreadSafe
{
}
