// Copyright (c) 2024 Contributors to the Eclipse Foundation
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
use examples_common::{open_service, PubSubEvent};
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_millis(1000);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let service_name = ServiceName::new("service_1")?;
    let node = NodeBuilder::new()
        .name(&"publisher 1".try_into()?)
        .create::<ipc::Service>()?;

    let (service_event, service_pubsub) = open_service(&node, &service_name)?;

    let publisher = service_pubsub.publisher_builder().create()?;
    let notifier = service_event
        .notifier_builder()
        // we only want to notify the other side explicitly when we have sent a sample
        // so we can define it as default event id
        .default_event_id(PubSubEvent::SentSample.into())
        .create()?;
    let mut counter: u64 = 0;

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;

    // we need to send out a sample with an interval of CYCLE_TIME, therefore we attach an
    // interval that wakes us up regularly to send out the next sample
    let _cycle_guard = waitset.attach_interval(CYCLE_TIME);

    waitset.wait_and_process(|_| {
        println!("{service_name}: Send sample {counter} ...");
        publisher
            .send_copy(counter)
            .expect("sample delivery successful.");
        notifier.notify().expect("notification successful.");
        counter += 1;
        CallbackProgression::Continue
    })?;

    println!("exit");

    Ok(())
}
