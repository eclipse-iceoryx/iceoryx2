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

#[derive(Parser)]
#[command(
    name = "iox2-services",
    about = "Query information about iceoryx2 services.",
    long_about = None,
    version = env!("CARGO_PKG_VERSION"),
    disable_help_subcommand = true,
    arg_required_else_help = false,
    help_template = help_template("iox2-services", false),
)]
pub struct Cli {
    #[clap(subcommand)]
    pub action: Option<Action>,
}

#[derive(Parser)]
pub struct DetailsOptions {
    #[clap(long = "static", short = 's')]
    pub static_flag: bool,
    #[clap(long, short = 'd')]
    pub dynamic: bool,
    pub service: String,
}

pub enum DetailsFilter {
    None,
    Static,
    Dynamic,
}

impl From<&DetailsOptions> for DetailsFilter {
    fn from(options: &DetailsOptions) -> Self {
        match (options.static_flag, options.dynamic) {
            (true, false) => DetailsFilter::Static,
            (false, true) => DetailsFilter::Dynamic,
            _ => DetailsFilter::None,
        }
    }
}

#[derive(Subcommand)]
pub enum Action {
    List,
    Details(DetailsOptions),
}
