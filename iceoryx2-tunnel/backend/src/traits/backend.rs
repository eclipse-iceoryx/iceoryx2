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

use iceoryx2::service::Service;

use crate::traits::{Discovery, EventRelay, PublishSubscribeRelay, RelayFactory};

pub trait Backend<S: Service>: Sized {
    type Config: Default + Debug;
    type CreationError: Debug;
    type Discovery: Discovery;

    type PublishSubscribeRelay: PublishSubscribeRelay<S> + Debug;
    type EventRelay: EventRelay<S> + Debug;

    type RelayFactory<'a>: RelayFactory<
        S,
        PublishSubscribeRelay = Self::PublishSubscribeRelay,
        EventRelay = Self::EventRelay,
    >
    where
        Self: 'a;

    fn create(config: &Self::Config) -> Result<Self, Self::CreationError>;
    fn discovery(&self) -> &impl Discovery;
    fn relay_builder(&self) -> Self::RelayFactory<'_>;
}
