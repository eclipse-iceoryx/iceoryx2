// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

use alloc::rc::Rc;
use iceoryx2::{
    port::event_id::EventId,
    service::{Service, service_hash::ServiceHash},
};
use iceoryx2_services_tunnel_backend::traits::{EventRelay, RelayBuilder};
use iceoryx2_services_tunnel_backend::types::service_description::ServiceDescription;

use crate::backend::session::{self, Session};

#[derive(Debug)]
pub enum CreationError {}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug)]
pub enum SendError {
    SendEvent(session::SendError),
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SendError::{self:?}")
    }
}

impl core::error::Error for SendError {}

#[derive(Debug)]
pub enum ReceiveError {
    ReceiveEvent(session::ReceiveError),
}

impl core::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl core::error::Error for ReceiveError {}

#[derive(Debug)]
pub struct Builder<'a, S: Service> {
    session: Rc<Session>,
    description: &'a ServiceDescription,
    _phantom: core::marker::PhantomData<S>,
}

impl<'a, S: Service> Builder<'a, S> {
    pub fn new(session: Rc<Session>, description: &'a ServiceDescription) -> Self {
        Self {
            session,
            description,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S: Service> RelayBuilder for Builder<'_, S> {
    type CreationError = CreationError;
    type Relay = Relay<S>;

    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        Ok(Relay {
            session: self.session,
            service_hash: self.description.service_hash,
            _phantom: core::marker::PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct Relay<S: Service> {
    session: Rc<Session>,
    service_hash: ServiceHash,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> EventRelay<S> for Relay<S> {
    type SendError = SendError;
    type ReceiveError = ReceiveError;

    fn send(&self, event_id: iceoryx2::prelude::EventId) -> Result<(), Self::SendError> {
        self.session
            .send_event(&self.service_hash, event_id.as_value() as u64)
            .map_err(SendError::SendEvent)
    }

    fn receive(&self) -> Result<Option<iceoryx2::prelude::EventId>, Self::ReceiveError> {
        Ok(self
            .session
            .recv_event(&self.service_hash)
            .map_err(ReceiveError::ReceiveEvent)?
            .map(|id| EventId::new(id as usize)))
    }
}
