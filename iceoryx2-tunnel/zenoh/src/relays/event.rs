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

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    Error,
}

#[derive(Debug)]
pub struct Builder {}

impl iceoryx2_tunnel_traits::RelayBuilder for Builder {
    type CreationError = CreationError;

    fn create(self) -> Result<Box<dyn iceoryx2_tunnel_traits::Relay>, Self::CreationError> {
        Ok(Box::new(Relay {}))
    }
}

pub struct Relay {}

impl iceoryx2_tunnel_traits::Relay for Relay {
    fn propagate(&self, bytes: *const u8, len: usize) {
        todo!()
    }

    fn ingest(&self, loan: &mut dyn FnMut(usize) -> (*mut u8, usize)) -> bool {
        todo!()
    }
}
