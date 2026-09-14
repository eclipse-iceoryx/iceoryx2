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

use core::marker::PhantomData;

use iceoryx2::port::event_id::EventId;
use iceoryx2::service::Service;
use iceoryx2_link_backend::origin;
use iceoryx2_link_backend::relay::{EventRelay, RelayBuilder};
use iceoryx2_link_backend::service_description::{EventDescription, ServiceDescriptor};
use iceoryx2_link_backend::wire::event::{decode, encode};
use iceoryx2_log::fail;

use crate::relay::{CreationError, ReceiveError, SendError};
use iceoryx2_link_carrier::Frame;
use iceoryx2_link_carrier::{Carrier, Channel};

pub struct Builder<'a, S: Service, C: Carrier> {
    carrier: &'a mut C,
    description: EventDescription<'a>,
    /// What the service is shared with the peers as.
    descriptor: &'a ServiceDescriptor,
    _service: PhantomData<S>,
}

impl<'a, S: Service, C: Carrier> Builder<'a, S, C> {
    pub(super) fn new(
        carrier: &'a mut C,
        description: EventDescription<'a>,
        descriptor: &'a ServiceDescriptor,
    ) -> Self {
        Self {
            carrier,
            description,
            descriptor,
            _service: PhantomData,
        }
    }
}

impl<S: Service, C: Carrier> RelayBuilder for Builder<'_, S, C> {
    type CreationError = CreationError<C::ChannelError>;
    type Relay = Relay<S, C::Channel>;

    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        let origin = origin!("Builder::create");

        let channel = fail!(
            from origin,
            when self.carrier.open_channel(self.descriptor),
            to CreationError<C::ChannelError>,
            "Failed to open the channel of service {}", self.description.name()
        );
        Ok(Relay {
            channel,
            _service: PhantomData,
        })
    }
}

/// Moves event ids over a carrier channel as [`Frame`]s without a header.
pub struct Relay<S: Service, C: Channel> {
    channel: C,
    _service: PhantomData<S>,
}

impl<S: Service, C: Channel> EventRelay<S> for Relay<S, C> {
    type SendError = SendError<C::Error>;
    type ReceiveError = ReceiveError<C::Error>;

    fn send(&mut self, id: EventId) -> Result<(), Self::SendError> {
        let origin = origin!("Relay::send");

        let frame = Frame {
            header: &[],
            payload: &encode(id),
        };
        fail!(
            from origin,
            when self.channel.send(frame),
            to SendError<C::Error>,
            "Failed to send a frame"
        );

        Ok(())
    }

    fn receive(&mut self) -> Result<Option<EventId>, Self::ReceiveError> {
        let origin = origin!("Relay::receive");
        let received = fail!(
            from origin,
            when self.channel.receive(|bytes| {
                let id = fail!(
                    from origin,
                    when decode(bytes).ok_or(ReceiveError::Malformed),
                    "Received a frame that is not an event id"
                );
                Ok(id)
            }),
            to ReceiveError<C::Error>,
            "Failed to receive a frame"
        );
        received.transpose()
    }
}
