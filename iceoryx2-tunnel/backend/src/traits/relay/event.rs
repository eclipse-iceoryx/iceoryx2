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

use core::error::Error;

use iceoryx2::{prelude::EventId, service::Service};

/// Relay for exchanging events between iceoryx and the backend.
pub trait EventRelay<S: Service> {
    /// Error type returned when sending an event fails
    type SendError: Error;
    /// Error type returned when receiving an event fails
    type ReceiveError: Error;

    /// Send an event with the specified ID via the backend communication
    /// mechanism.
    fn send(&self, event_id: EventId) -> Result<(), Self::SendError>;

    /// Receive an event via the backend communication mechanism.
    fn receive(&self) -> Result<Option<EventId>, Self::ReceiveError>;
}
