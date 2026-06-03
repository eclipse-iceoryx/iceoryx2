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
use iceoryx2::service::Service;

/// Process-local wake source signaled by reactive backends when new data is
/// ready to be processed, waking any [`WaitSet`](iceoryx2::waitset::WaitSet)
/// with the corresponding listener attached.
///
/// The wake service type `W` is chosen by the backend. A thread-safe variant
/// is required only when the backend signals from a thread other than the one
/// reading the listener.
#[derive(Debug)]
pub struct WakeHandle<W: Service> {
    notifier: Notifier<W>,
}

impl<W: Service> WakeHandle<W> {
    pub fn new(notifier: Notifier<W>) -> Self {
        Self { notifier }
    }

    /// Wakes any listener attached to the backing event service.
    pub fn signal(&self) {
        let _ = self.notifier.notify();
    }
}
