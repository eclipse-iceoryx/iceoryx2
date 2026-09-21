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

use core::error::Error;

use iceoryx2::port::event_id::EventId;
use iceoryx2_link_backend::wire::event::NotAnEventId;

/// A channel to carry one service's events between local and remote peers.
pub trait EventChannel {
    type Error: Error;

    /// Sends one id to every peer connected to the channel.
    fn send(&mut self, id: EventId) -> Result<(), Self::Error>;

    /// Receives the next pending id, or returns `None` if nothing is
    /// pending. Bytes that are not an id should be dropped and return error.
    fn receive(&mut self) -> Result<Option<EventId>, EventReceiveError<Self::Error>>;
}

/// Reasons for receiving an id may fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventReceiveError<Failure> {
    /// The bytes are not an id, and are dropped.
    Malformed,
    /// The channel failed.
    Failed(Failure),
}

impl<Failure: core::fmt::Display> core::fmt::Display for EventReceiveError<Failure> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Malformed => write!(f, "EventReceiveError::Malformed"),
            Self::Failed(error) => write!(f, "EventReceiveError::Failed({error})"),
        }
    }
}

impl<Failure: Error> Error for EventReceiveError<Failure> {}

impl<Failure> From<NotAnEventId> for EventReceiveError<Failure> {
    fn from(_: NotAnEventId) -> Self {
        Self::Malformed
    }
}
