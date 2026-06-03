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

//! Zenoh subscriber handler that buffers incoming samples and signals a
//! [`WakeHandle`] on every push.

use std::sync::Arc;

use iceoryx2::service::local_threadsafe;
use iceoryx2_services_tunnel_backend::types::wake::WakeHandle;
use zenoh::handlers::{Callback, IntoHandler};

/// FIFO-buffered handler factory that signals a [`WakeHandle`] on every push.
///
/// When `wake` is `None`, behaves like a plain FIFO buffer with no wake side
/// effect (same as FifoChannel) — used by polled-mode relays.
pub struct WakeAwareChannel {
    capacity: usize,
    wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
}

impl WakeAwareChannel {
    pub fn new(capacity: usize, wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>) -> Self {
        Self { capacity, wake }
    }
}

/// Receiver side of a [`WakeAwareChannel`]. Drained lazily by the relay's
/// `receive()` path.
#[derive(Debug)]
pub struct WakeAwareReceiver<T>(flume::Receiver<T>);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ChannelDisconnected;

impl core::fmt::Display for ChannelDisconnected {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ChannelDisconnected")
    }
}

impl core::error::Error for ChannelDisconnected {}

impl<T> WakeAwareReceiver<T> {
    pub fn try_recv(&self) -> Result<Option<T>, ChannelDisconnected> {
        match self.0.try_recv() {
            Ok(value) => Ok(Some(value)),
            Err(flume::TryRecvError::Empty) => Ok(None),
            Err(flume::TryRecvError::Disconnected) => Err(ChannelDisconnected),
        }
    }
}

impl<T: Send + 'static> IntoHandler<T> for WakeAwareChannel {
    type Handler = WakeAwareReceiver<T>;

    fn into_handler(self) -> (Callback<T>, Self::Handler) {
        let (sender, receiver) = flume::bounded(self.capacity);
        let wake = self.wake;
        let callback = Callback::from(move |t: T| {
            let _ = sender.send(t);
            if let Some(wake) = &wake {
                wake.signal();
            }
        });
        (callback, WakeAwareReceiver(receiver))
    }
}
