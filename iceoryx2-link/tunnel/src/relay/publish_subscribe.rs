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
use iceoryx2_link_backend::relay::{PublishSubscribeRelay, RelayBuilder};
use iceoryx2_link_backend::service_description::{
    PublishSubscribeDescription, SampleTypes, ServiceDescriptor,
};
use iceoryx2_link_backend::wire::publish_subscribe::Sample;
use iceoryx2_link_backend::wire::sample::{LoanableSample, payload_bytes, user_header_bytes};
use iceoryx2_link_carrier::{Carrier, SampleChannel, SampleReceiveError};
use iceoryx2_log::{fail, origin};

use crate::relay::{CreationError, ReceiveError, SendError};

/// Creates relays over a carrier.
pub struct Builder<'a, S: Service, C: Carrier> {
    carrier: &'a mut C,
    description: PublishSubscribeDescription<'a>,
    /// What the service is shared with the peers as.
    descriptor: &'a ServiceDescriptor,
    _service: PhantomData<S>,
}

impl<'a, S: Service, C: Carrier> Builder<'a, S, C> {
    pub(super) fn new(
        carrier: &'a mut C,
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
    type Relay = Relay<S, C::SampleChannel>;

    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        let origin = origin!("Builder::create");

        let channel = fail!(
            from origin,
            when self.carrier.open_sample_channel(self.descriptor),
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

/// Moves publish-subscribe samples over a carrier channel.
pub struct Relay<S: Service, C: SampleChannel> {
    channel: C,
    types: SampleTypes,
    _service: PhantomData<S>,
}

impl<S: Service, C: SampleChannel> PublishSubscribeRelay<S> for Relay<S, C> {
    type SendError = SendError<C::Error>;
    type ReceiveError = ReceiveError<C::Error>;

    fn send(&mut self, sample: &Sample<S>) -> Result<(), Self::SendError> {
        let origin = origin!("Relay::send");

        // SAFETY: the sample belongs to the service this relay was built
        // for, whose description states the user header size.
        let header =
            unsafe { user_header_bytes(sample.user_header(), self.types.user_header.size) };
        let payload = payload_bytes(sample.payload());
        fail!(
            from origin,
            when self.channel.send(&[header, payload]),
            to SendError<C::Error>,
            "Failed to send a sample"
        );

        Ok(())
    }

    fn receive<L: LoanableSample>(
        &mut self,
        loanable: L,
    ) -> Result<Option<L::Sample>, Self::ReceiveError> {
        let origin = origin!("Relay::receive");
        match self.channel.receive(loanable) {
            Ok(received) => Ok(received),
            Err(SampleReceiveError::Malformed) => {
                fail!(
                    from origin,
                    with ReceiveError::Malformed,
                    "Received bytes that do not fit the service"
                );
            }
            Err(SampleReceiveError::Exhausted) => {
                fail!(
                    from origin,
                    with ReceiveError::Loan,
                    "No sample to receive into"
                );
            }
            Err(SampleReceiveError::Channel(error)) => {
                fail!(
                    from origin,
                    with ReceiveError::Channel(error),
                    "Failed to receive a sample"
                );
            }
        }
    }
}
