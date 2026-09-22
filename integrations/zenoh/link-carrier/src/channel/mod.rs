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
use iceoryx2_log::{fail, origin};
use zenoh::Wait;
use zenoh::bytes::ZBytes;
use zenoh::key_expr::OwnedKeyExpr;
use zenoh::pubsub::{Publisher, Subscriber};
use zenoh::qos::Reliability;
use zenoh::sample::{Locality, Sample};

use crate::inbox::Inbox;

mod event;
mod sample;

pub use event::ZenohEventChannel;
pub use sample::ZenohSampleChannel;

/// Samples a channel holds pending.
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

/// The zenoh publisher and subscriber one service's bytes cross on.
pub(crate) struct ZenohChannel {
    publisher: Publisher<'static>,
    subscriber: Subscriber<Inbox<Sample>>,
}

impl ZenohChannel {
    pub(crate) fn open(
        session: &zenoh::Session,
        key: OwnedKeyExpr,
        wake: Arc<OnceLock<WakeHandle>>,
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
        })
    }

    /// Puts the bytes as one zenoh payload.
    fn put(&self, bytes: &[&[u8]]) -> Result<(), Error> {
        let origin = origin!("ZenohChannel::put");

        let mut concatenated = ZBytes::writer();
        for slice in bytes {
            concatenated.append(ZBytes::from(*slice));
        }
        fail!(
            from origin,
            when self.publisher.put(concatenated.finish()).wait(),
            with Error::Put,
            "Failed to put the bytes"
        );

        Ok(())
    }

    /// The next pending bytes, if any.
    fn pop(&self) -> Option<Vec<u8>> {
        self.subscriber
            .handler()
            .pop()
            .map(|sample| sample.payload().to_bytes().into_owned())
    }
}
