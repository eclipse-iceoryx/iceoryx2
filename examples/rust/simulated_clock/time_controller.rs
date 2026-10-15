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

use std::io::{Read, Write};

use iceoryx2::prelude::*;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let _node = NodeBuilder::new().create::<ipc_simulated_clock::Service>()?;

    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut previous_value = 100.0f32;
    let mut prompt = |v| {
        print!(" > enter time advance in ms [{v}] ");
        stdout.flush().expect("stdout is writable");
    };

    prompt(previous_value);

    loop {
        let mut input = String::new();
        let mut byte = [0u8; 1];

        'read: loop {
            stdin.read_exact(&mut byte)?;
            match byte[0] {
                b'\n' => break 'read,
                c => input.push(c as char),
            }
        }

        let text = input.trim();
        previous_value = match text.parse::<f32>() {
            Ok(v) => v,
            Err(_) => previous_value,
        };

        // todo handle clock here

        prompt(previous_value);
    }
}
