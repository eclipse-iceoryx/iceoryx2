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

use core::time::Duration;

extern crate alloc;
use alloc::boxed::Box;

use examples_common::BlackboardKey;
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let key_0 = BlackboardKey { x: 0, y: -4, z: 4 };
    let key_1 = BlackboardKey { x: 1, y: -4, z: 4 };
    let service = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .blackboard_opener::<BlackboardKey>()
        .open()?;

    let reader = service.reader_builder().create()?;

    let entry_handle_0 = reader.entry::<i32>(&key_0)?;
    let entry_handle_1 = reader.entry::<f64>(&key_1)?;

    while node.wait(CYCLE_TIME).is_ok() {
        cout!("read values:");

        cout!("key: 0, value: {}", entry_handle_0.get());
        cout!("key: 1, value: {}\n", entry_handle_1.get());
    }

    cout!("exit");

    Ok(())
}
