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

impl<'a> iceoryx2_tunnel_traits::RelayBuilder for Builder<'a> {
    type CreationError = CreationError;
    type Relay = Relay;

    fn create(
        self,
    ) -> Result<
        Box<
            dyn iceoryx2_tunnel_traits::Relay<
                PropagationError = <Self::Relay as iceoryx2_tunnel_traits::Relay>::PropagationError,
                IngestionError = <Self::Relay as iceoryx2_tunnel_traits::Relay>::IngestionError,
            >,
        >,
        Self::CreationError,
    > {
        todo!()
    }
}

pub struct Relay {}

impl iceoryx2_tunnel_traits::Relay for Relay {
    type PropagationError = PropagationError;
    type IngestionError = IngestionError;

    fn propagate(&self, bytes: *const u8, len: usize) -> Result<(), Self::PropagationError> {
        todo!()
    }

    fn ingest(
        &self,
        loan: &mut dyn FnMut(usize) -> (*mut u8, usize),
    ) -> Result<bool, Self::IngestionError> {
        todo!()
    }
}
