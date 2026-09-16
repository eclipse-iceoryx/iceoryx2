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

mod cli;
mod tunnel;
mod wake;

use clap::Parser;
use iceoryx2::prelude::*;
use iceoryx2_cli::install_panic_handlers;
use iceoryx2_log::{LogLevel, fail, info, set_log_level_from_env_or};

use cli::Cli;
use tunnel::{ZenohTunnel, create_tunnel, spin};
use wake::WakeSources;

const ORIGIN: &str = "iox2-link-tunnel-zenoh";

fn main() -> anyhow::Result<()> {
    install_panic_handlers!();
    set_log_level_from_env_or(LogLevel::Info);
    let cli = Cli::parse();

    info!(from ORIGIN, "Starting iox2-link-tunnel-zenoh v{}", env!("CARGO_PKG_VERSION"));

    let iceoryx_config = iceoryx2::config::Config::default();
    let mut tunnel = fail!(
        from ORIGIN,
        when create_tunnel(&cli, &iceoryx_config),
        "Failed to create the tunnel"
    );
    let wake_sources = fail!(
        from ORIGIN,
        when WakeSources::new(&cli, &mut tunnel),
        "Failed to set up the wake sources"
    );

    info!(from ORIGIN, "Tunnel started, Ctrl-C to stop");
    fail!(
        from ORIGIN,
        when run(&mut tunnel, &wake_sources),
        "The tunnel stopped on an error"
    );
    info!(from ORIGIN, "Tunnel stopped");
    Ok(())
}

fn run(tunnel: &mut ZenohTunnel, wake_sources: &WakeSources) -> anyhow::Result<()> {
    let waitset = fail!(
        from ORIGIN,
        when WaitSetBuilder::new().create::<ipc::Service>(),
        "Failed to create the wait set"
    );
    let _attached = fail!(
        from ORIGIN,
        when wake_sources.attach(&waitset),
        "Failed to attach the wake sources"
    );
    fail!(
        from ORIGIN,
        when waitset.wait_and_process(|_| {
            wake_sources.drain();
            spin(tunnel);
            CallbackProgression::Continue
        }),
        "The waitset failed"
    );
    Ok(())
}
