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
use iceoryx2::prelude::*;
use iceoryx2_bb_container::vector::StaticVec;
use std::fmt::Debug;

pub struct TestContainers {}

impl TestContainers {
    pub fn new() -> Self {
        TestContainers {}
    }
}

#[derive(Debug, Clone, Copy, ZeroCopySend)]
#[type_name("ContainerTestRequest")]
#[repr(C)]
struct ContainerTestRequest {
    vector_type_sequence: i32,
    container_size: i32,
    container_alignment: i32,
    size_of_data_component: i32,
    offset_of_data_component: i32,
    size_of_size_component: i32,
    offset_of_size_component: i32,
    size_component_type_is_unsigned: bool,
}

#[derive(Debug, Clone, Copy, ZeroCopySend)]
#[type_name("ContainerTestResponse")]
#[repr(C)]
struct ContainerTestResponse {
    vector_type_sequence: i32,
    all_fields_match: bool,
}

#[derive(Debug, Clone, Copy, ZeroCopySend)]
#[type_name("ContainerTestOverAligned")]
#[repr(C)]
#[repr(align(64))]
struct ContainerTestOverAligned {
    i: i32,
}

#[derive(Debug)]
enum VectorTypeSequence {
    VecI32_10 = 1,
    VecI64_20 = 2,
    VecOverAligned5 = 3,
    VecVecI8_10 = 4,
    EndOfTest = -1,
}

fn container_request<T, const CAPACITY: usize>(
    sequence_id: VectorTypeSequence,
) -> ContainerTestRequest {
    let v = StaticVec::<T, CAPACITY>::default();
    let stats = iceoryx2_bb_container::vector::VectorMemoryLayoutMetrics::from_vector(&v);
    assert!(stats.vector_size < i32::MAX as usize);
    assert!(stats.vector_alignment < i32::MAX as usize);
    assert!(stats.size_data < i32::MAX as usize);
    assert!(stats.offset_data < i32::MAX as usize);
    assert!(stats.size_len < i32::MAX as usize);
    assert!(stats.offset_len < i32::MAX as usize);
    ContainerTestRequest {
        vector_type_sequence: sequence_id as i32,
        container_size: stats.vector_size as i32,
        container_alignment: stats.vector_alignment as i32,
        size_of_data_component: stats.size_data as i32,
        offset_of_data_component: stats.offset_data as i32,
        size_of_size_component: stats.size_len as i32,
        offset_of_size_component: stats.offset_len as i32,
        size_component_type_is_unsigned: stats.len_is_unsigned,
    }
}

fn request_end_of_test() -> ContainerTestRequest {
    ContainerTestRequest {
        vector_type_sequence: VectorTypeSequence::EndOfTest as i32,
        container_size: 0,
        container_alignment: 0,
        size_of_data_component: 0,
        offset_of_data_component: 0,
        size_of_size_component: 0,
        offset_of_size_component: 0,
        size_component_type_is_unsigned: false,
    }
}

impl ComponentTest for TestContainers {
    fn test_name(&self) -> &'static str {
        "containers"
    }

    fn run_test(&mut self, node: &Node<ipc::Service>) -> Result<(), Box<dyn core::error::Error>> {
        let cycle_time = core::time::Duration::from_millis(100);
        let sb = node.service_builder(&ServiceName::new(&format!(
            "iox2-component-tests-{}",
            self.test_name()
        ))?);
        let service = sb
            .request_response::<ContainerTestRequest, ContainerTestResponse>()
            .open_or_create()?;

        let client = service.client_builder().create()?;
        wait_for_pred(
            node,
            &|| service.dynamic_config().number_of_servers() > 0,
            core::time::Duration::from_secs(2),
            cycle_time,
        )?;
        for test in [
            VectorTypeSequence::VecI32_10,
            VectorTypeSequence::VecI64_20,
            VectorTypeSequence::VecOverAligned5,
            VectorTypeSequence::VecVecI8_10,
            VectorTypeSequence::EndOfTest,
        ] {
            println!("       * Requesting {:?}", test);
            let request = match test {
                VectorTypeSequence::VecI32_10 => container_request::<i32, 10>(test),
                VectorTypeSequence::VecI64_20 => container_request::<i64, 20>(test),
                VectorTypeSequence::VecOverAligned5 => {
                    container_request::<ContainerTestOverAligned, 5>(test)
                }
                VectorTypeSequence::VecVecI8_10 => container_request::<StaticVec<i8, 10>, 10>(test),
                VectorTypeSequence::EndOfTest => request_end_of_test(),
            };
            let pending_response = client.send_copy(request)?;
            let response = await_response(
                node,
                &pending_response,
                core::time::Duration::from_secs(5),
                cycle_time,
            )?;
            if (response.vector_type_sequence != request.vector_type_sequence)
                || (!response.all_fields_match)
            {
                println!("Invalid response from component test server");
                return Err(Box::new(GenericTestError {}));
            }
        }
        Ok(())
    }
}
