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

use std::str::FromStr;

use clap::Args;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;

use iceoryx2_cli_utils::help_template;
use iceoryx2_cli_utils::Format;
use iceoryx2_pal_posix::posix::pid_t;

#[derive(Parser)]
#[command(
    name = "iox2-nodes",
    about = "Query information about iceoryx2 nodes",
    long_about = None,
    version = env!("CARGO_PKG_VERSION"),
    disable_help_subcommand = true,
    arg_required_else_help = false,
    help_template = help_template("iox2-nodes", false),
)]
pub struct Cli {
    #[clap(subcommand)]
    pub action: Option<Action>,

    #[clap(long, short = 'f', value_enum, global = true, value_enum, default_value_t = Format::Ron)]
    pub format: Format,
}

#[derive(Clone, Debug)]
pub enum NodeIdentifier {
    Name(String),
    Id(String),
    Pid(pid_t),
}

fn is_valid_hex(s: &str) -> bool {
    s.len() == 32 && s.chars().all(|c| c.is_ascii_hexdigit())
}

impl FromStr for NodeIdentifier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(pid) = s.parse::<pid_t>() {
            Ok(NodeIdentifier::Pid(pid))
        } else if is_valid_hex(s) {
            Ok(NodeIdentifier::Id(s.to_string()))
        } else {
            Ok(NodeIdentifier::Name(s.to_string()))
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "PascalCase")]
#[derive(Default)]
pub enum StateFilter {
    Alive,
    Dead,
    Inaccessible,
    Undefined,
    #[default]
    All,
}

#[derive(Debug, Clone, Args)]
pub struct ListFilter {
    #[clap(short, long, value_enum, default_value_t = StateFilter::All)]
    pub state: StateFilter,
}

#[derive(Args)]
pub struct ListOptions {
    #[command(flatten)]
    pub filter: ListFilter,
}

#[derive(Debug, Clone, Args)]
pub struct DetailsFilter {
    #[clap(short, long, value_enum, default_value_t = StateFilter::All)]
    pub state: StateFilter,
}

#[derive(Args)]
pub struct DetailsOptions {
    #[clap(help = "Name, ID or PID")]
    pub node: NodeIdentifier,

    #[command(flatten)]
    pub filter: DetailsFilter,
}

#[derive(Subcommand)]
pub enum Action {
    #[clap(about = "List all existing nodes")]
    List(ListOptions),
    #[clap(about = "Show details of an existing node")]
    Details(DetailsOptions),
}
