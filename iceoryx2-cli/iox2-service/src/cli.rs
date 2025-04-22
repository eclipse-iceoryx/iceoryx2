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

use iceoryx2_cli::filter::MessagingPatternFilter;
use iceoryx2_cli::help_template;
use iceoryx2_cli::Format;

#[derive(Parser)]
#[command(
    name = "iox2-service",
    about = "Query information about iceoryx2 services",
    long_about = None,
    version = env!("CARGO_PKG_VERSION"),
    disable_help_subcommand = true,
    arg_required_else_help = false,
    help_template = help_template("iox2 service", false),
)]
pub struct Cli {
    #[clap(subcommand)]
    pub action: Option<Action>,

    #[clap(long, short = 'f', value_enum, global = true, value_enum, default_value_t = Format::Ron)]
    pub format: Format,
}

#[derive(Debug, Clone, Args)]
pub struct OutputFilter {
    #[clap(short, long, value_enum, default_value_t = MessagingPatternFilter::All)]
    pub pattern: MessagingPatternFilter,
}

#[derive(Args)]
pub struct ListOptions {
    #[command(flatten)]
    pub filter: OutputFilter,
}

#[derive(Parser)]
pub struct DetailsOptions {
    #[clap(help = "Name of the service e.g. \"My Service\"")]
    pub service: String,

    #[command(flatten)]
    pub filter: OutputFilter,
}

#[derive(Parser)]
pub struct MonitorOptions {
    #[clap(
        long,
        help = "Name to use for the service discovery service",
        default_value = "iox2://monitor/services"
    )]
    pub service_name: String,

    #[clap(
        short,
        long,
        default_value = "1000",
        help = "Update rate in milliseconds"
    )]
    pub rate: u64,

    #[clap(long, help = "Do not publish details of detected changes")]
    pub disable_publish: bool,

    #[clap(long, default_value = "10", help = "The maximum number of subscribers")]
    pub max_subscribers: usize,

    #[clap(long, help = "Do not notify when changes detected")]
    pub disable_notify: bool,

    #[clap(long, default_value = "10", help = "The maximum number of listeners")]
    pub max_listeners: usize,
}

#[derive(Subcommand)]
pub enum Action {
    #[clap(
        about = "List all services",
        help_template = help_template("iox2 service list", false)
    )]
    List(ListOptions),
    #[clap(
        about = "Show service details",
        help_template = help_template("iox2 service details", false)
    )]
    Details(DetailsOptions),
    #[clap(
        about = "Start a service monitor", 
        help_template = help_template("iox2 service monitor", false)
    )]
    Monitor(MonitorOptions),
}
