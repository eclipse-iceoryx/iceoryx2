//  Copyright (c) 2024 Contributors to the Eclipse Foundation
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
use enum_iterator::all;
use iceoryx2::config::Config;
use iceoryx2_bb_posix::system_configuration::*;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Prints the whole system configuration with all limits, features and details to the console.
pub fn print_system_configuration() {
    const HEADER_COLOR: &str = "\x1b[4;92m";
    const VALUE_COLOR: &str = "\x1b[0;94m";
    const DISABLED_VALUE_COLOR: &str = "\x1b[0;90m";
    const ENTRY_COLOR: &str = "\x1b[0;37m";
    const DISABLED_ENTRY_COLOR: &str = "\x1b[0;90m";
    const COLOR_RESET: &str = "\x1b[0m";

    println!("{}posix system configuration{}", HEADER_COLOR, COLOR_RESET);
    println!();
    println!(" {}system info{}", HEADER_COLOR, COLOR_RESET);
    for i in all::<SystemInfo>().collect::<Vec<_>>() {
        println!(
            "  {ENTRY_COLOR}{:<50}{COLOR_RESET} {VALUE_COLOR}{}{COLOR_RESET}",
            format!("{:?}", i),
            i.value(),
        );
    }

    println!();
    println!(" {}limits{}", HEADER_COLOR, COLOR_RESET);
    for i in all::<Limit>().collect::<Vec<_>>() {
        let limit = i.value();
        let limit = if limit == 0 {
            "[ unlimited ]".to_string()
        } else {
            limit.to_string()
        };
        println!(
            "  {ENTRY_COLOR}{:<50}{COLOR_RESET} {VALUE_COLOR}{}{COLOR_RESET}",
            format!("{:?}", i),
            limit,
        );
    }

    println!();
    println!(" {}options{}", HEADER_COLOR, COLOR_RESET);
    for i in all::<SysOption>().collect::<Vec<_>>() {
        if i.is_available() {
            println!(
                "  {ENTRY_COLOR}{:<50}{COLOR_RESET} {VALUE_COLOR}{}{COLOR_RESET}",
                format!("{:?}", i),
                i.is_available(),
            );
        } else {
            println!(
                "  {DISABLED_ENTRY_COLOR}{:<50}{COLOR_RESET} {DISABLED_VALUE_COLOR}{}{COLOR_RESET}",
                format!("{:?}", i),
                i.is_available(),
            );
        }
    }

    println!();
    println!(" {}features{}", HEADER_COLOR, COLOR_RESET);
    for i in all::<Feature>().collect::<Vec<_>>() {
        if i.is_available() {
            println!(
                "  {ENTRY_COLOR}{:<50}{COLOR_RESET} {VALUE_COLOR}{}{COLOR_RESET}",
                format!("{:?}", i),
                i.is_available(),
            );
        } else {
            println!(
                "  {DISABLED_ENTRY_COLOR}{:<50}{COLOR_RESET} {DISABLED_VALUE_COLOR}{}{COLOR_RESET}",
                format!("{:?}", i),
                i.is_available(),
            );
        }
    }

    println!();
    println!(" {}process resource limits{}", HEADER_COLOR, COLOR_RESET);
    for i in all::<ProcessResourceLimit>().collect::<Vec<_>>() {
        println!(
            "  {ENTRY_COLOR}{:<43}{COLOR_RESET} soft:  {VALUE_COLOR}{:<24}{COLOR_RESET} hard:  {VALUE_COLOR}{}{COLOR_RESET}",
            format!("{:?}", i),
            i.soft_limit(),
            i.hard_limit()
        );
    }
}

pub fn show() -> Result<()> {
    print_system_configuration();

    Ok(())
}

pub fn generate() -> Result<()> {
    let default_file_path = Path::new("config/iceoryx2.toml");

    let default_config = Config::default();

    let toml_string = toml::to_string_pretty(&default_config)?;

    let mut file = File::create(&default_file_path)?;
    file.write_all(toml_string.as_bytes())?;

    println!(
        "Default configuration is generated at {}",
        default_file_path.display()
    );

    Ok(())
}
