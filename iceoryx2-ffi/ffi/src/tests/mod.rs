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

mod node_builder_tests;
mod node_name_tests;
mod node_tests;
mod service_name_tests;

use crate::*;
use iceoryx2::prelude::*;
use iceoryx2_bb_testing::assert_that;

trait ServiceTypeMapping {
    fn service_type() -> iox2_service_type_e;
}

impl ServiceTypeMapping for iceoryx2::service::zero_copy::Service {
    fn service_type() -> iox2_service_type_e {
        iox2_service_type_e::IPC
    }
}

impl ServiceTypeMapping for iceoryx2::service::process_local::Service {
    fn service_type() -> iox2_service_type_e {
        iox2_service_type_e::LOCAL
    }
}

fn create_node<S: Service + ServiceTypeMapping>(node_name: &str) -> iox2_node_h {
    unsafe {
        let node_builder_handle = iox2_node_builder_new(std::ptr::null_mut());

        let mut node_name_handle = std::ptr::null_mut();
        let ret_val = iox2_node_name_new(
            std::ptr::null_mut(),
            node_name.as_ptr() as *const _,
            node_name.len() as _,
            &mut node_name_handle,
        );
        assert_that!(ret_val, eq(IOX2_OK));
        iox2_node_builder_set_name(
            iox2_cast_node_builder_ref_h(node_builder_handle),
            iox2_cast_node_name_ptr(node_name_handle),
        );
        iox2_node_name_drop(node_name_handle);

        let mut node_handle: iox2_node_h = std::ptr::null_mut();
        let ret_val = iox2_node_builder_create(
            node_builder_handle,
            std::ptr::null_mut(),
            S::service_type(),
            &mut node_handle as *mut iox2_node_h,
        );

        assert_that!(ret_val, eq(IOX2_OK));

        node_handle
    }
}
