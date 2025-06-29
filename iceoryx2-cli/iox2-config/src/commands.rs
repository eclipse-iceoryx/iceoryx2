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
use enum_iterator::all;
use iceoryx2::config::Config;
use iceoryx2_bb_posix::directory::Directory;
use iceoryx2_bb_posix::file::Permission;
use iceoryx2_bb_posix::system_configuration::*;
use iceoryx2_bb_posix::*;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;
use std::panic::catch_unwind;

/// Prints the whole system configuration with all limits, features and details to the console.
pub fn print_system_configuration() {
    println!(
        "{}",
        "posix system configuration".underline().bright_green()
    );
    println!();
    println!(" {}", "system info".underline().bright_green());
    all::<SystemInfo>().for_each(|i| {
        println!(
            "  {:<50} {}",
            format!("{i:?}").white(),
            format!("{}", i.value()).bright_blue(),
        );
    });

    println!();
    println!(" {}", "limit".underline().bright_green());
    for i in all::<Limit>().collect::<Vec<_>>() {
        let limit = i.value();
        let limit = if limit == 0 {
            "[ unlimited ]".to_string()
        } else {
            limit.to_string()
        };
        println!("  {:<50} {}", format!("{i:?}").white(), limit.bright_blue(),);
    }

    println!();
    println!(" {}", "options".underline().bright_green());
    for i in all::<SysOption>().collect::<Vec<_>>() {
        if i.is_available() {
            println!(
                "  {:<50} {}",
                format!("{i:?}").white(),
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
                format!("{i:?}").white(),
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
                    format!("{i:?}").white(),
                    format!("{soft}").bright_blue(),
                    format!("{hard}").bright_blue()
                );
            }
            (Err(e), _) | (_, Err(e)) => {
                println!(
                    "  {:<43} Error: {}",
                    format!("{i:?}").white(),
                    format!("Unable to acquire limit due to: {e:?}").red()
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
    println!("{toml_config}");

    Ok(())
}

pub fn generate_global() -> Result<()> {
    let mut global_config_path = get_global_config_path();
    global_config_path.add_path_entry(&iceoryx2::config::Config::relative_config_path())?;
    let filepath = FilePath::from_path_and_file(
        &global_config_path,
        &iceoryx2::config::Config::default_config_file_name(),
    )
    .unwrap();

    generate(global_config_path, filepath)
}

pub fn generate_local() -> Result<()> {
    let user = iceoryx2_bb_posix::user::User::from_self().unwrap();
    let mut user_config_path = match user.details() {
        Some(details) => details.config_dir().clone(),
        None => {
            return Err(anyhow::anyhow!(
                "User config directory not available on this platform!"
            ))
        }
    };
    user_config_path.add_path_entry(&iceoryx2::config::Config::relative_config_path())?;
    let filepath = FilePath::from_path_and_file(
        &user_config_path,
        &iceoryx2::config::Config::default_config_file_name(),
    )
    .unwrap();

    generate(user_config_path, filepath)
}

fn generate(config_dir: Path, filepath: FilePath) -> Result<()> {
    if let Ok(exists) = file::File::does_exist(&filepath) {
        if exists {
            let proceed = Confirm::new()
                .with_prompt("Configuration file already exists. Do you want to overwrite it?")
                .default(false)
                .interact()?;

            if !proceed {
                println!("Operation cancelled. Configuration file was not overwritten.");
                return Ok(());
            }
        } else if let Ok(false) = Directory::does_exist(&config_dir) {
            Directory::create(
                &config_dir,
                file::Permission::OWNER_ALL
                    | Permission::GROUP_READ
                    | Permission::GROUP_EXEC
                    | Permission::OTHERS_READ
                    | Permission::OTHERS_EXEC,
            )
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        }
    }

    let toml_string = toml::to_string_pretty(&Config::default())?;

    let mut file = file::FileBuilder::new(&filepath)
        .creation_mode(file::CreationMode::PurgeAndCreate)
        .permission(
            file::Permission::OWNER_WRITE
                | file::Permission::OWNER_READ
                | file::Permission::GROUP_READ
                | file::Permission::OTHERS_READ,
        )
        .create()
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    file.write(toml_string.as_bytes())
        .expect("Failed to write to file");

    println!("Default configuration is generated at {filepath}");

    Ok(())
}
