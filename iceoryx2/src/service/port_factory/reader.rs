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
use core::fmt::Debug;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fail;

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
        Ok(
            fail!(from origin, when Reader::new(self.factory.service.clone()),"Failed to create new Reader port."),
        )
    }
}
