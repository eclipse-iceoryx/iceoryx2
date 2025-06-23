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
use core::fmt::Debug;

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
pub struct Writer<Service: service::Service, T: Send + Sync + Debug + 'static> {
    //service: Service, or ServiceState with BlackboardResources
    map: Service::BlackboardMgmt<T>,
}

impl<Service: service::Service, T: Send + Sync + Debug + 'static> Writer<Service, T> {
    pub(crate) fn new(mgmt: Service::BlackboardMgmt<T>) -> Result<Self, WriterCreateError> {
        let new_self = Self { map: mgmt };
        Ok(new_self)
    }

    pub fn write(&self) {
        //self.map
        //.get()
        //.store(3, core::sync::atomic::Ordering::Relaxed);
    }

    // TODO: remove
    pub fn read(&self) -> u32 {
        6
        //self.map.get().load(core::sync::atomic::Ordering::Relaxed)
    }
}
