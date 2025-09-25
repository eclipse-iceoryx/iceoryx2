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

use crate::Transport;

/// Relays data between iceoryx2 and the transport.
///
/// A relay should be created for each messaging pattern.
pub trait Relay {
    fn propagate(&mut self, data: *const u8, len: usize, align: usize);
    fn ingest(&mut self, data: *mut u8, len: usize, align: usize);
}

/// Builds a relay for a specific messaging pattern.
///
/// Allows for the transport to decide how to support
/// propagation of data on different messaging patterns.
pub trait RelayBuilder<T: Transport> {
    type CreationError;

    fn create(self) -> Result<Box<dyn Relay>, Self::CreationError>;
}

/// Retrieve the specific builder for different messaging patterns.
///
/// This also defines the messaging patterns which the transport must
/// support.
pub trait RelayFactory<T: Transport> {
    fn publish_subscribe<'a>(
        &'a self,
        service: &'a str,
        config: &'a T::PublishSubscribeConfig,
    ) -> impl RelayBuilder<T> + 'a
    where
        Self: 'a;

    fn event<'a>(
        &'a self,
        service: &'a str,
        config: &'a T::EventConfig,
    ) -> impl RelayBuilder<T> + 'a
    where
        Self: 'a;
}
