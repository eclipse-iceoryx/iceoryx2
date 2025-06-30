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
use iceoryx2_cal::dynamic_storage::DynamicStorage;

/// Defines a failure that can occur when a [`Reader`] is created with
/// [`crate::service::port_factory::reader::PortFactoryReader`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReaderCreateError {
    /// The maximum amount of [`Reader`]s that can connect to a
    /// [`Service`](crate::service::Service) is
    /// defined in [`crate::config::Config`]. When this is exceeded no more [`Reader`]s
    /// can be created for a specific [`Service`](crate::service::Service).
    ExceedsMaxSupportedReaders,
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
    //service: Service, or ServiceState with BlackboardResources
    map: Service::BlackboardMgmt<Mgmt<T>>,
}

impl<Service: service::Service, T: Send + Sync + Debug + 'static + Eq + ZeroCopySend + Clone>
    Reader<Service, T>
{
    pub(crate) fn new(mgmt: Service::BlackboardMgmt<Mgmt<T>>) -> Result<Self, ReaderCreateError> {
        let new_self = Self { map: mgmt };
        Ok(new_self)
    }

    pub fn read<ValueType>(&self, key: &T) -> Option<u64> {
        let entries_index = self.map.get().map.get(key);
        if entries_index.is_none() {
            return None;
        }
        let offset = self.map.get().entries[entries_index.unwrap()]
            .offset
            .load(core::sync::atomic::Ordering::Relaxed);
        Some(offset)
    }
}
