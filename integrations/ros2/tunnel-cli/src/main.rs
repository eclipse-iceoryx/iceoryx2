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

mod cli;

use clap::Parser;

use cli::Cli;

use iceoryx2_cli::install_panic_handlers;
use iceoryx2_log::LogLevel;
use iceoryx2_log::info;
use iceoryx2_log::set_log_level_from_env_or;

const ORIGIN: &str = "iox2-tunnel-ros2";

fn main() -> anyhow::Result<()> {
    install_panic_handlers!();

    set_log_level_from_env_or(LogLevel::Info);

    let cli = Cli::parse();

    info!(from ORIGIN, "Starting iox2-tunnel-ros2 v{}", env!("CARGO_PKG_VERSION"));

    check_ros_environment()?;

    anyhow::bail!(
        "the tunnel run loop is not implemented yet (mapping: {:?}, translator: {:?})",
        cli.mapping(),
        cli.translator
    )
}

/// Fails fast when no sourced ROS 2 environment is detected, before any
/// rcl call can die in the dynamic loader.
fn check_ros_environment() -> anyhow::Result<()> {
    if std::env::var_os("AMENT_PREFIX_PATH").is_none() {
        anyhow::bail!(
            "no sourced ROS 2 environment detected (AMENT_PREFIX_PATH is unset); \
             source your ROS 2 setup and retry"
        );
    }
    Ok(())
}
