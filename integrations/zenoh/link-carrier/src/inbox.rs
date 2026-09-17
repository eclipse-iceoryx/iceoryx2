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

/// A queue from zenoh's callback threads to the thread driving the
/// carrier.
pub(crate) struct Inbox<T> {
    sender: flume::Sender<T>,
    receiver: flume::Receiver<T>,
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
            wake,
        }
    }

    /// A handle to push with.
    pub(crate) fn sender(&self) -> Sender<T> {
        Sender {
            sender: self.sender.clone(),
            wake: self.wake.clone(),
        }
    }

    /// The oldest item not popped yet, if any.
    pub(crate) fn pop(&self) -> Option<T> {
        self.receiver.try_recv().ok()
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

impl<T: Send + 'static> IntoHandler<T> for Inbox<T> {
    type Handler = Self;

    fn into_handler(self) -> (Callback<T>, Self::Handler) {
        let sender = self.sender();
        (Callback::from(move |item: T| sender.push(item)), self)
    }
}
