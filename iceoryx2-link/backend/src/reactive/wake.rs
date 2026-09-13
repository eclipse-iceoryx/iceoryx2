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

use alloc::sync::Arc;

use iceoryx2::port::notifier::Notifier;
use iceoryx2::service::local_threadsafe;

/// The service a link is woken through, a `local_threadsafe` event
/// service so any thread in the process may signal it.
pub type WakeService = local_threadsafe::Service;

/// A handle a source signals when there is work to be done.
#[derive(Clone)]
pub struct WakeHandle {
    notifier: Arc<Notifier<WakeService>>,
}

impl WakeHandle {
    pub fn new(notifier: Notifier<WakeService>) -> Self {
        Self {
            notifier: Arc::new(notifier),
        }
    }

    /// Signals that there link should wake up to do work.
    pub fn signal(&self) {
        let _ = self.notifier.notify();
    }
}
