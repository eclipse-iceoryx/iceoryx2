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
use iceoryx2::service::Service;

pub trait EventRelay<S: Service> {
    type SendError: Error;
    type ReceiveError: Error;

    /// Sends a notification received by the local listener to the
    /// opposing side.
    fn send(&mut self, id: EventId) -> Result<(), Self::SendError>;

    /// Receives one notification from the opposing side for the local
    /// notifier, or `None` if nothing is pending.
    fn receive(&mut self) -> Result<Option<EventId>, Self::ReceiveError>;
}
