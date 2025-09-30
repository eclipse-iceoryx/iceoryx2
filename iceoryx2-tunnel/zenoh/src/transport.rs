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

use iceoryx2_bb_log::fail;
use zenoh::{Config, Session, Wait};

use crate::RelayFactory;

pub enum CreationError {
    FailedToCreateSession,
}

pub struct Transport {
    session: Session,
}

impl iceoryx2_tunnel_traits::Transport for Transport {
    type Config = Config;
    type CreationError = CreationError;
    type RelayFactory = RelayFactory;

    fn create(config: &Self::Config) -> Result<Self, Self::CreationError> {
        let session = zenoh::open(config.clone()).wait();
        let session = fail!(
            from "ZenohTransport::create()",
            when session,
            with Self::CreationError::FailedToCreateSession,
            "failed to create zenoh session"
        );

        Ok(Self { session })
    }

    fn relay_builder(&self) -> Self::RelayFactory {
        Self::RelayFactory {}
    }
}
