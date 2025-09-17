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

mod explain;
mod generate;
mod show;

pub(crate) use explain::*;
pub(crate) use generate::*;
pub(crate) use show::*;

use colored::Colorize;
use enum_iterator::all;
use iceoryx2_bb_posix::system_configuration::*;
use std::panic::catch_unwind;

/// Prints the whole system configuration with all limits, features and
/// details to the console.
pub(crate) fn print_system_configuration() {
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
