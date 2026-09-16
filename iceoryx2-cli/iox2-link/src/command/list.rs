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

use anyhow::Result;
use iceoryx2_bb_print::*;
use iceoryx2_cli::command::{ExternalCommandFinder, HostEnvironment};
use iceoryx2_log::fail;

use crate::ORIGIN;

/// Prints the installed CLIs for the specified `backend` or `when_empty` if
/// none are installed.
pub(crate) fn list(backend: &str, when_empty: &str) -> Result<()> {
    let commands = fail!(
        from ORIGIN,
        when ExternalCommandFinder::<HostEnvironment>::commands_with_prefix(backend),
        "Failed to discover the CLIs starting with {}", backend
    );

    if commands.is_empty() {
        println!("{YELLOW}{BOLD}{when_empty}{RESET}");
        return Ok(());
    }

    println!("{BRIGHT_GREEN}{BOLD}Discovered Commands:{RESET}");
    for command in commands {
        println!("  {BOLD}{}{RESET}", command.name);
    }

    Ok(())
}
