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

use core::fmt::Debug;

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
pub trait RelayBuilder {
    type Error: Debug;

    fn create(self) -> Result<Box<dyn Relay>, Self::Error>;
}

/// Retrieve the specific builder for different messaging patterns.
///
/// This also defines the messaging patterns which the transport must
/// support.
pub trait RelayFactory<T: Transport> {
    type PublishSubscribeBuilder: RelayBuilder + Debug;
    type EventBuilder: RelayBuilder + Debug;

    fn publish_subscribe(&self, service: &str) -> Self::PublishSubscribeBuilder;

    fn event(&self, service: &str) -> Self::EventBuilder;
}
