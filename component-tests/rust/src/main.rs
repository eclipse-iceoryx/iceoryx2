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

#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

mod common;
mod test_container_mutation;
mod test_containers;

use iceoryx2::prelude::*;
use iceoryx2_bb_container::string::StaticString;

fn component_tests() -> Vec<Box<dyn common::ComponentTest>> {
    vec![
        Box::new(test_containers::TestContainers::new()),
        Box::new(test_container_mutation::TestContainerMutation::new()),
    ]
}

#[derive(Debug, Clone, Copy, ZeroCopySend)]
#[type_name("ComponentTestHeader")]
#[repr(C)]
pub struct ComponentTestHeader {
    pub test_name: StaticString<32>,
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    println!("*** Component Tests Rust ***");

    let node = NodeBuilder::new().create::<ipc::Service>()?;
    let service = node
        .service_builder(&"iox2-component-tests".try_into()?)
        .publish_subscribe::<ComponentTestHeader>()
        .open_or_create()?;
    let publisher = service.publisher_builder().create()?;

    println!("Waiting for clients to connect...");
    while service.dynamic_config().number_of_subscribers() < 1 {
        node.wait(core::time::Duration::from_millis(100))?;
    }

    for mut boxed_test in component_tests() {
        let t = boxed_test.as_mut();
        let sample = publisher.loan_uninit()?;
        sample
            .write_payload(ComponentTestHeader {
                test_name: StaticString::from_str_truncated(t.test_name())?,
            })
            .send()?;
        println!("   - Running test {}...", t.test_name());
        t.run_test(&node)?;
        println!("     OK.");
    }

    Ok(())
}
