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

use anyhow::{anyhow, Context, Result};

use crate::command::{
    CommandExecutor, CommandFinder, Environment, HostEnvironment, IceoryxCommandExecutor,
    IceoryxCommandFinder,
};

fn execute_impl<E>(command_name: &str, args: Option<&[String]>) -> Result<()>
where
    E: Environment,
{
    let all_commands =
        IceoryxCommandFinder::<E>::commands().context("Failed to find command binaries")?;

    let command = all_commands
        .into_iter()
        .find(|command| command.name == command_name)
        .ok_or_else(|| anyhow!("Command not found: {}", command_name))?;

    IceoryxCommandExecutor::execute(&command, args)
}

pub(crate) fn execute(command_name: &str, args: Option<&[String]>) -> Result<()> {
    execute_impl::<HostEnvironment>(command_name, args)
}
