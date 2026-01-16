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

extern crate alloc;
use alloc::boxed::Box;
use alloc::string::String;

use clap::Parser;
use iceoryx2::prelude::*;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let args = parse_args();

    // create a new config based on the global config
    let mut config = Config::global_config().clone();

    // The domain name becomes the prefix for all resources.
    // Therefore, different domain names never share the same resources.
    config.global.prefix = FileName::new(args.domain.as_bytes())?;

    cout!("\nServices running in domain \"{}\":", args.domain);

    // use the custom config when listing the services
    ipc::Service::list(&config, |service| {
        cout!("  {}", &service.static_details.name());
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
}

fn parse_args() -> Args {
    let args = Args::parse();
    args
}
