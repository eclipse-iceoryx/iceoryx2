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

// open: key type must fit + max_readers >= creation::max_readers
// add only on creation
use iceoryx2::{
    prelude::*,
    service::builder::blackboard::{BlackboardCreateError, BlackboardOpenError},
};
use iceoryx2_bb_container::byte_string::FixedSizeByteString;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Trace);
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .blackboard::<u32>()
        .max_readers(10)
        .add::<f32>(5, 55.0)
        .add::<FixedSizeByteString<100>>(99, "Hello World".try_into()?)
        .open_or_create()?;

    println!("max readers: {}", service.static_config().max_readers());
    println!("max writers: {}", service.static_config().max_writers());
    println!("max nodes: {}", service.static_config().max_nodes());

    let result = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .blackboard::<u32>()
        .max_readers(10)
        .create();
    assert!(result.is_err());
    assert_eq!(result.err(), Some(BlackboardCreateError::AlreadyExists));

    let opened_service = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .blackboard::<u32>()
        .max_readers(10)
        //.add::<f32>(5, 55.0)
        //.add::<FixedSizeByteString<100>>(99, "Hello World".try_into()?)
        .open()?;

    println!(
        "max readers: {}",
        opened_service.static_config().max_readers()
    );
    println!(
        "max writers: {}",
        opened_service.static_config().max_writers()
    );
    println!("max nodes: {}", opened_service.static_config().max_nodes());

    let result = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .blackboard::<i32>()
        .max_readers(10)
        .open();
    assert!(result.is_err());
    assert_eq!(result.err(), Some(BlackboardOpenError::IncompatibleKeys));

    let result = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .blackboard::<u32>()
        .max_readers(20)
        .open();
    assert!(result.is_err());

    Ok(())
}
