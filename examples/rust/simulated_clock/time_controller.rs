// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use std::{
    io::{Read, Write},
    time::Duration,
};

use iceoryx2::prelude::*;
use iceoryx2_cal::clock::TimeController;

enum PromptResult {
    WaitingTime(f32),
    Exit,
}

fn prompt_for_next_timeout(
    previous_value: f32,
) -> Result<PromptResult, Box<dyn core::error::Error>> {
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    print!(" > enter time advance in ms [{previous_value}] ");
    stdout.flush().expect("stdout is writable");

    let mut byte = [0u8; 1];
    stdin.read_exact(&mut byte)?;
    if byte[0] == b'e' || byte[0] == b'q' {
        return Ok(PromptResult::Exit);
    }

    match String::from_utf8_lossy(&byte).parse::<f32>() {
        Ok(v) => Ok(PromptResult::WaitingTime(v.max(0.0))),
        Err(_) => Ok(PromptResult::WaitingTime(previous_value)),
    }
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let node = NodeBuilder::new().create::<ipc_simulated_clock::Service>()?;
    let time_controller = node.time_controller()?;
    let mut waiting_time_in_ms = 100.0f32;

    println!("type 'q' or 'e' to exit");
    loop {
        match prompt_for_next_timeout(waiting_time_in_ms)? {
            PromptResult::WaitingTime(t) => waiting_time_in_ms = t,
            PromptResult::Exit => break,
        }

        time_controller.advance(Duration::from_secs_f32(waiting_time_in_ms / 1000.0));
    }

    Ok(())
}
