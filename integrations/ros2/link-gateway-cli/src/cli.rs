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

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use iceoryx2_cli::help_template;

#[derive(Parser)]
#[command(
    name = "iox2 link gateway ros2",
    bin_name = "iox2 link gateway ros2",
    about = "Launch an iceoryx2 gateway to ROS 2.",
    long_about = None,
    version = env!("CARGO_PKG_VERSION"),
    help_template = help_template().build(),
)]
pub struct Cli {
    #[clap(
        long = "allow",
        short = 'a',
        value_name = "TOPIC",
        action = clap::ArgAction::Append,
        conflicts_with = "static_mapping",
        help = "Bridge the ROS 2 topics matching this wildcard pattern. '*' matches zero or \
                more characters and '?' matches one. Repeatable. When omitted, the gateway \
                attempts to bridge all topics except /rosout and /parameter_events."
    )]
    pub allow: Vec<String>,

    #[clap(
        long,
        value_name = "TOML",
        help = "Map iceoryx2 services to ROS 2 topics as specified in this file. When \
                omitted, services named ros2://topics/{NAMESPACE}/{TOPIC} are mapped to the \
                topics /{NAMESPACE}/{TOPIC}."
    )]
    static_mapping: Option<PathBuf>,

    #[clap(
        long = "preload-type",
        value_name = "TYPE",
        action = clap::ArgAction::Append,
        conflicts_with = "static_mapping",
        help = "Load the typesupport of this ROS 2 message type at startup. The gateway fails \
                to start when it cannot be loaded. Repeatable."
    )]
    pub preload_types: Vec<String>,

    #[clap(
        long,
        value_enum,
        default_value_t = Translator::Passthrough,
        help = "Payload translation strategy."
    )]
    pub translator: Translator,

    #[clap(
        long,
        help = "Create the services mirroring ROS 2 topics with the RosHeader user header. \
                It carries the ROS 2 message info of each sample."
    )]
    pub ros_header: bool,

    #[clap(
        long,
        value_name = "RATE",
        help = "Wake the gateway every RATE milliseconds. Defaults to 100 when neither \
                --reactive nor --listener is given."
    )]
    pub poll: Option<u64>,

    #[clap(long, help = "Wake the gateway when ROS 2 has new data or endpoints.")]
    pub reactive: bool,

    #[clap(
        long,
        help = "Notify the event service named after a service whenever samples from ROS 2 \
                were delivered to it."
    )]
    pub notify: bool,

    #[clap(
        long,
        value_name = "EVENT_SERVICE",
        help = "Wake the gateway when this iceoryx2 event service is notified. Repeatable."
    )]
    pub listener: Vec<String>,

    #[clap(
        long,
        help = "Report what every bridge moved after each propagation at trace log level."
    )]
    pub monitor: bool,
}

impl Cli {
    /// The selected service-to-topic mapping.
    pub fn mapping(&self) -> Mapping {
        match &self.static_mapping {
            Some(config) => Mapping::Static(config.clone()),
            None => Mapping::Prefix,
        }
    }
}

/// How iceoryx2 services are mapped onto ROS 2 topics.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Mapping {
    /// By the `ros2://topics/` name prefix.
    Prefix,
    /// Per the entries of the static mapping TOML at the given path.
    Static(PathBuf),
}

#[derive(ValueEnum, Debug, Clone, Copy, Eq, PartialEq)]
#[value(rename_all = "PascalCase")]
pub enum Translator {
    /// Payload bytes cross unmodified, the CDR of the ROS 2 type as a byte
    /// slice.
    Passthrough,
    /// (De)serializes payloads at the boundary to ROS 2 using the ROS 2
    /// typesupport libraries. Only supports fixed-sized structs that can be
    /// placed in shared memory.
    PlainStruct,
}
