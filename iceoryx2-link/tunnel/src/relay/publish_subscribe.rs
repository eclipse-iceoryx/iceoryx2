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

use iceoryx2::service::Service;
use iceoryx2_link_backend::description::{
    PublishSubscribeDescription, PublishSubscribeTypes, ServiceDescriptor,
};
use iceoryx2_link_backend::origin;
use iceoryx2_link_backend::relay::{PublishSubscribeRelay, RelayBuilder};
use iceoryx2_link_backend::wire::publish_subscribe::{
    LoanFn, Sample, SampleMut, initialize_sample, payload_bytes, user_header_bytes,
};
use iceoryx2_log::fail;

use crate::relay::{CreationError, ReceiveError, SendError};
use iceoryx2_link_carrier::Frame;
use iceoryx2_link_carrier::{Carrier, Channel};

/// Creates relays over a carrier.
pub struct Builder<'a, S: Service, C: Carrier> {
    carrier: &'a C,
    description: PublishSubscribeDescription<'a>,
    /// What the service is shared with the peers as.
    descriptor: &'a ServiceDescriptor,
    _service: PhantomData<S>,
}

impl<'a, S: Service, C: Carrier> Builder<'a, S, C> {
    pub(super) fn new(
        carrier: &'a C,
        description: PublishSubscribeDescription<'a>,
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
            types: self.description.types().clone(),
            _service: PhantomData,
        })
    }
}

/// Moves publish-subscribe samples over a carrier channel as [`Frame`]s.
pub struct Relay<S: Service, C: Channel> {
    channel: C,
    types: PublishSubscribeTypes,
    _service: PhantomData<S>,
}

impl<S: Service, C: Channel> PublishSubscribeRelay<S> for Relay<S, C> {
    type SendError = SendError<C::Error>;
    type ReceiveError = ReceiveError<C::Error>;

    fn send(&self, sample: &Sample<S>) -> Result<(), Self::SendError> {
        let origin = origin!("Relay::send");

        let frame = Frame {
            // SAFETY: the sample belongs to the service this relay was
            // built for, whose description states the user header size.
            header: unsafe { user_header_bytes(sample.user_header(), self.types.user_header.size) },
            payload: payload_bytes(sample.payload()),
        };
        fail!(
            from origin,
            when self.channel.send(frame),
            to SendError<C::Error>,
            "Failed to send a frame"
        );

        Ok(())
    }

    fn receive<LoanError>(
        &self,
        loan: &mut LoanFn<'_, S, LoanError>,
    ) -> Result<Option<SampleMut<S>>, Self::ReceiveError> {
        let origin = origin!("Relay::receive");
        let received = fail!(
            from origin,
            when self.channel.receive(|bytes| {
                let frame = fail!(
                    from origin,
                    when Frame::parse(bytes, &self.types),
                    with ReceiveError::Malformed,
                    "Received a frame that does not fit the service"
                );
                let sample = fail!(
                    from origin,
                    when loan(frame.payload.len()),
                    with ReceiveError::Loan,
                    "Failed to loan a sample for a received frame"
                );
                // SAFETY: the frame was checked against the description and
                // the sample was loaned for the payload's size.
                Ok(unsafe { initialize_sample(sample, frame.header, frame.payload) })
            }),
            to ReceiveError<C::Error>,
            "Failed to receive a frame"
        );
        received.transpose()
    }
}
