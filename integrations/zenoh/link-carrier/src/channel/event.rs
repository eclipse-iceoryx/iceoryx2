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

use iceoryx2::port::event_id::EventId;
use iceoryx2_link_backend::wire::event::{decode, encode};
use iceoryx2_link_carrier::{EventChannel, EventReceiveError};
use iceoryx2_log::{fail, origin};

use super::{Error, ZenohChannel};

/// The ids of one event service over zenoh.
pub struct ZenohEventChannel(pub(crate) ZenohChannel);

impl EventChannel for ZenohEventChannel {
    type Error = Error;

    fn send(&mut self, id: EventId) -> Result<(), Self::Error> {
        self.0.send(&[&encode(id)])
    }

    fn receive(&mut self) -> Result<Option<EventId>, EventReceiveError<Self::Error>> {
        let origin = origin!("ZenohEventChannel::receive");

        let Some(bytes) = self.0.receive() else {
            return Ok(None);
        };

        let id = fail!(
            from origin,
            when decode(bytes),
            to EventReceiveError<Error>,
            "Dropped {} bytes that are not an event id", bytes.len()
        );

        Ok(Some(id))
    }
}
