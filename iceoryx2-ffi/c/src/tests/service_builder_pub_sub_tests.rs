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

#[generic_tests::define]
mod service_builder {
    use crate::tests::*;

    #[test]
    fn basic_service_builder_pub_sub_test<S: Service + ServiceTypeMapping>() {
        unsafe {
            let node_handle = create_node::<S>("bar");

            let service_name = "all/glory/to/hypnotaod";

            let mut service_name_handle: iox2_service_name_h = core::ptr::null_mut();
            let ret_val = iox2_service_name_new(
                core::ptr::null_mut(),
                service_name.as_ptr() as *const _,
                service_name.len(),
                &mut service_name_handle,
            );
            assert_that!(ret_val, eq(IOX2_OK));

            let service_builder_handle = iox2_node_service_builder(
                &node_handle,
                core::ptr::null_mut(),
                iox2_cast_service_name_ptr(service_name_handle),
            );
            iox2_service_name_drop(service_name_handle);

            let service_builder_handle = iox2_service_builder_pub_sub(service_builder_handle);
            iox2_service_builder_pub_sub_set_max_publishers(&service_builder_handle, 10);
            iox2_service_builder_pub_sub_set_max_subscribers(&service_builder_handle, 10);

            let mut pub_sub_factory: iox2_port_factory_pub_sub_h = core::ptr::null_mut();
            iox2_service_builder_pub_sub_open_or_create(
                service_builder_handle,
                core::ptr::null_mut(),
                &mut pub_sub_factory as *mut _,
            );
            assert_that!(ret_val, eq(IOX2_OK));

            iox2_port_factory_pub_sub_drop(pub_sub_factory);
            iox2_node_drop(node_handle);
        }
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
