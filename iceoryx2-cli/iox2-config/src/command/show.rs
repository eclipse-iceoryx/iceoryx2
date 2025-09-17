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

use crate::command::print_system_configuration;
use anyhow::Result;
use iceoryx2::config::Config;

pub fn show_system_config() -> Result<()> {
    print_system_configuration();

    Ok(())
}

pub fn show_current_config() -> Result<()> {
    let config = Config::global_config();
    let toml_config = toml::to_string_pretty(&config)?;
    println!("{toml_config}");

    Ok(())
}
