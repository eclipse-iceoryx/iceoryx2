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

use super::common::*;
use core::fmt::Debug;
use iceoryx2::prelude::*;

pub struct TestContainerMutation {}

impl TestContainerMutation {
    pub fn new() -> Self {
        TestContainerMutation {}
    }
}

#[derive(Debug, Clone, ZeroCopySend)]
#[type_name("ContainerMutationTestRequest")]
#[repr(C)]
struct ContainerMutationTestRequest {
    vector_add_element: iceoryx2_bb_container::vector::StaticVec<i32, 10>,
    vector_remove_element: iceoryx2_bb_container::vector::StaticVec<i32, 10>,
    string_append: iceoryx2_bb_container::string::StaticString<64>,
    vector_strings_change_middle: iceoryx2_bb_container::vector::StaticVec<
        iceoryx2_bb_container::string::StaticString<16>,
        5,
    >,
}

#[derive(Debug, Clone, ZeroCopySend)]
#[type_name("ContainerMutationTestResponse")]
#[repr(C)]
struct ContainerMutationTestResponse {
    vector_add_element: iceoryx2_bb_container::vector::StaticVec<i32, 10>,
    vector_remove_element: iceoryx2_bb_container::vector::StaticVec<i32, 10>,
    string_append: iceoryx2_bb_container::string::StaticString<64>,
    vector_strings_change_middle: iceoryx2_bb_container::vector::StaticVec<
        iceoryx2_bb_container::string::StaticString<16>,
        5,
    >,
}

impl ComponentTest for TestContainerMutation {
    fn test_name(&self) -> &'static str {
        "container_mutation"
    }

    fn run_test(&mut self, node: &Node<ipc::Service>) -> Result<(), Box<dyn core::error::Error>> {
        use iceoryx2_bb_container::vector::Vector;
        let cycle_time = core::time::Duration::from_millis(100);
        let sb = node.service_builder(&ServiceName::new(&format!(
            "iox2-component-tests-{}",
            self.test_name()
        ))?);
        let service = sb
            .request_response::<ContainerMutationTestRequest, ContainerMutationTestResponse>()
            .open_or_create()?;

        let client = service.client_builder().create()?;
        wait_for_pred(
            node,
            &|| service.dynamic_config().number_of_servers() > 0,
            core::time::Duration::from_secs(2),
            cycle_time,
        )?;
        let mut request = ContainerMutationTestRequest {
            vector_add_element: iceoryx2_bb_container::vector::StaticVec::default(),
            vector_remove_element: iceoryx2_bb_container::vector::StaticVec::default(),
            string_append: iceoryx2_bb_container::string::StaticString::from_str_truncated(
                "Hello",
            )?,
            vector_strings_change_middle: iceoryx2_bb_container::vector::StaticVec::default(),
        };
        request.vector_add_element.push(1)?;
        request.vector_add_element.push(2)?;
        request.vector_add_element.push(3)?;
        request.vector_add_element.push(4)?;
        request.vector_remove_element.push(1)?;
        request.vector_remove_element.push(2)?;
        request.vector_remove_element.push(9999)?;
        request.vector_remove_element.push(3)?;
        request.vector_remove_element.push(4)?;
        request.vector_remove_element.push(9999)?;
        request.vector_remove_element.push(5)?;
        request.vector_remove_element.push(9999)?;
        request
            .vector_strings_change_middle
            .push(iceoryx2_bb_container::string::StaticString::from_str_truncated("Howdy!")?)?;
        request
            .vector_strings_change_middle
            .push(iceoryx2_bb_container::string::StaticString::from_str_truncated("Yeehaw!")?)?;
        request.vector_strings_change_middle.push(
            iceoryx2_bb_container::string::StaticString::from_str_truncated("How's the missus")?,
        )?;
        request.vector_strings_change_middle.push(
            iceoryx2_bb_container::string::StaticString::from_str_truncated("I'll be gone")?,
        )?;
        request.vector_strings_change_middle.push(
            iceoryx2_bb_container::string::StaticString::from_str_truncated("See you soon")?,
        )?;
        let pending_response = client.send_copy(request)?;
        let response = await_response(
            node,
            &pending_response,
            core::time::Duration::from_secs(5),
            cycle_time,
        )?;
        if !check_response(response.payload()) {
            return Err(Box::new(GenericTestError {}));
        }

        Ok(())
    }
}

fn check_response(res: &ContainerMutationTestResponse) -> bool {
    if !res.vector_add_element.iter().eq([1, 2, 3, 4, 123].iter()) {
        println!(
            "Unexpected value for vector_add_element {:?}",
            res.vector_add_element
        );
        return false;
    }
    if !res.vector_remove_element.iter().eq([1, 2, 3, 4, 5].iter()) {
        println!(
            "Unexpected value for vector_remove_element {:?}",
            res.vector_remove_element
        );
        return false;
    }
    if res.string_append != "Hello my baby, hello my honey, hello my ragtime gal" {
        println!("Unexpected value for string_append {:?}", res.string_append);
        return false;
    }
    if !res.vector_strings_change_middle.iter().len() == 5
        || (*res.vector_strings_change_middle.first().unwrap() != "Howdy!")
        || (*res.vector_strings_change_middle.get(1).unwrap() != "Yeehaw!")
        || (*res.vector_strings_change_middle.get(2).unwrap() != "How's the mister")
        || (*res.vector_strings_change_middle.get(3).unwrap() != "I'll be gone")
        || (*res.vector_strings_change_middle.get(4).unwrap() != "See you soon")
    {
        println!(
            "Unexpected value for vector_strings_change_middle {:?}",
            res.vector_strings_change_middle
        );
        return false;
    }
    true
}
