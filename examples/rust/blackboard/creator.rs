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
    const INITIAL_VALUE_1: f64 = 1.1;
    let service = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .blackboard_creator::<BlackboardKey>()
        .add::<i32>(key_0, 3)
        .add::<f64>(key_1, INITIAL_VALUE_1)
        .create()?;

    cout!("Blackboard created.\n");

    let writer = service.writer_builder().create()?;

    let entry_handle_mut_0 = writer.entry::<i32>(&key_0)?;
    let mut entry_handle_mut_1 = writer.entry::<f64>(&key_1)?;

    let mut counter = 0;

    while node.wait(CYCLE_TIME).is_ok() {
        counter += 1;

        entry_handle_mut_0.update_with_copy(counter);
        cout!("Write new value for key 0: {counter}");

        let entry_value_uninit = entry_handle_mut_1.loan_uninit();
        let value = INITIAL_VALUE_1 * counter as f64;
        entry_handle_mut_1 = entry_value_uninit.update_with_copy(value);
        cout!("Write new value for key 1: {}\n", value);
    }

    cout!("exit");

    Ok(())
}
