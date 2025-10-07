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

pub mod event;
mod factory;
pub mod publish_subscribe;

pub use factory::*;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2_bb_log::{error, fail};
use zenoh::Wait;
use zenoh::{sample::Locality, Session};

use crate::keys;

pub fn announce_service(
    session: &Session,
    static_config: &StaticConfig,
) -> Result<(), zenoh::Error> {
    let key = keys::service_details(static_config.service_id());
    let service_config_serialized = fail!(
        from "announce_service()",
        when serde_json::to_string(&static_config),
        "failed to serialize service config"
    );

    // Notify all current hosts.
    fail!(
        from "announce_service()",
        when session
            .put(key.clone(), service_config_serialized.clone())
            .allowed_destination(Locality::Remote)
            .wait(),
        "failed to share service details with remote hosts"
    );

    // Set up a queryable to respond to future hosts.
    fail!(
        from "announce_service()",
        when session
            .declare_queryable(key.clone())
            .callback(move |query| {
                let _ = query
                    .reply(key.clone(), service_config_serialized.clone())
                    .wait()
                    .inspect_err(|e| {
                        error!("Failed to announce service {}: {}", key, e);
                    });
            })
            .allowed_origin(Locality::Remote)
            .background()
            .wait(),
        "failed to set up queryable to share service details with remote hosts"
    );

    Ok(())
}
