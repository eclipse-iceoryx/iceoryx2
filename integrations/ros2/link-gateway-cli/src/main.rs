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
mod gateway;
mod wake;

use clap::Parser;
use iceoryx2::prelude::*;
use iceoryx2_cli::install_panic_handlers;
use iceoryx2_log::{LogLevel, fail, info, set_log_level_from_env_or};

use cli::Cli;
use gateway::{GatewayInstance, create_gateway};
use wake::WakeSources;

const ORIGIN: &str = "iox2-link-gateway-ros2";

fn main() -> anyhow::Result<()> {
    install_panic_handlers!();
    set_log_level_from_env_or(LogLevel::Info);
    let cli = Cli::parse();
    info!(from ORIGIN, "Starting iox2-link-gateway-ros2 v{}", env!("CARGO_PKG_VERSION"));

    fail!(
        from ORIGIN,
        when check_ros_environment(),
        "No usable ROS 2 environment"
    );

    let iceoryx_config = iceoryx2::config::Config::global_config();
    let mut gateway = fail!(
        from ORIGIN,
        when create_gateway(&cli, iceoryx_config),
        "Failed to create the gateway"
    );
    let wake_sources = fail!(
        from ORIGIN,
        when WakeSources::new(&cli, &mut *gateway),
        "Failed to set up the wake sources"
    );

    info!(from ORIGIN, "Gateway started, Ctrl-C to stop");
    fail!(
        from ORIGIN,
        when run(&mut *gateway, &wake_sources),
        "The gateway stopped on an error"
    );
    info!(from ORIGIN, "Gateway stopped");
    Ok(())
}

/// Spins `gateway` on every wake until stopped.
fn run(gateway: &mut dyn GatewayInstance, wake_sources: &WakeSources) -> anyhow::Result<()> {
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
            gateway.spin();
            CallbackProgression::Continue
        }),
        "The waitset failed"
    );
    Ok(())
}

/// Fails fast when no sourced ROS 2 environment is detected.
fn check_ros_environment() -> anyhow::Result<()> {
    if std::env::var_os("AMENT_PREFIX_PATH").is_none() {
        fail!(
            from ORIGIN,
            with anyhow::anyhow!("no sourced ROS 2 environment"),
            "No sourced ROS 2 environment detected, AMENT_PREFIX_PATH is unset"
        );
    }
    Ok(())
}
