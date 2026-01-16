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

    let publisher = service.publisher_builder().create()?;

    let mut counter: u64 = 0;

    while node.wait(CYCLE_TIME).is_ok() {
        counter += 1;
        let mut sample = publisher.loan_uninit()?;

        sample
            .user_header_mut()
            .push_bytes(b"Are Hypnotoad and Kermit related like Fry and the Professor?")?;
        let sample = sample.write_payload(StaticVec::try_from(
            [counter, counter + 1, counter + 2].as_slice(),
        )?);

        sample.send()?;

        cout!("Send sample {counter} ...");
    }

    cout!("exit");

    Ok(())
}
