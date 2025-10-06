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

use iceoryx2::service::static_config::StaticConfig;
use iceoryx2_tunnel_backend::traits::{EventRelay, RelayBuilder};
use zenoh::Session;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    Error,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PropagationError {
    Error,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum IngestionError {
    Error,
}

#[derive(Debug)]
pub struct Builder<'a> {
    session: &'a Session,
    static_config: &'a StaticConfig,
}

impl<'a> Builder<'a> {
    pub fn new(session: &'a Session, static_config: &'a StaticConfig) -> Builder<'a> {
        Builder {
            session,
            static_config,
        }
    }
}

impl<'a> RelayBuilder for Builder<'a> {
    type CreationError = CreationError;
    type Relay = Relay;

    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Relay {}

impl EventRelay for Relay {
    type PropagationError = PropagationError;
    type IngestionError = IngestionError;

    fn propagate(&self) -> Result<(), Self::PropagationError> {
        todo!()
    }

    fn ingest(&self) -> Result<(), Self::IngestionError> {
        todo!()
    }
}
