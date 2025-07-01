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

use crate::service::builder::blackboard::{BlackboardResources, Mgmt};
use crate::service::config_scheme::{blackboard_data_config, blackboard_mgmt_config};
use crate::service::dynamic_config::blackboard::ReaderDetails;
use crate::service::{self, ServiceState};
use core::fmt::Debug;
use core::sync::atomic::Ordering;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_lock_free::mpmc::container::ContainerHandle;
use iceoryx2_bb_lock_free::spmc::unrestricted_atomic::UnrestrictedAtomic;
use iceoryx2_bb_log::fail;
use iceoryx2_cal::dynamic_storage::{DynamicStorage, DynamicStorageBuilder};
use iceoryx2_cal::event::{NamedConcept, NamedConceptBuilder};
use iceoryx2_cal::shared_memory::{SharedMemory, SharedMemoryBuilder};
use iceoryx2_cal::shm_allocator::bump_allocator::BumpAllocator;

extern crate alloc;
use alloc::sync::Arc;

use super::port_identifiers::UniqueReaderId;

/// Defines a failure that can occur when a [`Reader`] is created with
/// [`crate::service::port_factory::reader::PortFactoryReader`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReaderCreateError {
    /// The maximum amount of [`Reader`]s that can connect to a
    /// [`Service`](crate::service::Service) is
    /// defined in [`crate::config::Config`]. When this is exceeded no more [`Reader`]s
    /// can be created for a specific [`Service`](crate::service::Service).
    ExceedsMaxSupportedReaders,
    /// The data segment could not be opened.
    UnableToOpenDataSegment,
}

impl core::fmt::Display for ReaderCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "ReaderCreateError::{:?}", self)
    }
}

impl core::error::Error for ReaderCreateError {}

/// Reading endpoint of a blackboard based communication.
#[derive(Debug)]
pub struct Reader<
    Service: service::Service,
    T: Send + Sync + Debug + 'static + Eq + ZeroCopySend + Clone,
> {
    service_state: Arc<ServiceState<Service, BlackboardResources<Service, T>>>,
    mgmt: Service::BlackboardMgmt<Mgmt<T>>,
    payload: Service::BlackboardPayload,
    dynamic_reader_handle: Option<ContainerHandle>,
}

impl<Service: service::Service, T: Send + Sync + Debug + 'static + Eq + ZeroCopySend + Clone> Drop
    for Reader<Service, T>
{
    fn drop(&mut self) {
        if let Some(handle) = self.dynamic_reader_handle {
            self.service_state
                .dynamic_storage
                .get()
                .blackboard()
                .release_reader_handle(handle)
        }
    }
}

impl<Service: service::Service, T: Send + Sync + Debug + 'static + Eq + ZeroCopySend + Clone>
    Reader<Service, T>
{
    pub(crate) fn new(
        service: Arc<ServiceState<Service, BlackboardResources<Service, T>>>,
    ) -> Result<Self, ReaderCreateError> {
        let origin = "Reader::new()";
        let msg = "Unable to create Reader port";

        // open payload data segment
        let name = service.additional_resource.mgmt.name();
        let shm_config = blackboard_data_config::<Service, Mgmt<T>>(service.shared_node.config());
        let payload_shm = fail!(from origin,
            when <<Service::BlackboardPayload as SharedMemory<BumpAllocator>
                >::Builder as NamedConceptBuilder<Service::BlackboardPayload>>::new(&name)
                .config(&shm_config)
                .open(),
            with ReaderCreateError::UnableToOpenDataSegment,
            "{} since the payload data segment could not be opened.", msg);

        // open management segment
        let mgmt_config = blackboard_mgmt_config::<Service, Mgmt<T>>(service.shared_node.config());
        let mgmt_storage = fail!(from origin,
            when <Service::BlackboardMgmt<Mgmt<T>> as DynamicStorage<Mgmt<T>>>::Builder::new(&name)
                .config(&mgmt_config)
                .has_ownership(false)
                .open(),
            with ReaderCreateError::UnableToOpenDataSegment,
            "{} since the management data segment could not be opened.", msg);

        let mut new_self = Self {
            service_state: service.clone(),
            mgmt: mgmt_storage,
            payload: payload_shm,
            dynamic_reader_handle: None,
        };

        core::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a reader is added to the dynamic config without the
        // creation of all required resources
        let dynamic_reader_handle = match service.dynamic_storage.get().blackboard().add_reader_id(
            ReaderDetails {
                reader_id: UniqueReaderId::new(),
                node_id: *service.shared_node.id(),
            },
        ) {
            Some(unique_index) => unique_index,
            None => {
                fail!(from origin, with ReaderCreateError::ExceedsMaxSupportedReaders,
                            "{} since it would exceed the maximum supported amount of readers of {}.",
                            msg, service.static_config.blackboard().max_readers);
            }
        };

        new_self.dynamic_reader_handle = Some(dynamic_reader_handle);

        Ok(new_self)
    }

    pub fn read<ValueType: Copy>(&self, key: &T) -> Option<ValueType> {
        let index = self.mgmt.get().map.get(key);
        if index.is_none() {
            return None;
        }
        let offset = self.mgmt.get().entries[index.unwrap()]
            .offset
            .load(core::sync::atomic::Ordering::Relaxed);
        let atomic = (self.payload.payload_start_address() as u64 + offset)
            as *mut UnrestrictedAtomic<ValueType>;
        let value = unsafe { (*atomic).load() };
        Some(value)
    }
}
