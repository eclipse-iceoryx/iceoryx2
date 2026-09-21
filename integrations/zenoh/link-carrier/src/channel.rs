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

use std::sync::{Arc, OnceLock};

use iceoryx2_link_backend::WakeHandle;
use iceoryx2_link_backend::wire::sample::{LoanableSample, WriteError};
use iceoryx2_link_carrier::{Channel, Frame, ReceiveError};
use iceoryx2_log::{fail, origin};
use zenoh::Wait;
use zenoh::key_expr::OwnedKeyExpr;
use zenoh::pubsub::{Publisher, Subscriber};
use zenoh::qos::Reliability;
use zenoh::sample::{Locality, Sample};

use crate::inbox::Inbox;

/// Frames a channel holds pending.
const CAPACITY: usize = 64;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ChannelError {
    Publisher,
    Subscriber,
}

impl core::fmt::Display for ChannelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ChannelError::{self:?}")
    }
}

impl core::error::Error for ChannelError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Error {
    Put,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Error::{self:?}")
    }
}

impl core::error::Error for Error {}

/// The frames of one service over zenoh.
pub struct ZenohChannel {
    publisher: Publisher<'static>,
    subscriber: Subscriber<Inbox<Sample>>,
    header_size: usize,
}

impl ZenohChannel {
    pub(crate) fn open(
        session: &zenoh::Session,
        key: OwnedKeyExpr,
        wake: Arc<OnceLock<WakeHandle>>,
        header_size: usize,
    ) -> Result<Self, ChannelError> {
        let origin = origin!("ZenohChannel::open");
        let publisher = fail!(
            from origin,
            when session
                .declare_publisher(key.clone())
                .allowed_destination(Locality::Remote)
                .reliability(Reliability::Reliable)
                .wait(),
            with ChannelError::Publisher,
            "Failed to declare the publisher of {}", key
        );
        let subscriber = fail!(
            from origin,
            when session
                .declare_subscriber(key.clone())
                .with(Inbox::new(CAPACITY, wake))
                .allowed_origin(Locality::Remote)
                .wait(),
            with ChannelError::Subscriber,
            "Failed to declare the subscriber of {}", key
        );
        Ok(Self {
            publisher,
            subscriber,
            header_size,
        })
    }
}

impl Channel for ZenohChannel {
    type Error = Error;

    fn send(&mut self, frame: Frame<'_>) -> Result<(), Self::Error> {
        let origin = origin!("ZenohChannel::send");
        let bytes = frame.to_bytes();
        fail!(
            from origin,
            when self.publisher.put(bytes).wait(),
            with Error::Put,
            "Failed to put a frame"
        );
        Ok(())
    }

    fn receive<L: LoanableSample>(
        &mut self,
        loanable: L,
    ) -> Result<Option<L::Sample>, ReceiveError<Self::Error>> {
        let origin = origin!("ZenohChannel::receive");

        let Some(sample) = self.subscriber.handler().pop() else {
            return Ok(None);
        };
        let bytes = sample.payload().to_bytes();
        let frame = fail!(
            from origin,
            when Frame::split(&bytes, self.header_size),
            with ReceiveError::Rejected(WriteError::Malformed),
            "Dropped a frame of {} bytes shorter than the header", bytes.len()
        );
        let sample = fail!(
            from origin,
            when frame.write_into(loanable),
            to ReceiveError<Error>,
            "Dropped a frame of {} bytes", bytes.len()
        );
        Ok(Some(sample))
    }
}
