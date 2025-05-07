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

mod cli;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use cli::Transport;

use iceoryx2::prelude::*;
use iceoryx2_bb_log::set_log_level_from_env_or;
use iceoryx2_bb_log::LogLevel;

use iceoryx2_tunnels_zenoh::ZenohTunnel;

fn main() -> Result<()> {
    #[cfg(not(debug_assertions))]
    {
        setup_panic!();
    }
    #[cfg(debug_assertions)]
    {
        better_panic::Settings::debug()
            .most_recent_first(false)
            .lineno_suffix(true)
            .verbosity(better_panic::Verbosity::Full)
            .install();
    }

    set_log_level_from_env_or(LogLevel::Warn);

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // --help and --version is treated as a parse error by clap
            // printing the error is actually printing the result of those commands ...
            let _ = e.print();
            return Ok(());
        }
    };

    if let Some(transport) = cli.transport {
        match transport {
            Transport::Zenoh(_options) => {
                const RATE_MS: u64 = 100;

                let mut tunnel = ZenohTunnel::new();
                tunnel.setup();

                let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
                let guard = waitset.attach_interval(core::time::Duration::from_millis(RATE_MS))?;
                let tick = WaitSetAttachmentId::from_guard(&guard);

                let on_event = |id: WaitSetAttachmentId<ipc::Service>| {
                    if id == tick {
                        tunnel.spin();
                    }
                    CallbackProgression::Continue
                };

                waitset.wait_and_process(on_event)?;

                tunnel.shutdown();
            }
        }
    }

    Ok(())
}
