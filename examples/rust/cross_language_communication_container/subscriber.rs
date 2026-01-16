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

use iceoryx2::prelude::*;
use iceoryx2_bb_container::string::*;
use iceoryx2_bb_container::vector::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&"CrossLanguageContainer".try_into()?)
        .publish_subscribe::<StaticVec<u64, 32>>()
        .user_header::<StaticString<64>>()
        // add some QoS, disable safe overflow and the subscriber shall get the
        // last 5 samples when connecting to the service
        .history_size(5)
        .subscriber_max_buffer_size(5)
        .enable_safe_overflow(false)
        .open_or_create()?;

    let subscriber = service.subscriber_builder().create()?;

    cout!("Subscriber ready to receive data!");

    while node.wait(CYCLE_TIME).is_ok() {
        while let Some(sample) = subscriber.receive()? {
            cout!(
                "received: {:?}, user_header: {:?}",
                *sample,
                sample.user_header()
            );
        }
    }

    cout!("exit");

    Ok(())
}
