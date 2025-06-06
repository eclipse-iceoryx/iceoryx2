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

use clap::Args;
use clap::Parser;
use clap::Subcommand;

use iceoryx2_cli::filter::NodeIdentifier;
use iceoryx2_cli::filter::StateFilter;
use iceoryx2_cli::help_template;
use iceoryx2_cli::Format;
use iceoryx2_cli::HelpOptions;

#[derive(Parser)]
#[command(
    name = "iox2 node",
    bin_name = "iox2 node",
    about = "Query information about iceoryx2 nodes",
    long_about = None,
    version = env!("CARGO_PKG_VERSION"),
    disable_help_subcommand = true,
    arg_required_else_help = false,
    help_template = help_template(HelpOptions::PrintCommandSection),
)]
pub struct Cli {
    #[clap(subcommand)]
    pub action: Option<Action>,

    #[clap(long, short = 'f', value_enum, global = true, value_enum, default_value_t = Format::Ron)]
    pub format: Format,
}

#[derive(Debug, Clone, Args)]
pub struct OutputFilter {
    #[clap(short, long, value_enum, default_value_t = StateFilter::All)]
    pub state: StateFilter,
}

#[derive(Args)]
pub struct ListOptions {
    #[command(flatten)]
    pub filter: OutputFilter,
}

#[derive(Args)]
pub struct DetailsOptions {
    #[clap(help = "Name, ID or PID of the node")]
    pub node: NodeIdentifier,

    #[command(flatten)]
    pub filter: OutputFilter,
}

#[derive(Subcommand)]
pub enum Action {
    #[clap(about = "List all nodes", help_template = help_template(HelpOptions::DontPrintCommandSection))]
    List(ListOptions),
    #[clap(about = "Show node details", help_template = help_template(HelpOptions::DontPrintCommandSection))]
    Details(DetailsOptions),
}
