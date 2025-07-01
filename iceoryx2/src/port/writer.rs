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

use crate::service;
use crate::service::builder::blackboard::Mgmt;
use core::fmt::Debug;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_lock_free::spmc::unrestricted_atomic::UnrestrictedAtomic;
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::shared_memory::SharedMemory;

/// Defines a failure that can occur when a [`Writer`] is created with
/// [`crate::service::port_factory::writer::PortFactoryWriter`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WriterCreateError {
    /// The maximum amount of [`Writer`]s that can connect to a
    /// [`Service`](crate::service::Service) is
    /// defined in [`crate::config::Config`]. When this is exceeded no more [`Writer`]s
    /// can be created for a specific [`Service`](crate::service::Service).
    ExceedsMaxSupportedWriters,
}

impl core::fmt::Display for WriterCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "WriterCreateError::{:?}", self)
    }
}

impl core::error::Error for WriterCreateError {}

/// Producing endpoint of a blackboard based communication.
#[derive(Debug)]
pub struct Writer<
    Service: service::Service,
    T: Send + Sync + Debug + 'static + Eq + ZeroCopySend + Clone,
> {
    mgmt: Service::BlackboardMgmt<Mgmt<T>>,
    payload: Service::BlackboardPayload,
}

impl<Service: service::Service, T: Send + Sync + Debug + 'static + Eq + ZeroCopySend + Clone>
    Writer<Service, T>
{
    pub(crate) fn new(
        mgmt: Service::BlackboardMgmt<Mgmt<T>>,
        payload: Service::BlackboardPayload,
    ) -> Result<Self, WriterCreateError> {
        // TODO: error handling
        let new_self = Self { mgmt, payload };
        Ok(new_self)
    }

    pub fn update_with_copy<ValueType: Copy>(&self, key: &T, value: ValueType) {
        let index = self.mgmt.get().map.get(key);
        if index.is_some() {
            let offset = self.mgmt.get().entries[index.unwrap()]
                .offset
                .load(core::sync::atomic::Ordering::Relaxed);
            let atomic = (self.payload.payload_start_address() as u64 + offset)
                as *mut UnrestrictedAtomic<ValueType>;
            // TODO: error handling (see UnrestrictedAtomic example)
            unsafe {
                (*atomic).acquire_producer().unwrap().store(value);
            };
        }
    }
}
