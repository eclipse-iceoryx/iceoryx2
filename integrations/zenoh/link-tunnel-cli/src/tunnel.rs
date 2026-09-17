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

use iceoryx2::prelude::*;
use iceoryx2_integrations_zenoh_link_carrier::ZenohCarrier;
use iceoryx2_link::Link;
use iceoryx2_link_backend::AllowList;
use iceoryx2_link_tunnel::Tunnel;
use iceoryx2_log::{fail, info, warn};

use crate::ORIGIN;
use crate::cli::Cli;

pub type ZenohTunnel = Link<ipc::Service, Tunnel<ZenohCarrier>>;

/// The tunnel the command line configures.
pub fn create_tunnel(
    cli: &Cli,
    iceoryx_config: &iceoryx2::config::Config,
) -> anyhow::Result<ZenohTunnel> {
    let zenoh_config = fail!(
        from ORIGIN,
        when zenoh_config(cli.zenoh_config.as_deref()),
        "Failed to load the zenoh config"
    );
    let node = fail!(
        from ORIGIN,
        when NodeBuilder::new().config(iceoryx_config).create::<ipc::Service>(),
        "Failed to create the tunnel's node"
    );
    let carrier = fail!(
        from ORIGIN,
        when ZenohCarrier::create(zenoh_config),
        "Failed to create the zenoh carrier"
    );
    let tunnel = Link::new(node, Tunnel::new(carrier, iceoryx_config));
    let tunnel = if cli.monitor {
        info!(from ORIGIN, "Monitoring what the bridges move");
        tunnel.with_monitoring()
    } else {
        tunnel
    };

    // Bridge only the services the allow list admits, every one when none is given.
    if cli.allow.is_empty() {
        return Ok(tunnel);
    }
    info!(from ORIGIN, "Allowing {:?}", cli.allow);
    let allow_list = AllowList::new(&cli.allow);
    Ok(tunnel.with_filter(move |name: &ServiceName| allow_list.admits(name.as_str())))
}

/// One round of discovery and propagation.
pub fn spin(tunnel: &mut ZenohTunnel) {
    if let Err(error) = tunnel.discover() {
        warn!(from ORIGIN, "Discovery failed: {}", error);
    }
    if let Err(error) = tunnel.propagate() {
        warn!(from ORIGIN, "Propagation failed: {}", error);
    }
}

fn zenoh_config(path: Option<&str>) -> anyhow::Result<zenoh::Config> {
    match path {
        Some(path) => {
            info!(from ORIGIN, "Loading the zenoh config from {:?}", path);
            let config = fail!(
                from ORIGIN,
                when zenoh::Config::from_file(path).map_err(anyhow::Error::from_boxed),
                "Failed to read the zenoh config file {:?}", path
            );
            Ok(config)
        }
        None => {
            info!(from ORIGIN, "Using the default zenoh config");
            Ok(zenoh::Config::default())
        }
    }
}
