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
use iceoryx2::prelude::*;
use iceoryx2_bb_container::byte_string::FixedSizeByteString;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    type KeyType = u32;
    let service = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .blackboard_opener::<KeyType>()
        .open()?;

    let reader = service.reader_builder().create()?;

    let entry_handle_0 = reader.entry::<u64>(&0)?;
    let entry_handle_5 = reader.entry::<FixedSizeByteString<30>>(&5)?;
    let entry_handle_9 = reader.entry::<f32>(&9)?;

    while node.wait(CYCLE_TIME).is_ok() {
        println!("read values:");

        println!("key: 0, value: {}", entry_handle_0.get());
        println!("key: 5, value: {}", entry_handle_5.get());
        println!("key: 9, value: {}\n", entry_handle_9.get());
    }

    println!("exit");

    Ok(())
}
