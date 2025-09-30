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

mod publisher;
pub use publisher::*;

mod subscriber;
pub use subscriber::*;

mod listener;
pub use listener::*;

mod notifier;
pub use notifier::*;

/// Represents errors that can occur during the propagation process in a channel.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PropagationError {
    /// Indicates a failure occurred in the iceoryx port during propagation.
    IceoryxPort,
    /// Indicates a failure occurred in a port other than the iceoryx port during propagation.
    OtherPort,
    /// Indicates that propagation was only partially successful, with at least one channel failing.
    Incomplete,
}

impl core::fmt::Display for PropagationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        core::write!(f, "PropagationError::{self:?}")
    }
}

impl core::error::Error for PropagationError {}

pub trait Channel {
    fn propagate(&self) -> Result<(), PropagationError>;
}
