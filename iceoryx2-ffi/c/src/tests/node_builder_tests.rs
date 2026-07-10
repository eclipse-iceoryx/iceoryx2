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

mod node_creation_failure_string {
    use crate::api::*;
    use core::ffi::CStr;
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn converts_upper_snake_case_to_words() {
        unsafe {
            let internal_error = CStr::from_ptr(iox2_node_creation_failure_string(
                iox2_node_creation_failure_e::INTERNAL_ERROR,
            ))
            .to_str()
            .unwrap();
            assert_that!(internal_error, eq("internal error"));

            let insufficient_permissions = CStr::from_ptr(iox2_node_creation_failure_string(
                iox2_node_creation_failure_e::INSUFFICIENT_PERMISSIONS,
            ))
            .to_str()
            .unwrap();
            assert_that!(insufficient_permissions, eq("insufficient permissions"));
        }
    }
}

#[generic_tests::define]
mod node_builder {
    use crate::api::*;
    use crate::tests::ServiceTypeMapping;
    use iceoryx2::prelude::*;
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn basic_node_builder_api_test<S: Service + ServiceTypeMapping>() {
        unsafe {
            let node_builder_handle = iox2_node_builder_new(core::ptr::null_mut());
            let mut node_handle: iox2_node_h = core::ptr::null_mut();
            let ret_val = iox2_node_builder_create(
                node_builder_handle,
                core::ptr::null_mut(),
                S::service_type(),
                &mut node_handle as *mut iox2_node_h,
            );

            assert_that!(ret_val, eq(IOX2_OK));
            assert_that!(node_handle, ne(core::ptr::null_mut()));

            iox2_node_drop(node_handle);
        }
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
