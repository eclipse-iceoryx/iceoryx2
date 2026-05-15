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

use core::alloc::Layout;
use core::any::Any;
use core::marker::PhantomData;
use iceoryx2_bb_posix::testing::generate_file_path;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_testing_macros::conformance_test;
use iceoryx2_bb_testing_macros::conformance_tests;
use iceoryx2_cal::communication_channel::CommunicationChannel;
use iceoryx2_cal::communication_channel::CommunicationChannelCreator;
use iceoryx2_cal::event::Event;
use iceoryx2_cal::event::ListenerBuilder;
use iceoryx2_cal::monitoring::Monitoring;
use iceoryx2_cal::monitoring::MonitoringBuilder;
use iceoryx2_cal::resizable_shared_memory::ResizableSharedMemory;
use iceoryx2_cal::resizable_shared_memory::ResizableSharedMemoryBuilder;
use iceoryx2_cal::shared_memory::SharedMemory;
use iceoryx2_cal::shared_memory::SharedMemoryBuilder;
use iceoryx2_cal::shm_allocator::ShmAllocator;
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;
use iceoryx2_cal::static_storage::StaticStorage;
use iceoryx2_cal::static_storage::StaticStorageBuilder;
use iceoryx2_cal::testing::*;
use iceoryx2_cal::zero_copy_connection::ZeroCopyConnection;
use iceoryx2_cal::zero_copy_connection::ZeroCopyConnectionBuilder;
use iceoryx2_cal::{dynamic_storage::*, named_concept::*};

pub trait NamedConceptTest {
    type Sut: NamedConceptMgmt + 'static;
    fn create(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Box<dyn Any>;
}

pub struct DynamicStorageTest<T: DynamicStorage<u64> + 'static>(PhantomData<T>);

impl<T: DynamicStorage<u64>> NamedConceptTest for DynamicStorageTest<T> {
    type Sut = T;
    fn create(name: &FileName, config: &T::Configuration) -> Box<dyn Any> {
        let sut = <Self::Sut as DynamicStorage<u64>>::Builder::new(name)
            .config(config)
            .has_ownership(true)
            .create(123)
            .unwrap();

        Box::new(sut)
    }
}

pub struct CommunicationChannelTest<T: CommunicationChannel<u64> + 'static>(PhantomData<T>);

impl<T: CommunicationChannel<u64> + 'static> NamedConceptTest for CommunicationChannelTest<T> {
    type Sut = T;

    fn create(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Box<dyn Any> {
        let sut = <Self::Sut as CommunicationChannel<u64>>::Creator::new(name)
            .config(config)
            .create_receiver()
            .unwrap();

        Box::new(sut)
    }
}

pub struct EventTest<T: Event + 'static>(PhantomData<T>);

impl<T: Event + 'static> NamedConceptTest for EventTest<T> {
    type Sut = T;

    fn create(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Box<dyn Any> {
        let sut = <Self::Sut as Event>::ListenerBuilder::new(name)
            .config(config)
            .create()
            .unwrap();

        Box::new(sut)
    }
}

pub struct ZeroCopyConnectionTest<T: ZeroCopyConnection + 'static>(PhantomData<T>);

impl<T: ZeroCopyConnection + 'static> NamedConceptTest for ZeroCopyConnectionTest<T> {
    type Sut = T;

    fn create(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Box<dyn Any> {
        let sut = <Self::Sut as ZeroCopyConnection>::Builder::new(name)
            .config(config)
            .create_sender()
            .unwrap();

        Box::new(sut)
    }
}

pub struct StaticStorageTest<T: StaticStorage + 'static>(PhantomData<T>);

impl<T: StaticStorage + 'static> NamedConceptTest for StaticStorageTest<T> {
    type Sut = T;

    fn create(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Box<dyn Any> {
        let sut = <Self::Sut as StaticStorage>::Builder::new(name)
            .config(config)
            .has_ownership(true)
            .create(b"bla")
            .unwrap();

        Box::new(sut)
    }
}

pub struct SharedMemoryTest<T: SharedMemory<PoolAllocator> + 'static>(PhantomData<T>);

impl<T: SharedMemory<PoolAllocator> + 'static> NamedConceptTest for SharedMemoryTest<T> {
    type Sut = T;

    fn create(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Box<dyn Any> {
        type AllocatorConfig = <PoolAllocator as ShmAllocator>::Configuration;
        let sut = <Self::Sut as SharedMemory<PoolAllocator>>::Builder::new(name)
            .config(config)
            .size(1024)
            .create(&AllocatorConfig {
                bucket_layout: Layout::new::<u64>(),
            })
            .unwrap();

        Box::new(sut)
    }
}

pub struct ResizableSharedMemoryTest<
    S: SharedMemory<PoolAllocator> + 'static,
    T: ResizableSharedMemory<PoolAllocator, S> + 'static,
>(PhantomData<T>, PhantomData<S>);

impl<S: SharedMemory<PoolAllocator> + 'static, T: ResizableSharedMemory<PoolAllocator, S> + 'static>
    NamedConceptTest for ResizableSharedMemoryTest<S, T>
{
    type Sut = T;

    fn create(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Box<dyn Any> {
        let sut = <Self::Sut as ResizableSharedMemory<PoolAllocator, S>>::MemoryBuilder::new(name)
            .config(config)
            .create()
            .unwrap();

        Box::new(sut)
    }
}

pub struct MonitoringTest<T: Monitoring + 'static>(PhantomData<T>);

impl<T: Monitoring + 'static> NamedConceptTest for MonitoringTest<T> {
    type Sut = T;

    fn create(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Box<dyn Any> {
        let sut = <Self::Sut as Monitoring>::Builder::new(name)
            .config(config)
            .token()
            .unwrap();

        Box::new(sut)
    }
}

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod named_concept_trait {
    use super::*;

    #[conformance_test]
    pub fn drop_does_not_panic_when_concept_is_already_removed<T: NamedConceptTest>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<T::Sut>();

        let sut = T::create(&name, &config);
        assert_that!(unsafe { T::Sut::remove_cfg(&name, &config).unwrap() }, eq true);

        // this shall not panic
        drop(sut);
    }
}
