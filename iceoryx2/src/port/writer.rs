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
use core::{fmt::Debug, marker::PhantomData, sync::atomic::AtomicU32};
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_cal::dynamic_storage::DynamicStorage;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WriterCreateError {
    ExceedsMaxSupportedWriters,
}

impl core::fmt::Display for WriterCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "WriterCreateError::{:?}", self)
    }
}

impl core::error::Error for WriterCreateError {}

#[derive(Debug)]
pub struct Writer<
    Service: service::Service,
    T: Send + Sync + Debug + 'static + Eq + ZeroCopySend + Clone,
> {
    //service: Service, or ServiceState with BlackboardResources
    map: Service::BlackboardMgmt<Mgmt<T>>,
}

impl<Service: service::Service, T: Send + Sync + Debug + 'static + Eq + ZeroCopySend + Clone>
    Writer<Service, T>
{
    pub(crate) fn new(mgmt: Service::BlackboardMgmt<Mgmt<T>>) -> Result<Self, WriterCreateError> {
        let new_self = Self { map: mgmt };
        Ok(new_self)
    }

    pub fn write(&self, value: u64) {
        let entry = self.map.get();
        entry.entries[0]
            .offset
            .store(value, core::sync::atomic::Ordering::Relaxed);
    }
}
