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

use iceoryx2::service::{Service, static_config::StaticConfig};
use iceoryx2_services_tunnel_backend::traits::{PublishSubscribeRelay, RelayBuilder};

#[derive(Debug)]
pub enum CreationError {}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug)]
pub enum SendError {}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SendError::{self:?}")
    }
}

impl core::error::Error for SendError {}

#[derive(Debug)]
pub enum ReceiveError {}

impl core::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl core::error::Error for ReceiveError {}

#[derive(Debug)]
pub struct Builder<'a, S: Service> {
    static_config: &'a StaticConfig,
    _phantom: core::marker::PhantomData<S>,
}

impl<'a, S: Service> Builder<'a, S> {
    pub fn new(static_config: &'a StaticConfig) -> Self {
        Self {
            static_config,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S: Service> RelayBuilder for Builder<'_, S> {
    type CreationError = CreationError;
    type Relay = Relay<S>;

    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        Ok(Relay {
            static_config: self.static_config.clone(),
            _phantom: core::marker::PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct Relay<S: Service> {
    static_config: StaticConfig,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> PublishSubscribeRelay<S> for Relay<S> {
    type SendError = SendError;

    type ReceiveError = ReceiveError;

    fn send(
        &self,
        sample: iceoryx2_services_tunnel_backend::types::publish_subscribe::Sample<S>,
    ) -> Result<(), Self::SendError> {
        todo!()
    }

    fn receive<LoanError>(
        &self,
        loan: &mut iceoryx2_services_tunnel_backend::types::publish_subscribe::LoanFn<
            '_,
            S,
            LoanError,
        >,
    ) -> Result<
        Option<iceoryx2_services_tunnel_backend::types::publish_subscribe::SampleMut<S>>,
        Self::ReceiveError,
    > {
        todo!()
    }
}
