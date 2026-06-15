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

use std::sync::Arc;

use iceoryx2::service::{Service, local_threadsafe, static_config::StaticConfig};
use iceoryx2_services_tunnel_backend::traits::{PublishSubscribeRelay, RelayBuilder};
use iceoryx2_services_tunnel_backend::types::publish_subscribe::{LoanFn, Sample, SampleMut};
use iceoryx2_services_tunnel_backend::types::wake::WakeHandle;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SendError {}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SendError::{self:?}")
    }
}

impl core::error::Error for SendError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ReceiveError {}

impl core::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl core::error::Error for ReceiveError {}

/// Relays publish-subscribe payloads between iceoryx2 and a ROS 2 topic.
#[derive(Debug)]
pub struct Relay<S: Service> {
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> PublishSubscribeRelay<S> for Relay<S> {
    type SendError = SendError;
    type ReceiveError = ReceiveError;

    fn send(&self, _sample: Sample<S>) -> Result<(), Self::SendError> {
        todo!()
    }

    fn receive<LoanError>(
        &self,
        _loan: &mut LoanFn<'_, S, LoanError>,
    ) -> Result<Option<SampleMut<S>>, Self::ReceiveError> {
        todo!()
    }
}

/// Builder for publish-subscribe [`Relay`]s.
#[derive(Debug)]
pub struct Builder<'config, S: Service> {
    static_config: &'config StaticConfig,
    wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<'config, S: Service> Builder<'config, S> {
    pub fn new(
        static_config: &'config StaticConfig,
        wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    ) -> Self {
        Self {
            static_config,
            wake,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S: Service> RelayBuilder for Builder<'_, S> {
    type CreationError = CreationError;
    type Relay = Relay<S>;

    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        todo!()
    }
}
