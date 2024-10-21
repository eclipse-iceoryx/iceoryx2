// Copyright (c) 2024 Contributors to the Eclipse Foundation
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
use colored::Colorize;
use dialoguer::Confirm;
use dirs::config_dir;
use enum_iterator::all;
use iceoryx2::config::Config;
use iceoryx2_bb_posix::system_configuration::*;
use std::fs::{self, File};
use std::io::Write;
use std::panic::catch_unwind;

/// Prints the whole system configuration with all limits, features and details to the console.
pub fn print_system_configuration() {
    println!(
        "{}",
        "posix system configuration".underline().bright_green()
    );
    println!();
    println!(" {}", "system info".underline().bright_green());
    for i in all::<SystemInfo>().collect::<Vec<_>>() {
        println!(
            "  {:<50} {}",
            format!("{:?}", i).white(),
            format!("{}", i.value()).bright_blue(),
        );
    }

    println!();
    println!(" {}", "limit".underline().bright_green());
    for i in all::<Limit>().collect::<Vec<_>>() {
        let limit = i.value();
        let limit = if limit == 0 {
            "[ unlimited ]".to_string()
        } else {
            limit.to_string()
        };
        println!(
            "  {:<50} {}",
            format!("{:?}", i).white(),
            limit.bright_blue(),
        );
    }

    println!();
    println!(" {}", "options".underline().bright_green());
    for i in all::<SysOption>().collect::<Vec<_>>() {
        if i.is_available() {
            println!(
                "  {:<50} {}",
                format!("{:?}", i).white(),
                format!("{}", i.is_available()).bright_blue()
            );
        } else {
            println!("  {:<50} {}", format!("{:?}", i), i.is_available(),);
        }
    }

    println!();
    println!(" {}", "features".underline().bright_green());
    for i in all::<Feature>().collect::<Vec<_>>() {
        if i.is_available() {
            println!(
                "  {:<50} {}",
                format!("{:?}", i).white(),
                format!("{}", i.is_available()).bright_blue(),
            );
        } else {
            println!("  {:<50} {}", format!("{:?}", i), i.is_available(),);
        }
    }

    println!();
    println!(" {}", "process resource limits".underline().bright_green());
    for i in all::<ProcessResourceLimit>().collect::<Vec<_>>() {
        let soft_limit_result = catch_unwind(|| i.soft_limit());
        let hard_limit_result = catch_unwind(|| i.hard_limit());

        match (soft_limit_result, hard_limit_result) {
            (Ok(soft), Ok(hard)) => {
                println!(
                    "  {:<43} soft:  {:<24} hard:  {}",
                    format!("{:?}", i).white(),
                    format!("{}", soft).bright_blue(),
                    format!("{}", hard).bright_blue()
                );
            }
            (Err(e), _) | (_, Err(e)) => {
                println!(
                    "  {:<43} Error: {}",
                    format!("{:?}", i).white(),
                    format!("Unable to acquire limit due to: {:?}", e).red()
                );
            }
        }
    }
}

pub fn show_system_config() -> Result<()> {
    print_system_configuration();

    Ok(())
}

pub fn show_current_config() -> Result<()> {
    let config = Config::global_config();
    let toml_config = toml::to_string_pretty(&config)?;
    println!("{}", toml_config);

    Ok(())
}

pub fn generate() -> Result<()> {
    let config_dir = config_dir().unwrap().join("iceoryx2");
    fs::create_dir_all(&config_dir)?;

    let default_file_path = config_dir.join("config.toml");

    if default_file_path.exists() {
        let proceed = Confirm::new()
            .with_prompt("Configuration file already exists. Do you want to overwrite it?")
            .default(false)
            .interact()?;

        if !proceed {
            println!("Operation cancelled. Configuration file was not overwritten.");
            return Ok(());
        }
    }

    let toml_string = toml::to_string_pretty(&Config::default())?;

    let mut file = File::create(&default_file_path)?;
    file.write_all(toml_string.as_bytes())?;

    println!(
        "Default configuration is generated at {}",
        default_file_path.display()
    );

    Ok(())
}
