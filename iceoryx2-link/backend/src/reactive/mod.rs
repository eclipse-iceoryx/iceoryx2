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

mod wake;

pub use wake::{WakeHandle, WakeService};

/// Signals a wake whenever what it lists or carries may have changed.
/// What cannot tell does not implement it.
pub trait Reactive {
    /// Attaches the wake to signal from now on.
    ///
    /// Called at most once. A wake attached is signalled whenever the
    /// listing or a channel may have changed, and may be signalled when
    /// nothing did.
    fn attach(&mut self, wake: WakeHandle);
}
