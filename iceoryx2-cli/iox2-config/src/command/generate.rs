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
use dialoguer::Confirm;
use iceoryx2::config::Config;
use iceoryx2_bb_posix::directory::Directory;
use iceoryx2_bb_posix::file::Permission;
use iceoryx2_bb_posix::system_configuration::GLOBAL_CONFIG_PATH;
use iceoryx2_bb_posix::*;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;

pub(crate) fn generate_global(force: bool) -> Result<()> {
    let mut global_config_path = GLOBAL_CONFIG_PATH;
    global_config_path.add_path_entry(&iceoryx2::config::Config::relative_config_path())?;
    let filepath = FilePath::from_path_and_file(
        &global_config_path,
        &iceoryx2::config::Config::default_config_file_name(),
    )
    .unwrap();

    generate(global_config_path, filepath, force)
}

pub(crate) fn generate_local(force: bool) -> Result<()> {
    let user = iceoryx2_bb_posix::user::User::from_self().unwrap();
    let mut user_config_path = match user.details() {
        Some(details) => *details.config_dir(),
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

    generate(user_config_path, filepath, force)
}

fn generate(config_dir: Path, filepath: FilePath, force: bool) -> Result<()> {
    if let Ok(exists) = file::File::does_exist(&filepath) {
        if exists && !force {
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

    println!("Default configuration generated at {filepath}");

    Ok(())
}
