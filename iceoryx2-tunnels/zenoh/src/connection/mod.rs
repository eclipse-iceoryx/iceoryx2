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

mod event;
mod publish_subscribe;

pub use event::*;
pub use publish_subscribe::*;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PropagationError {
    Error,
}

impl core::fmt::Display for PropagationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        core::write!(f, "PropagationError::{self:?}")
    }
}

impl core::error::Error for PropagationError {}

pub trait Connection {
    fn propagate(&self) -> Result<(), PropagationError>;
}
