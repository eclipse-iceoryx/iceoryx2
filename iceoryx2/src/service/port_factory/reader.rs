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
use crate::port::reader::{Reader, ReaderCreateError};
use crate::service;
use crate::service::builder::blackboard::Mgmt;
use crate::service::config_scheme::{blackboard_data_config, blackboard_mgmt_config};
use core::fmt::Debug;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fail;
use iceoryx2_cal::dynamic_storage::{DynamicStorage, DynamicStorageBuilder};
use iceoryx2_cal::event::{NamedConcept, NamedConceptBuilder};
use iceoryx2_cal::shared_memory::{SharedMemory, SharedMemoryBuilder};
use iceoryx2_cal::shm_allocator::bump_allocator::BumpAllocator;

/// Factory to create a new [`Reader`] port/endpoint for
/// [`MessagingPattern::Blackboard`](crate::service::messaging_pattern::MessagingPattern::Blackboard)
/// based communication.
#[derive(Debug)]
pub struct PortFactoryReader<
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
    > PortFactoryReader<'factory, Service, T>
{
    pub(crate) fn new(factory: &'factory PortFactory<Service, T>) -> Self {
        Self { factory }
    }

    /// Creates a new [`Reader`] or returns a [`ReaderCreateError`] on failure.
    pub fn create(self) -> Result<Reader<Service, T>, ReaderCreateError> {
        let origin = format!("{:?}", self);

        let name = self.factory.service.additional_resource.mgmt.name();

        // open payload data segment
        let shm_config =
            blackboard_data_config::<Service, Mgmt<T>>(self.factory.service.shared_node.config());
        let payload_shm =
            <<Service::BlackboardPayload as SharedMemory<BumpAllocator,
            >>::Builder as NamedConceptBuilder<Service::BlackboardPayload>>::new(&name)
            .config(&shm_config)
            .open()
            .unwrap();

        // open management segment
        let mgmt_config =
            blackboard_mgmt_config::<Service, Mgmt<T>>(self.factory.service.shared_node.config());
        // TODO: error type and message
        let mgmt_storage = fail!(from origin,
            when <Service::BlackboardMgmt<Mgmt<T>> as DynamicStorage<Mgmt<T>>>::Builder::new(name)
                .config(&mgmt_config)
                .has_ownership(false)
                .open(),
            with ReaderCreateError::ExceedsMaxSupportedReaders,
            "blub");
        Ok(
            fail!(from origin, when Reader::new(mgmt_storage, payload_shm),"Failed to create new Reader port."),
        )
    }
}
