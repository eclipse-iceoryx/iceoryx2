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

use crate::command::{CommandFinder, Environment, HostEnvironment, IceoryxCommandFinder};

fn list_impl<E>() -> Result<()>
where
    E: Environment,
{
    let commands = IceoryxCommandFinder::<E>::commands()?;

    println!("{}", "Discovered Commands:".bright_green().bold());
    for command in commands {
        println!("  {}", command.name.bold());
    }

    Ok(())
}

pub(crate) fn list() -> Result<()> {
    list_impl::<HostEnvironment>()
}
