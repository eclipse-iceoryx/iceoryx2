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
use core::time::Duration;
use examples_common::TransmissionData;
use iceoryx2::prelude::*;
use iceoryx2_bb_log::{set_log_level, LogLevel};

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let args = parse_args();

    // create a new config based on the global config
    let mut config = Config::global_config().clone();

    // The domain name becomes the prefix for all resources.
    // Therefore, different domain names never share the same resources.
    config.global.prefix = FileName::new(args.domain.as_bytes())?;

    let node = NodeBuilder::new()
        // use the custom config when creating the custom node
        // every service constructed by the node will use this config
        .config(&config)
        .create::<ipc::Service>()?;

    ////////////////////////////////////////////////////////////////
    // from here on it is the publish_subscribe publisher example
    ////////////////////////////////////////////////////////////////
    let service = node
        .service_builder(&args.service.as_str().try_into()?)
        .publish_subscribe::<TransmissionData>()
        .open_or_create()?;

    let publisher = service.publisher_builder().create()?;

    let mut counter: u64 = 0;

    while node.wait(CYCLE_TIME).is_ok() {
        counter += 1;
        let sample = publisher.loan_uninit()?;

        let sample = sample.write_payload(TransmissionData {
            x: counter as i32,
            y: counter as i32 * 3,
            funky: counter as f64 * 812.12,
        });

        sample.send()?;

        println!(
            "[domain: \"{}\", service: \"{}\"] Send sample {} ...",
            args.domain, args.service, counter
        );
    }

    println!("exit");

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
    #[clap(short, long, default_value = "iox2_")]
    domain: String,
    /// The name of the service.
    #[clap(short, long, default_value = "my_funky_service")]
    service: String,
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
