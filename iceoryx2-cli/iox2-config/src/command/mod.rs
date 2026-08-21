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

use enum_iterator::all;
use iceoryx2_bb_posix::system_configuration::*;
use iceoryx2_bb_print::*;
use std::panic::catch_unwind;

/// Prints the whole system configuration with all limits, features and
/// details to the console.
pub(crate) fn print_system_configuration() {
    println!("{UNDERLINE}{BRIGHT_GREEN}posix system configuration{RESET}");
    println!();
    println!(" {UNDERLINE}{BRIGHT_GREEN}system info{RESET}");
    all::<SystemInfo>().for_each(|i| {
        println!("  {WHITE}{i:?} {BRIGHT_BLUE}{}{RESET}", i.value());
    });

    println!();
    println!(" {UNDERLINE}{BRIGHT_GREEN}limit{RESET}");
    for i in all::<Limit>().collect::<Vec<_>>() {
        let limit = i.value();
        let limit = if limit == 0 {
            "[ unlimited ]".to_string()
        } else {
            limit.to_string()
        };
        println!("  {WHITE}{i:?} {BRIGHT_BLUE}{}{RESET}", limit,);
    }

    println!();
    println!(" {UNDERLINE}{BRIGHT_GREEN}options{RESET}");
    for i in all::<SysOption>().collect::<Vec<_>>() {
        if i.is_available() {
            println!("  {WHITE}{i:?} {BRIGHT_BLUE}{}{RESET}", i.is_available());
        } else {
            println!("  {WHITE}{i:?} {BRIGHT_RED}{}{RESET}", i.is_available(),);
        }
    }

    println!();
    println!(" {UNDERLINE}{BRIGHT_GREEN}features{RESET}");
    for i in all::<Feature>().collect::<Vec<_>>() {
        if i.is_available() {
            println!("  {WHITE}{i:?} {BRIGHT_BLUE}{}{RESET}", i.is_available());
        } else {
            println!("  {WHITE}{i:?} {BRIGHT_RED}{}{RESET}", i.is_available());
        }
    }

    println!();
    println!(" {UNDERLINE}{BRIGHT_GREEN}process resource limits{RESET}");
    for i in all::<ProcessResourceLimit>().collect::<Vec<_>>() {
        let soft_limit_result = catch_unwind(|| i.soft_limit());
        let hard_limit_result = catch_unwind(|| i.hard_limit());

        match (soft_limit_result, hard_limit_result) {
            (Ok(soft), Ok(hard)) => {
                println!(
                    "  {WHITE}{:<43} soft:  {BRIGHT_BLUE}{:<24} {WHITE}hard:  {BRIGHT_BLUE}{}{RESET}",
                    format!("{i:?}"),
                    soft,
                    hard,
                );
            }
            (Err(e), _) | (_, Err(e)) => {
                println!(
                    "  {WHITE}{:<43} Error: {RED}Unable to acquire limit due to: {e:?}{RESET}",
                    format!("{i:?}"),
                );
            }
        }
    }
}
