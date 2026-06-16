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

extern crate alloc;

use alloc::boxed::Box;
use core::time::Duration;
use flatbuffers::FlatBufferBuilder;
use iceoryx2::prelude::*;
use std::marker::PhantomData;

use crate::unbounded_data_generated::example::UnboundedData;

#[path = "unbounded_data_generated.rs"]
mod unbounded_data_generated;

const CYCLE_TIME: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub struct Flatbuffer<T> {
    _data: PhantomData<T>,
}

unsafe impl<T> ZeroCopySend for Flatbuffer<T> {}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&"My/Flatbuffer/Service".try_into()?)
        .publish_subscribe::<Flatbuffer<UnboundedData>>()
        .open_or_create()?;

    let publisher = service.publisher_builder().create()?;

    let mut counter: u64 = 0;

    while node.wait(CYCLE_TIME).is_ok() {
        counter += 1;
        let sample = publisher.loan_uninit()?;

        coutln!("Send sample {counter} ...");
    }

    coutln!("exit");

    Ok(())
}
