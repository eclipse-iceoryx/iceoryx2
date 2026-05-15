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

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use core::alloc::Layout;
use core::any::Any;
use core::marker::PhantomData;
use core::time::Duration;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_posix::file::AccessMode;
use iceoryx2_bb_posix::testing::generate_file_path;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::conformance_test;
use iceoryx2_bb_testing_macros::conformance_tests;
use iceoryx2_cal::communication_channel::CommunicationChannel;
use iceoryx2_cal::communication_channel::CommunicationChannelConnector;
use iceoryx2_cal::communication_channel::CommunicationChannelCreator;
use iceoryx2_cal::event::Event;
use iceoryx2_cal::event::ListenerBuilder;
use iceoryx2_cal::event::NotifierBuilder;
use iceoryx2_cal::monitoring::Monitoring;
use iceoryx2_cal::monitoring::MonitoringBuilder;
use iceoryx2_cal::resizable_shared_memory::ResizableSharedMemory;
use iceoryx2_cal::resizable_shared_memory::ResizableSharedMemoryBuilder;
use iceoryx2_cal::resizable_shared_memory::ResizableSharedMemoryViewBuilder;
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
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>>;

    fn open(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>>;
}

pub struct DynamicStorageTest<T: DynamicStorage<u64> + 'static>(PhantomData<T>);

impl<T: DynamicStorage<u64>> NamedConceptTest for DynamicStorageTest<T> {
    type Sut = T;
    fn create(
        name: &FileName,
        config: &T::Configuration,
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>> {
        let sut = <Self::Sut as DynamicStorage<u64>>::Builder::new(name)
            .config(config)
            .has_ownership(true)
            .create(123)?;

        Ok(Box::new(sut))
    }

    fn open(
        name: &FileName,
        config: &T::Configuration,
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>> {
        let sut = <Self::Sut as DynamicStorage<u64>>::Builder::new(name)
            .config(config)
            .open(AccessMode::Read)?;

        Ok(Box::new(sut))
    }
}

pub struct CommunicationChannelTest<T: CommunicationChannel<u64> + 'static>(PhantomData<T>);

impl<T: CommunicationChannel<u64> + 'static> NamedConceptTest for CommunicationChannelTest<T> {
    type Sut = T;

    fn create(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>> {
        let sut = <Self::Sut as CommunicationChannel<u64>>::Creator::new(name)
            .config(config)
            .create_receiver()?;

        Ok(Box::new(sut))
    }

    fn open(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>> {
        let sut = <Self::Sut as CommunicationChannel<u64>>::Connector::new(name)
            .config(config)
            .open_sender()?;

        Ok(Box::new(sut))
    }
}

pub struct EventTest<T: Event + 'static>(PhantomData<T>);

impl<T: Event + 'static> NamedConceptTest for EventTest<T> {
    type Sut = T;

    fn create(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>> {
        let sut = <Self::Sut as Event>::ListenerBuilder::new(name)
            .config(config)
            .create()?;

        Ok(Box::new(sut))
    }

    fn open(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>> {
        let sut = <Self::Sut as Event>::NotifierBuilder::new(name)
            .config(config)
            .open()?;

        Ok(Box::new(sut))
    }
}

pub struct ZeroCopyConnectionTest<T: ZeroCopyConnection + 'static>(PhantomData<T>);

impl<T: ZeroCopyConnection + 'static> NamedConceptTest for ZeroCopyConnectionTest<T> {
    type Sut = T;

    fn create(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>> {
        let sut = <Self::Sut as ZeroCopyConnection>::Builder::new(name)
            .config(config)
            .create_sender()?;

        Ok(Box::new(sut))
    }

    fn open(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>> {
        if Self::Sut::does_exist_cfg(name, config)? {
            let sut = <Self::Sut as ZeroCopyConnection>::Builder::new(name)
                .config(config)
                .create_sender()?;
            Ok(Box::new(sut))
        } else {
            Err("No such connection".into())
        }
    }
}

pub struct StaticStorageTest<T: StaticStorage + 'static>(PhantomData<T>);

impl<T: StaticStorage + 'static> NamedConceptTest for StaticStorageTest<T> {
    type Sut = T;

    fn create(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>> {
        let sut = <Self::Sut as StaticStorage>::Builder::new(name)
            .config(config)
            .has_ownership(true)
            .create(b"bla")?;

        Ok(Box::new(sut))
    }

    fn open(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>> {
        let sut = <Self::Sut as StaticStorage>::Builder::new(name)
            .config(config)
            .open(Duration::from_secs(0))?;

        Ok(Box::new(sut))
    }
}

pub struct SharedMemoryTest<T: SharedMemory<PoolAllocator> + 'static>(PhantomData<T>);

impl<T: SharedMemory<PoolAllocator> + 'static> NamedConceptTest for SharedMemoryTest<T> {
    type Sut = T;

    fn create(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>> {
        type AllocatorConfig = <PoolAllocator as ShmAllocator>::Configuration;
        let sut = <Self::Sut as SharedMemory<PoolAllocator>>::Builder::new(name)
            .config(config)
            .size(1024)
            .create(&AllocatorConfig {
                bucket_layout: Layout::new::<u64>(),
            })?;

        Ok(Box::new(sut))
    }

    fn open(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>> {
        let sut = <Self::Sut as SharedMemory<PoolAllocator>>::Builder::new(name)
            .config(config)
            .size(1024)
            .open(AccessMode::Read)?;

        Ok(Box::new(sut))
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
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>> {
        let sut = <Self::Sut as ResizableSharedMemory<PoolAllocator, S>>::MemoryBuilder::new(name)
            .config(config)
            .create()?;

        Ok(Box::new(sut))
    }

    fn open(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>> {
        let sut = <Self::Sut as ResizableSharedMemory<PoolAllocator, S>>::ViewBuilder::new(name)
            .config(config)
            .open(AccessMode::Read)?;

        Ok(Box::new(sut))
    }
}

pub struct MonitoringTest<T: Monitoring + 'static>(PhantomData<T>);

impl<T: Monitoring + 'static> NamedConceptTest for MonitoringTest<T> {
    type Sut = T;

    fn create(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>> {
        let sut = <Self::Sut as Monitoring>::Builder::new(name)
            .config(config)
            .token()?;

        Ok(Box::new(sut))
    }

    fn open(
        name: &FileName,
        config: &<Self::Sut as NamedConceptMgmt>::Configuration,
    ) -> Result<Box<dyn Any>, Box<dyn core::error::Error>> {
        let sut = <Self::Sut as Monitoring>::Builder::new(name)
            .config(config)
            .cleaner()?;

        Ok(Box::new(sut))
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

        let sut = T::create(&name, &config).unwrap();
        assert_that!(unsafe { T::Sut::remove_cfg(&name, &config).unwrap() }, eq true);

        // this shall not panic
        drop(sut);
    }

    #[conformance_test]
    pub fn does_exist_works<T: NamedConceptTest>() {
        const NUMBER_OF_ENTITIES: usize = 11;
        let config = generate_isolated_config::<T::Sut>();

        let mut suts = vec![];
        let mut names = vec![];

        for _ in 0..NUMBER_OF_ENTITIES {
            let storage_name = generate_file_path().file_name();
            let sut = T::create(&storage_name, &config).unwrap();
            names.push(storage_name);
            suts.push(sut);
        }

        for name in names {
            assert_that!(T::Sut::does_exist_cfg(&name, &config), eq Ok(true));
        }
    }

    #[conformance_test]
    pub fn list_and_does_exist_works<T: NamedConceptTest>() {
        let mut sut_names = vec![];
        const LIMIT: usize = 8;
        let config = generate_isolated_config::<T::Sut>();
        let mut entities = vec![];

        for i in 0..LIMIT {
            assert_that!(<T::Sut as NamedConceptMgmt>::list_cfg(&config).unwrap(), len i);
            sut_names.push(generate_file_path().file_name());
            assert_that!(<T::Sut as NamedConceptMgmt>::does_exist_cfg(&sut_names[i], &config), eq Ok(false));

            entities.push(T::create(&sut_names[i], &config).unwrap());
            assert_that!(<T::Sut as NamedConceptMgmt>::does_exist_cfg(&sut_names[i], &config), eq Ok(true));

            let list = <T::Sut as NamedConceptMgmt>::list_cfg(&config).unwrap();
            assert_that!(list, len i + 1);
            let does_exist_in_list = |value| {
                for e in &list {
                    if e == value {
                        return true;
                    }
                }
                false
            };

            for name in &sut_names {
                assert_that!(does_exist_in_list(name), eq true);
            }
        }

        assert_that!(<T::Sut as NamedConceptMgmt>::list_cfg(&config).unwrap(), len LIMIT);

        for sut_name in sut_names.iter().take(LIMIT) {
            assert_that!(unsafe{<T::Sut as NamedConceptMgmt>::remove_cfg(sut_name, &config)}, eq Ok(true));
            assert_that!(unsafe{<T::Sut as NamedConceptMgmt>::remove_cfg(sut_name, &config)}, eq Ok(false));
        }

        assert_that!(<T::Sut as NamedConceptMgmt>::list_cfg(&config).unwrap(), len 0);
    }

    #[conformance_test]
    pub fn custom_suffix_ensures_separation<T: NamedConceptTest>() {
        let config = generate_isolated_config::<T::Sut>();
        let config_1 = unsafe { config.clone().suffix(&FileName::new_unchecked(b".s1")) };
        let config_2 = unsafe { config.suffix(&FileName::new_unchecked(b".s2")) };

        let sut_name = generate_file_path().file_name();

        assert_that!(<T::Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(false));
        assert_that!(<T::Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(false));
        assert_that!(<T::Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 0);
        assert_that!(<T::Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let sut_1 = T::create(&sut_name, &config_1).unwrap();

        assert_that!(<T::Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(true));
        assert_that!(<T::Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(false));
        assert_that!(<T::Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<T::Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let sut_2 = T::create(&sut_name, &config_2).unwrap();

        assert_that!(<T::Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(true));
        assert_that!(<T::Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(true));
        assert_that!(<T::Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<T::Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 1);

        assert_that!(<T::Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap()[0], eq sut_name);
        assert_that!(<T::Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap()[0], eq sut_name);

        core::mem::forget(sut_1);
        core::mem::forget(sut_2);

        assert_that!(unsafe {<T::Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_1)}, eq Ok(true));
        assert_that!(unsafe {<T::Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_1)}, eq Ok(false));
        assert_that!(unsafe {<T::Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_2)}, eq Ok(true));
        assert_that!(unsafe {<T::Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_2)}, eq Ok(false));
    }

    #[conformance_test]
    pub fn defaults_for_configuration_are_set_correctly<T: NamedConceptTest>() {
        let config = <T::Sut as NamedConceptMgmt>::Configuration::default();
        assert_that!(*config.get_path_hint(), eq T::Sut::default_path_hint());
        assert_that!(*config.get_prefix(), eq T::Sut::default_prefix());
    }

    #[conformance_test]
    pub fn remove_works<T: NamedConceptTest>() {
        const NUMBER_OF_ENTITIES: usize = 26;
        let config = generate_isolated_config::<T::Sut>();

        let mut names = vec![];

        for _ in 0..NUMBER_OF_ENTITIES {
            let storage_name = generate_file_path().file_name();
            assert_that!(unsafe { T::Sut::remove_cfg(&storage_name, &config) }, eq Ok(false));
            let sut = T::create(&storage_name, &config).unwrap();
            core::mem::forget(sut);
            names.push(storage_name);
        }

        for name in names {
            assert_that!(T::Sut::does_exist_cfg(&name, &config), eq Ok(true));
            assert_that!(unsafe { T::Sut::remove_cfg(&name, &config) }, eq Ok(true));
            assert_that!(unsafe { T::Sut::remove_cfg(&name, &config) }, eq Ok(false));
            assert_that!(T::Sut::does_exist_cfg(&name, &config), eq Ok(false));
        }
    }

    #[conformance_test]
    pub fn cannot_be_created_twice<T: NamedConceptTest>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<T::Sut>();

        let _sut = T::create(&name, &config).unwrap();
        assert_that!(T::create(&name, &config), is_err);
    }

    #[conformance_test]
    pub fn cannot_open_non_existing<T: NamedConceptTest>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<T::Sut>();

        assert_that!(T::open(&name, &config), is_err);
    }
}
