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

use iceoryx2_bb_log::fail;

use crate::port::writer::{Writer, WriterCreateError};
use crate::service;

use super::blackboard::PortFactory;

#[derive(Debug)]
pub struct PortFactoryWriter<'factory, Service: service::Service> {
    pub(crate) factory: &'factory PortFactory<Service>,
}

impl<'factory, Service: service::Service> PortFactoryWriter<'factory, Service> {
    pub(crate) fn new(factory: &'factory PortFactory<Service>) -> Self {
        Self { factory }
    }

    pub fn create(self) -> Result<Writer<'factory, Service>, WriterCreateError> {
        let origin = format!("{:?}", self);
        Ok(
            fail!(from origin, when Writer::new(&self.factory.mgmt),"Failed to create new Writer port."),
        )
    }
}
