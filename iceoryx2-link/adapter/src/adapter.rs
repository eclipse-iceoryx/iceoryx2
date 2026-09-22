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

use core::error::Error;

use iceoryx2_bb_elementary::generation::Generation;
use iceoryx2_link_backend::service_description::Identified;

use crate::{EndpointDescription, EventEndpoints, PublishSubscribeEndpoints};

/// The abstraction over the middleware a gateway connects `iceoryx2` to.
pub trait Adapter {
    type ListError: Error;
    type OpenError: Error;
    type EndpointSettings: Clone + PartialEq + Identified;
    type EndpointTypes: Clone + PartialEq;
    type PublishSubscribeEndpoints: PublishSubscribeEndpoints;
    type EventEndpoints: EventEndpoints;

    /// The generation of the middleware's endpoints. A tracked one moves
    /// whenever they may have changed since the last call and holds still
    /// while they have not. Untracked, every listing is taken as changed.
    fn generation(&self) -> Generation {
        Generation::Untracked
    }

    /// Calls `callback` once for each description remote endpoints are
    /// listed under, before returning. The gateway's own endpoints alone
    /// should not be listed.
    #[allow(clippy::type_complexity)] // the description is spelled out so an implementer sees both of its halves
    fn endpoints(
        &self,
        callback: &mut dyn FnMut(&EndpointDescription<Self::EndpointSettings, Self::EndpointTypes>),
    ) -> Result<(), Self::ListError>;

    /// The gateway's publish-subscribe endpoints matching `description`.
    fn publish_subscribe(
        &mut self,
        description: &EndpointDescription<Self::EndpointSettings, Self::EndpointTypes>,
    ) -> Result<Self::PublishSubscribeEndpoints, Self::OpenError>;

    /// The gateway's event endpoints matching `description`.
    fn event(
        &mut self,
        description: &EndpointDescription<Self::EndpointSettings, Self::EndpointTypes>,
    ) -> Result<Self::EventEndpoints, Self::OpenError>;
}
