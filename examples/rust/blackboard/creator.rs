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
        .blackboard_creator::<KeyType>()
        .add_with_default::<u64>(0)
        .add::<FixedSizeByteString<30>>(5, "Groovy".try_into()?)
        .add_with_default::<f32>(9)
        .create()?;

    println!("Blackboard created.\n");

    let writer = service.writer_builder().create()?;

    let entry_handle_mut_0 = writer.entry::<u64>(&0)?;
    let mut entry_handle_mut_5 = writer.entry::<FixedSizeByteString<30>>(&5)?;
    let entry_handle_mut_9 = writer.entry::<f32>(&9)?;

    let mut counter = 0;

    while node.wait(CYCLE_TIME).is_ok() {
        counter += 1;

        entry_handle_mut_0.update_with_copy(counter);
        println!("Write new value for key 0: {counter}");

        let entry_value_uninit = entry_handle_mut_5.loan_uninit();
        let value = format!("Funky {}", counter);
        let entry_value =
            entry_value_uninit.write(FixedSizeByteString::<30>::from_bytes(value.as_bytes())?);
        entry_handle_mut_5 = entry_value.update();
        println!("Write new value for key 5: {}", value);

        let value = counter as f32 * 7.7;
        entry_handle_mut_9.update_with_copy(value);
        println!("Write new value for key 9: {value}\n");
    }

    println!("exit");

    Ok(())
}
