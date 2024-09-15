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

use clap::Parser;
use clap::Subcommand;

use iceoryx2_cli_utils::help_template;
use iceoryx2_cli_utils::Format;

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

    #[clap(long, short = 'f', value_enum, global = true)]
    pub format: Option<Format>,
}

#[derive(Parser)]
pub struct DetailsOptions {
    #[clap(help = "")]
    pub node: String,
}

#[derive(Subcommand)]
pub enum Action {
    #[clap(about = "")]
    List,
    #[clap(about = "")]
    Details(DetailsOptions),
}
