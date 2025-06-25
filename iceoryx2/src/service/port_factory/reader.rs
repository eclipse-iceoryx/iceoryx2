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
use crate::service::config_scheme::blackboard_mgmt_data_segment_config;
use core::fmt::Debug;
use core::sync::atomic::AtomicU32;
use iceoryx2_bb_log::fail;
use iceoryx2_cal::dynamic_storage::DynamicStorageBuilder;
use iceoryx2_cal::event::{NamedConcept, NamedConceptBuilder};

#[derive(Debug)]
pub struct PortFactoryReader<'factory, Service: service::Service, T: Send + Sync + Debug + 'static>
{
    pub(crate) factory: &'factory PortFactory<Service, T>,
}

impl<'factory, Service: service::Service, T: Send + Sync + Debug + 'static>
    PortFactoryReader<'factory, Service, T>
{
    pub(crate) fn new(factory: &'factory PortFactory<Service, T>) -> Self {
        Self { factory }
    }

    pub fn create(self) -> Result<Reader<Service, AtomicU32>, ReaderCreateError> {
        let origin = format!("{:?}", self);

        let mgmt_name = self.factory.service.additional_resource.mgmt.name();
        let mgmt_config = blackboard_mgmt_data_segment_config::<Service, AtomicU32>(
            self.factory.service.shared_node.config(),
        );
        // TODO: error type and message
        let storage = fail!(from origin,
            when <Service::BlackboardMgmt<AtomicU32> as iceoryx2_cal::dynamic_storage::DynamicStorage<AtomicU32>>::Builder::new(mgmt_name)
                .config(&mgmt_config)
                .has_ownership(false)
                .open(),
            with ReaderCreateError::ExceedsMaxSupportedReaders,
            "blub");
        Ok(fail!(from origin, when Reader::new(storage),"Failed to create new Reader port."))
    }
}
