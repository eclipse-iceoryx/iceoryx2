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
use iceoryx2_link_backend::relay::{PublishSubscribeRelay, ReceiveOutcome, RelayBuilder};
use iceoryx2_link_backend::service_description::{
    PublishSubscribeDescription, SampleTypes, ServiceDescriptor,
};
use iceoryx2_link_backend::wire::publish_subscribe::Sample;
use iceoryx2_link_backend::wire::sample::{
    LoanError, LoanableSample, SampleBytesRef, WritableSample, payload_bytes, user_header_bytes,
};
use iceoryx2_link_carrier::{Carrier, SampleChannel};
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
            when self.channel.send(SampleBytesRef { header, payload }),
            to SendError<C::Error>,
            "Failed to send a sample"
        );

        Ok(())
    }

    fn receive<L: LoanableSample>(
        &mut self,
        loanable: L,
    ) -> Result<ReceiveOutcome<L::WritableSample>, Self::ReceiveError> {
        let origin = origin!("Relay::receive");

        let bytes = fail!(
            from origin,
            when self.channel.receive(),
            to ReceiveError<C::Error>,
            "Failed to receive a sample"
        );
        let Some(bytes) = bytes else {
            return Ok(ReceiveOutcome::Empty);
        };

        let header_size = self.types.user_header.size;
        if bytes.len() < header_size {
            fail!(
                from origin,
                with ReceiveError::Malformed,
                "Received fewer bytes than expected, {} bytes where the header size is {}", bytes.len(), header_size
            );
        }
        let (header, payload) = bytes.split_at(header_size);

        let mut writable = match loanable.loan(payload.len()) {
            Ok(writable) => writable,
            Err(LoanError::Exhausted) => {
                fail!(
                    from origin,
                    with ReceiveError::Loan,
                    "The publisher has no free sample for the received bytes of size {}", bytes.len()
                );
            }
            Err(LoanError::Malformed | LoanError::NotResizable) => {
                fail!(
                    from origin,
                    with ReceiveError::Malformed,
                    "Received a payload of {} bytes, which the service does not support", payload.len()
                );
            }
        };
        let regions = writable.as_mut();
        regions.header.copy_from_slice(header);
        regions.payload.copy_from_slice(payload);

        Ok(ReceiveOutcome::Sample(writable))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::collections::VecDeque;
    use alloc::vec::Vec;
    use core::convert::Infallible;

    use iceoryx2::service::local;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_link_backend::service_description::ServiceTypes;
    use iceoryx2_link_backend::wire::sample::SampleBytes;

    use crate::testing::description;

    const SERVICE: &str = "tunnel/relay/publish_subscribe";
    const PAYLOAD: &str = "u64";
    /// The size of a header and of a payload alike, as `description` states.
    const SIZE: usize = 8;
    const HEADER: [u8; SIZE] = [0x11; SIZE];
    const PAYLOAD_BYTES: [u8; SIZE] = [0x22; SIZE];

    /// A channel holding the bytes pending for the relay.
    #[derive(Default)]
    struct StubChannel {
        pending: VecDeque<Vec<u8>>,
        received: Vec<u8>,
    }

    impl SampleChannel for StubChannel {
        type Error = Infallible;

        fn send(&mut self, _: SampleBytesRef<'_>) -> Result<(), Self::Error> {
            Ok(())
        }

        fn receive(&mut self) -> Result<Option<&[u8]>, Self::Error> {
            match self.pending.pop_front() {
                Some(bytes) => {
                    self.received = bytes;
                    Ok(Some(&self.received))
                }
                None => Ok(None),
            }
        }
    }

    /// Loans heap buffers with a header of the service's size.
    struct Loanable;

    impl LoanableSample for Loanable {
        type WritableSample = SampleBytes;

        fn loan(self, payload_len: usize) -> Result<Self::WritableSample, LoanError> {
            Ok(SampleBytes {
                header: alloc::vec![0; SIZE],
                payload: alloc::vec![0; payload_len],
            })
        }
    }

    /// Refuses every loan with `refusal`.
    struct Refusing(LoanError);

    impl LoanableSample for Refusing {
        type WritableSample = SampleBytes;

        fn loan(self, _: usize) -> Result<Self::WritableSample, LoanError> {
            Err(self.0)
        }
    }

    fn relay(pending: &[&[u8]]) -> Relay<local::Service, StubChannel> {
        let ServiceTypes::PublishSubscribe(types) = description(SERVICE, PAYLOAD).types().clone()
        else {
            unreachable!("description is publish-subscribe");
        };
        Relay {
            channel: StubChannel {
                pending: pending.iter().map(|bytes| bytes.to_vec()).collect(),
                received: Vec::new(),
            },
            types,
            _service: PhantomData,
        }
    }

    #[test]
    fn header_and_payload_are_split_from_the_received_bytes() {
        let mut relay = relay(&[&[HEADER, PAYLOAD_BYTES].concat()]);

        let outcome = relay.receive(Loanable).expect("receiving succeeds");

        let ReceiveOutcome::Sample(sample) = outcome else {
            panic!("a sample is received");
        };
        assert_that!(sample.header, eq HEADER);
        assert_that!(sample.payload, eq PAYLOAD_BYTES);
    }

    #[test]
    fn nothing_pending_is_empty() {
        let mut relay = relay(&[]);

        let outcome = relay.receive(Loanable).expect("receiving succeeds");

        assert_that!(matches!(outcome, ReceiveOutcome::Empty), eq true);
    }

    #[test]
    fn bytes_shorter_than_the_header_are_malformed() {
        const SHORT: [u8; 2] = [0xAB; 2];
        let mut relay = relay(&[&SHORT, &[HEADER, PAYLOAD_BYTES].concat()]);

        let refusal = relay
            .receive(Loanable)
            .expect_err("the short bytes are refused");
        assert_that!(matches!(refusal, ReceiveError::Malformed), eq true);

        // The next sample is unaffected.
        let outcome = relay.receive(Loanable).expect("receiving succeeds");
        assert_that!(matches!(outcome, ReceiveOutcome::Sample(_)), eq true);
    }

    #[test]
    fn a_payload_the_loan_calls_malformed_is_malformed() {
        let mut relay = relay(&[&[HEADER, PAYLOAD_BYTES].concat()]);

        let refusal = relay
            .receive(Refusing(LoanError::Malformed))
            .expect_err("the sample is refused");

        assert_that!(matches!(refusal, ReceiveError::Malformed), eq true);
    }

    #[test]
    fn an_exhausted_loan_is_reported_as_such() {
        let mut relay = relay(&[&[HEADER, PAYLOAD_BYTES].concat()]);

        let refusal = relay
            .receive(Refusing(LoanError::Exhausted))
            .expect_err("the sample is refused");

        assert_that!(matches!(refusal, ReceiveError::Loan), eq true);
    }
}
