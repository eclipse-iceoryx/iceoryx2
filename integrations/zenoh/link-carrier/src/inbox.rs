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
use iceoryx2_log::{origin, warn};
use zenoh::handlers::{Callback, IntoHandler};
use zenoh::sample::Sample;

/// The bytes provided by Zenoh from one read.
///
/// If not contiguous, the parts are reconstructed in a heap buffer to be
/// referenced. Otherwise, the bytes are referenced directly from the buffer
/// they were written to.
pub(crate) enum ReceivedBytes {
    Contiguous(Sample),
    Joined(Vec<u8>),
}

impl ReceivedBytes {
    pub(crate) fn bytes(&self) -> &[u8] {
        match self {
            Self::Contiguous(sample) => sample.payload().slices().next().unwrap_or(&[]),
            Self::Joined(bytes) => bytes,
        }
    }
}

impl From<Sample> for ReceivedBytes {
    fn from(sample: Sample) -> Self {
        let fragmented = sample.payload().slices().nth(1).is_some();
        match fragmented {
            false => Self::Contiguous(sample),
            true => Self::Joined(sample.payload().to_bytes().into_owned()),
        }
    }
}

/// A queue from zenoh's callback threads to the thread driving the
/// carrier.
pub(crate) struct Inbox<T> {
    sender: flume::Sender<T>,
    receiver: flume::Receiver<T>,
    /// The recevied item at the head of the queue referenced by consumers on
    /// consume.
    head: Option<T>,
    /// Signalled on every push.
    wake: Arc<OnceLock<WakeHandle>>,
}

impl<T> Inbox<T> {
    /// An inbox holding up to `capacity` items.
    pub(crate) fn new(capacity: usize, wake: Arc<OnceLock<WakeHandle>>) -> Self {
        let (sender, receiver) = flume::bounded(capacity);
        Self {
            sender,
            receiver,
            head: None,
            wake,
        }
    }

    /// The handle to push bytes into the inbox.
    pub(crate) fn sender(&self) -> Sender<T> {
        Sender {
            sender: self.sender.clone(),
            wake: self.wake.clone(),
        }
    }

    /// Advanced to the next item in the inbox.
    pub(crate) fn next(&mut self) -> Option<&T> {
        self.head = self.receiver.try_recv().ok();
        self.head.as_ref()
    }
}

/// The pushing side of an [`Inbox`].
#[derive(Clone)]
pub(crate) struct Sender<T> {
    sender: flume::Sender<T>,
    wake: Arc<OnceLock<WakeHandle>>,
}

impl<T> Sender<T> {
    /// Pushes an item, dropped with a warning when the inbox is full.
    pub(crate) fn push(&self, item: T) {
        let origin = origin!("Inbox::push");

        if self.sender.try_send(item).is_err() {
            warn!(from origin, "An arrival was dropped, the inbox is full");
        }
        if let Some(wake) = self.wake.get() {
            wake.signal();
        }
    }
}

impl<T: From<Sample> + Send + 'static> IntoHandler<Sample> for Inbox<T> {
    type Handler = Self;

    fn into_handler(self) -> (Callback<Sample>, Self::Handler) {
        let sender = self.sender();
        (
            Callback::from(move |sample: Sample| sender.push(T::from(sample))),
            self,
        )
    }
}
