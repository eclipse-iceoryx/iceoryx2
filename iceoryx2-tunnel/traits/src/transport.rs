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

use core::fmt::Debug;

use crate::{Discovery, Relay, RelayFactory};

/// Abstraction of the transport over which data is propagated.
///
/// Enables implementations to define custom initialization logic.
pub trait Transport: Sized {
    type Config: Default + Debug;
    type CreationError: Debug;
    type Discovery: Discovery;
    type RelayFactory<'a>: RelayFactory
    where
        Self: 'a; // Self must live at least as long as 'a
    type PublishSubscribeRelay: Relay;
    type EventRelay: Relay;

    fn create(config: &Self::Config) -> Result<Self, Self::CreationError>;
    fn discovery(&self) -> &impl Discovery;
    fn relay_builder(&self) -> Self::RelayFactory<'_>;
}
