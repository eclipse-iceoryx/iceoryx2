use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct CustomServiceVariant {}

impl iceoryx2::service::Service for CustomServiceVariant {
    type StaticStorage = iceoryx2_cal::static_storage::recommended::Ipc;
    type ConfigSerializer = iceoryx2_cal::serialize::recommended::Recommended;
    // use a dynamic storage based on a file
    type DynamicStorage = iceoryx2_cal::dynamic_storage::file::Storage<
        iceoryx2::service::dynamic_config::DynamicConfig,
    >;
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
    type ArcThreadSafetyPolicy<T: Send + Debug> =
        iceoryx2_cal::arc_sync_policy::single_threaded::SingleThreaded<T>;
    // the blackboard mgmt segment is replaced with a file based version
    type BlackboardMgmt<KeyType: Send + Sync + Debug + 'static> =
        iceoryx2_cal::dynamic_storage::file::Storage<KeyType>;
    // the blackboard payload is replaced with a file based version
    type BlackboardPayload = iceoryx2_cal::shared_memory::file::Memory<
        iceoryx2_cal::shm_allocator::bump_allocator::BumpAllocator,
    >;
}

impl iceoryx2::service::internal::ServiceInternal<CustomServiceVariant> for CustomServiceVariant {}
