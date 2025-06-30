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

// TODO: example

use super::blackboard::PortFactory;
use crate::port::writer::{Writer, WriterCreateError};
use crate::service;
use crate::service::builder::blackboard::Mgmt;
use crate::service::config_scheme::{blackboard_data_config, blackboard_mgmt_config};
use core::fmt::Debug;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_lock_free::spmc::unrestricted_atomic::UnrestrictedAtomic;
use iceoryx2_bb_log::fail;
use iceoryx2_cal::dynamic_storage::DynamicStorageBuilder;
use iceoryx2_cal::event::{NamedConcept, NamedConceptBuilder};
use iceoryx2_cal::shared_memory::{SharedMemory, SharedMemoryBuilder};

#[derive(Debug)]
pub struct PortFactoryWriter<
    'factory,
    Service: service::Service,
    T: Send + Sync + Debug + 'static + Eq + ZeroCopySend + Clone,
> {
    pub(crate) factory: &'factory PortFactory<Service, T>,
}

impl<
        'factory,
        Service: service::Service,
        T: Send + Sync + Debug + 'static + Eq + ZeroCopySend + Clone,
    > PortFactoryWriter<'factory, Service, T>
{
    pub(crate) fn new(factory: &'factory PortFactory<Service, T>) -> Self {
        Self { factory }
    }

    pub fn create(self) -> Result<Writer<Service, T>, WriterCreateError> {
        let origin = format!("{:?}", self);

        let name = self.factory.service.additional_resource.mgmt.name();

        //// test to open payload data segment
        let shm_config =
            blackboard_data_config::<Service, Mgmt<T>>(self.factory.service.shared_node.config());
        let payload_shm =
            <<Service::BlackboardPayload as iceoryx2_cal::shared_memory::SharedMemory<
                iceoryx2_cal::shm_allocator::bump_allocator::BumpAllocator,
            >>::Builder as NamedConceptBuilder<Service::BlackboardPayload>>::new(&name)
            .config(&shm_config)
            .open()
            .unwrap();
        let atomic = (payload_shm.payload_start_address()) as *mut UnrestrictedAtomic<u64>;
        let value = unsafe { &(*atomic) }.load();
        println!("PortFactoryWriter: value = {}", value);
        ////

        let mgmt_config =
            blackboard_mgmt_config::<Service, Mgmt<T>>(self.factory.service.shared_node.config());
        // TODO: error type and message
        let storage = fail!(from origin,
            when <Service::BlackboardMgmt<Mgmt<T>> as iceoryx2_cal::dynamic_storage::DynamicStorage<Mgmt<T>>>::Builder::new(name)
                .config(&mgmt_config)
                .has_ownership(false)
                .open(),
            with WriterCreateError::ExceedsMaxSupportedWriters,
            "blub");
        Ok(fail!(from origin, when Writer::new(storage),"Failed to create new Writer port."))
    }
}
