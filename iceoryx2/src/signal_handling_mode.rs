// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

/// Defines how signals are handled by constructs that might register a custom
/// [`SignalHandler`](iceoryx2_bb_posix::signal::SignalHandler)
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalHandlingMode {
    /// The signals [`Signal::Interrupt`](iceoryx2_bb_posix::signal::Signal::Interrupt) and
    /// [`Signal::Terminate`](iceoryx2_bb_posix::signal::Signal::Terminate) are registered and
    /// handled. If such a [`Signal`](iceoryx2_bb_posix::signal::Signal) is received the user will
    /// be notified.
    #[default]
    HandleTerminationRequests,
    /// No signal handler will be registered.
    Disabled,
}
