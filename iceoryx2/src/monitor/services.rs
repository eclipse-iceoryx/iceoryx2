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

use crate::config::Config;
use crate::service::ipc;
use crate::service::Service;
use iceoryx2_bb_elementary::CallbackProgression;

struct Registry {}

// - Registry of all services in the system
// - Callback to update registry
// - Changes to registry published on internal topic
struct Monitor {
    registry: Registry,
}

impl Monitor {
    pub fn update() {
        if let Err(_) = ipc::Service::list(Config::global_config(), |service| {
            println!("\n{:#?}", &service);
            CallbackProgression::Continue
        }) {
            // On error, function does nothing
        }
    }
    pub fn publish() {}
}
