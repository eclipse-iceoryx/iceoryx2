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

use crate::relays::{event, publish_subscribe};

pub struct RelayFactory {}

impl iceoryx2_tunnel_traits::RelayFactory for RelayFactory {
    type PublishSubscribeBuilder = publish_subscribe::Builder;
    type EventBuilder = event::Builder;

    fn publish_subscribe(&self, service: &str) -> Self::PublishSubscribeBuilder {
        Self::PublishSubscribeBuilder {}
    }

    fn event(&self, service: &str) -> Self::EventBuilder {
        Self::EventBuilder {}
    }
}
