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

use iceoryx2::port::notifier::Notifier;
use iceoryx2::service::local_threadsafe;

/// Process-local wake source backed by a [`Notifier`] on a private
/// [`local_threadsafe::Service`] event service.
///
/// Reactive backends signal a [`WakeHandle`] whenever they have new data ready
/// to be processed, waking up any [`WaitSet`](iceoryx2::waitset::WaitSet) that
/// has the corresponding listener attached.
///
/// The thread-safe service variant is required because the signal may
/// fire from a backend-internal thread while the listener is read from
/// the user's main loop.
#[derive(Debug)]
pub struct WakeHandle {
    notifier: Notifier<local_threadsafe::Service>,
}

impl WakeHandle {
    /// Wraps a [`Notifier`] on a [`local_threadsafe::Service`] event service.
    pub fn new(notifier: Notifier<local_threadsafe::Service>) -> Self {
        Self { notifier }
    }

    /// Wakes any listener attached to the backing event service.
    pub fn signal(&self) {
        let _ = self.notifier.notify();
    }
}
