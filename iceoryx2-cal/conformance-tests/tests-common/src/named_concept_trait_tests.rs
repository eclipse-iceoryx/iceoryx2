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

use iceoryx2_bb_testing::instantiate_conformance_tests_with_module;

use iceoryx2_cal::communication_channel::posix_shared_memory::Channel as CommunicationChannelPosixSharedMemory;
use iceoryx2_cal::communication_channel::process_local::Channel as CommunicationChannelProcessLocal;
use iceoryx2_cal::communication_channel::unix_datagram::Channel as CommunicationChannelUnixDatagram;
use iceoryx2_cal::dynamic_storage::file::Storage as DynamicStorageFile;
use iceoryx2_cal::dynamic_storage::posix_shared_memory::Storage as DynamicStoragePosixSharedMemory;
use iceoryx2_cal::dynamic_storage::process_local::Storage as DynamicStorageProcessLocal;
use iceoryx2_cal::event::process_local_socketpair::EventImpl as EventProcessLocal;
use iceoryx2_cal::event::unix_datagram_socket::EventImpl as EventUnixDatagram;
use iceoryx2_cal::monitoring::file_lock::FileLockMonitoring as MonitoringFileLock;
use iceoryx2_cal::monitoring::process_local::ProcessLocalMonitoring as MonitoringProcessLocal;
use iceoryx2_cal::resizable_shared_memory::dynamic::DynamicMemory as ResizableSharedMemoryDynamic;
use iceoryx2_cal::shared_memory::file::Memory as SharedMemoryFile;
use iceoryx2_cal::shared_memory::posix::Memory as SharedMemoryPosix;
use iceoryx2_cal::shared_memory::process_local::Memory as SharedMemoryProcessLocal;
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;
use iceoryx2_cal::static_storage::file::Storage as StaticStorageFile;
use iceoryx2_cal::static_storage::process_local::Storage as StaticStorageProcessLocal;
use iceoryx2_cal::zero_copy_connection::posix_shared_memory::Connection as ZeroCopyConnectionPosixSharedMemory;
use iceoryx2_cal::zero_copy_connection::process_local::Connection as ZeroCopyConnectionProcessLocal;

instantiate_conformance_tests_with_module!(
    communication_channel_posix_shared_memory,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    CommunicationChannelTest<super::CommunicationChannelPosixSharedMemory>
);

instantiate_conformance_tests_with_module!(
    communication_channel_process_local,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    CommunicationChannelTest<super::CommunicationChannelProcessLocal>
);

instantiate_conformance_tests_with_module!(
    communication_channel_unix_datagram,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    CommunicationChannelTest<super::CommunicationChannelUnixDatagram<u64>>
);

instantiate_conformance_tests_with_module!(
    dynamic_storage_posix_shared_memory,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    DynamicStorageTest<super::DynamicStoragePosixSharedMemory<u64>>
);

instantiate_conformance_tests_with_module!(
    dynamic_storage_file,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    DynamicStorageTest<super::DynamicStorageFile<u64>>
);

instantiate_conformance_tests_with_module!(
    dynamic_storage_process_local,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    DynamicStorageTest<super::DynamicStorageProcessLocal<u64>>
);

instantiate_conformance_tests_with_module!(
    event_process_local,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    EventTest<super::EventProcessLocal>
);

instantiate_conformance_tests_with_module!(
    event_unix_datagram,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    EventTest<super::EventUnixDatagram>
);

instantiate_conformance_tests_with_module!(
    monitoring_file_lock,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    MonitoringTest<super::MonitoringFileLock>
);

instantiate_conformance_tests_with_module!(
    monitoring_process_local,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    MonitoringTest<super::MonitoringProcessLocal>
);

instantiate_conformance_tests_with_module!(
    resizable_shared_memory_file,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    ResizableSharedMemoryTest < super::SharedMemoryFile::<super::PoolAllocator>,
    super::ResizableSharedMemoryDynamic::<super::PoolAllocator, super::SharedMemoryFile::<super::PoolAllocator>>>
);

instantiate_conformance_tests_with_module!(
    resizable_shared_memory_posix,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    ResizableSharedMemoryTest < super::SharedMemoryPosix::<super::PoolAllocator>,
    super::ResizableSharedMemoryDynamic::<super::PoolAllocator, super::SharedMemoryPosix::<super::PoolAllocator>>>
);

instantiate_conformance_tests_with_module!(
    resizable_shared_memory_process_local,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    ResizableSharedMemoryTest < super::SharedMemoryProcessLocal::<super::PoolAllocator>,
    super::ResizableSharedMemoryDynamic::<super::PoolAllocator, super::SharedMemoryProcessLocal::<super::PoolAllocator>>>
);

instantiate_conformance_tests_with_module!(
    shared_memory_file,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    SharedMemoryTest<super::SharedMemoryFile::<super::PoolAllocator>>
);

instantiate_conformance_tests_with_module!(
    shared_memory_posix,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    SharedMemoryTest<super::SharedMemoryPosix::<super::PoolAllocator>>
);

instantiate_conformance_tests_with_module!(
    shared_memory_process_local,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    SharedMemoryTest<super::SharedMemoryProcessLocal::<super::PoolAllocator>>
);

instantiate_conformance_tests_with_module!(
    static_storage_file,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    StaticStorageTest<super::StaticStorageFile>
);

instantiate_conformance_tests_with_module!(
    static_storage_process_local,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    StaticStorageTest<super::StaticStorageProcessLocal>
);

instantiate_conformance_tests_with_module!(
    zero_copy_connection_process_local,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    ZeroCopyConnectionTest<super::ZeroCopyConnectionProcessLocal>
);

instantiate_conformance_tests_with_module!(
    zero_copy_connection_posix_shared_memory,
    iceoryx2_cal_conformance_tests::named_concept_trait,
    ZeroCopyConnectionTest<super::ZeroCopyConnectionPosixSharedMemory>
);
