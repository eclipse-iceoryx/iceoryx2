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

use anyhow::Result;
use colored::*;

use iceoryx2_cli::command::ExternalCommandFinder;
use iceoryx2_cli::command::HostEnvironment;

use super::COMMAND_PREFIX;

pub(crate) fn list() -> Result<()> {
    let commands = ExternalCommandFinder::<HostEnvironment>::commands_with_prefix(COMMAND_PREFIX)?;

    if commands.is_empty() {
        println!("{}", "No tunnel backends found.".yellow().bold());
        println!();
        println!("Install a backend to get started, e.g.:");
        println!("  cargo install iceoryx2-integrations-zenoh-tunnel-cli");
        return Ok(());
    }

    println!("{}", "Discovered Commands:".bright_green().bold());
    for command in commands {
        println!("  {}", command.name.bold());
    }

    Ok(())
}
