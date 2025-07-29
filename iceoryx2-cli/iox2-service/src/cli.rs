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
use clap::ValueEnum;

use iceoryx2_cli::filter::MessagingPatternFilter;
use iceoryx2_cli::help_template;
use iceoryx2_cli::Format;
use iceoryx2_cli::HelpOptions;

#[derive(Parser)]
#[command(
    name = "iox2 service",
    bin_name = "iox2 service",
    about = "Query information about iceoryx2 services",
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
pub struct DiscoveryOptions {
    #[clap(
        short,
        long,
        default_value = "100",
        help = "Update rate in milliseconds"
    )]
    pub rate: u64,

    #[clap(long, help = "Do not publish discovered services")]
    pub disable_publish: bool,

    #[clap(long, default_value = "10", help = "The maximum number of subscribers")]
    pub max_subscribers: usize,

    #[clap(long, help = "Do not notify of discovered services")]
    pub disable_notify: bool,

    #[clap(long, default_value = "10", help = "The maximum number of listeners")]
    pub max_listeners: usize,
}

#[derive(Parser)]
pub struct NotifyOptions {
    #[clap(help = "Name of the service which shall be notified.")]
    pub service: String,
    #[clap(
        short,
        long,
        default_value = "0",
        help = "EventId value used for the notification."
    )]
    pub event_id: usize,
    #[clap(
        short,
        long,
        default_value = "shell_node",
        help = "Defines the node name of the notification endpoint."
    )]
    pub node_name: String,
    #[clap(
        short = 'u',
        long,
        default_value = "1",
        help = "How often shall the notification be sent."
    )]
    pub num: u64,
    #[clap(
        short,
        long,
        default_value = "250",
        help = "Interval between notifications."
    )]
    pub interval_in_ms: u64,
}

#[derive(Parser)]
pub struct ListenOptions {
    #[clap(help = "Name of the service which shall be waited on for a notification.")]
    pub service: String,
    #[clap(
        short,
        long,
        default_value = "shell_node",
        help = "Defines the node name of the listening endpoint."
    )]
    pub node_name: String,
    #[clap(
        short,
        long,
        default_value = "1000",
        help = "Maximum delay between two notifications. Set to 0 to wait indefinitely."
    )]
    pub timeout_in_ms: u64,
    #[clap(
        short,
        long,
        help = "[Optional] How often shall the notification receive loop be repeated. If its not specified the call will listen indefinitely."
    )]
    pub repetitions: Option<u64>,
}

#[derive(Clone, Copy, ValueEnum)]
#[value(rename_all = "UPPERCASE")]
pub enum CliTypeVariant {
    Dynamic,
    FixedSize,
}

#[derive(Clone, Copy, ValueEnum, Default)]
#[value(rename_all = "UPPERCASE")]
pub enum DataRepresentation {
    Iox2Dump,
    #[default]
    Hex,
}

impl From<DataRepresentation> for iceoryx2_userland_record_and_replay::record::DataRepresentation {
    fn from(value: DataRepresentation) -> Self {
        match value {
            DataRepresentation::Hex => {
                iceoryx2_userland_record_and_replay::record::DataRepresentation::Hex
            }
            DataRepresentation::Iox2Dump => {
                iceoryx2_userland_record_and_replay::record::DataRepresentation::Iox2Dump
            }
        }
    }
}

#[derive(Parser)]
pub struct PublishOptions {
    #[clap(help = "Name of the service which shall the message be sent to.")]
    pub service: String,
    #[clap(
        short,
        long,
        default_value = "shell_node",
        help = "Defines the node name of the publish endpoint."
    )]
    pub node_name: String,
    #[clap(
        short,
        long,
        help = "The messages that shall be sent. Can be multiple messages. If no messages are given stdin is read."
    )]
    pub message: Vec<String>,
    #[clap(short, long, help = "When set, the data from this file is published.")]
    pub input_file: Option<String>,
    #[clap(
        short,
        long,
        default_value = "1000",
        help = "Time between the messages in milliseconds."
    )]
    pub time_between_messages: usize,

    #[clap(
        short,
        long,
        default_value = "HEX",
        help = "Defines how the provided data is encoded."
    )]
    pub data_representation: DataRepresentation,

    #[clap(
        short,
        long,
        default_value = "1",
        help = "How often shall the messages be sent. If `0` is set the messages will be sent indefinitely."
    )]
    pub repetitions: usize,

    #[clap(
        long,
        default_value = "4096",
        help = "It defines the initial payload size for dynamic type variants."
    )]
    pub initial_payload_size: usize,

    #[clap(
        long,
        default_value = "u8",
        help = "Defines the unique type identifier of the services type."
    )]
    pub type_name: String,
    #[clap(
        long,
        default_value = "1",
        help = "Defines the type size of the services type."
    )]
    pub type_size: usize,
    #[clap(
        long,
        default_value = "1",
        help = "Defines the type alignment of the services type."
    )]
    pub type_alignment: usize,
    #[clap(long, default_value = "DYNAMIC", help = "Defines variant.")]
    pub type_variant: CliTypeVariant,

    #[clap(
        long,
        default_value = "()",
        help = "Defines the unique type identifier of the services user header type."
    )]
    pub header_type_name: String,
    #[clap(
        long,
        default_value = "0",
        help = "Defines the type size of the services user header type."
    )]
    pub header_type_size: usize,
    #[clap(
        long,
        default_value = "1",
        help = "Defines the type alignment of the services user header type."
    )]
    pub header_type_alignment: usize,
}

#[derive(Parser)]
pub struct SubscribeOptions {
    #[clap(help = "Name of the service which shall be waited on for a message.")]
    pub service: String,
    #[clap(
        short,
        long,
        default_value = "shell_node",
        help = "Defines the node name of the subscriber endpoint."
    )]
    pub node_name: String,

    #[clap(
        short,
        long,
        default_value = "HEX",
        help = "Defines how the data shall be displayed."
    )]
    pub data_representation: DataRepresentation,

    #[clap(
        short,
        long,
        help = "Maximum runtime in milliseconds. When the timeout has passed the process stops."
    )]
    pub timeout: Option<u64>,

    #[clap(
        short,
        long,
        help = "Maximum number of messages to be received before the process stops."
    )]
    pub max_messages: Option<u64>,

    #[clap(
        short,
        long,
        help = "When set, the received data is additionally added to the provided file."
    )]
    pub output_file: Option<String>,

    #[clap(action, short, long, help = "Do not show the received data.")]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Action {
    #[clap(
        about = "List all services",
        help_template = help_template(HelpOptions::DontPrintCommandSection)
    )]
    List(ListOptions),
    #[clap(
        about = "Show service details",
        help_template = help_template(HelpOptions::DontPrintCommandSection)
    )]
    Details(DetailsOptions),
    #[clap(
        about = "Runs the service discovery service within a process",
        help_template = help_template(HelpOptions::DontPrintCommandSection)
    )]
    Discovery(DiscoveryOptions),
    #[clap(
        about = "Send a notification",
        help_template = help_template(HelpOptions::DontPrintCommandSection)
    )]
    Notify(NotifyOptions),
    #[clap(
        about = "Wait until a notification arrives",
        help_template = help_template(HelpOptions::DontPrintCommandSection)
    )]
    Listen(ListenOptions),
    #[clap(
        about = "Publish a message to any service.",
        help_template = help_template(HelpOptions::DontPrintCommandSection)
    )]
    Publish(PublishOptions),
    #[clap(
        about = "Subscribe to any service and introspect its messages.",
        help_template = help_template(HelpOptions::DontPrintCommandSection)
    )]
    Subscribe(SubscribeOptions),
}
