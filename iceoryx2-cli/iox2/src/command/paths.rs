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

use anyhow::{Context, Result};
use colored::*;

use crate::command::{CommandFinder, Environment, HostEnvironment, IceoryxCommandFinder};

fn paths_impl<E>() -> Result<()>
where
    E: Environment,
{
    let paths = IceoryxCommandFinder::<E>::paths().context("Failed to list search paths")?;

    if !paths.build.is_empty() {
        println!("{}", "Build Paths:".bright_green().bold());
        for dir in &paths.build {
            println!("  {}", dir.display().to_string().bold());
        }
        println!();
    }
    if !paths.install.is_empty() {
        println!("{}", "Install Paths:".bright_green().bold());
        for dir in &paths.install {
            println!("  {}", dir.display().to_string().bold());
        }
    }

    Ok(())
}

pub(crate) fn paths() -> Result<()> {
    paths_impl::<HostEnvironment>()
}
