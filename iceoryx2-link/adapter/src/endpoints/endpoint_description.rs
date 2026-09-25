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

/// The types used by the endpoint by messaging pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndpointTypes<SampleTypes> {
    /// The sample type used by an endpoint for the publish-subscribe messaging
    /// pattern.
    PublishSubscribe(SampleTypes),
    /// Events carry no types.
    Event,
}

/// The middleware's description of an endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointDescription<EndpointSettings, EndpointTypes> {
    pub settings: EndpointSettings,
    pub types: EndpointTypes,
}
