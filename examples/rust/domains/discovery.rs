// Copyright (c) 2023 Contributors to the Eclipse Foundation
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
use iceoryx2::prelude::*;
use iceoryx2_bb_log::{set_log_level, LogLevel};

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let args = parse_args();

    // create a new config based on the global config
    let mut config = Config::global_config().clone();

    // The domain name becomes the prefix for all resources.
    // Therefore, different domain names never share the same resources.
    config.global.prefix = FileName::new(args.domain.as_bytes())?;

    println!("\nServices running in domain \"{}\":", args.domain);

    // use the custom config when listing the services
    ipc::Service::list(&config, |service| {
        println!("  {}", &service.static_details.name());
        CallbackProgression::Continue
    })?;

    Ok(())
}

/////////////////////////////////
// uninteresting part, contains
//   * arguments parsing
//   * log level setup
/////////////////////////////////

#[derive(Parser, Debug)]
struct Args {
    /// The name of the domain. Must be a valid file name.
    #[clap(short, long, default_value = "iox2")]
    domain: String,
    /// Enable full debug log output
    #[clap(long, default_value_t = false)]
    debug: bool,
}

fn define_log_level(args: &Args) {
    if args.debug {
        set_log_level(LogLevel::Trace);
    } else {
        set_log_level(LogLevel::Warn);
    }
}

fn parse_args() -> Args {
    let args = Args::parse();
    define_log_level(&args);
    args
}
