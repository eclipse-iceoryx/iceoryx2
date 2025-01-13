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
    fn basic_listener_test<S: Service + ServiceTypeMapping>() {
        unsafe {
            let node_handle = create_node::<S>("bar");

            let event_service_handle = create_event_service(&node_handle, "all/glory/to/hypnotaod");

            let listener_builder_handle = iox2_port_factory_event_listener_builder(
                &event_service_handle,
                core::ptr::null_mut(),
            );

            let mut listener_handle = core::ptr::null_mut();
            let ret_val = iox2_port_factory_listener_builder_create(
                listener_builder_handle,
                core::ptr::null_mut(),
                &mut listener_handle,
            );
            assert_that!(ret_val, eq(IOX2_OK));

            iox2_listener_drop(listener_handle);
            iox2_port_factory_event_drop(event_service_handle);
            iox2_node_drop(node_handle);
        }
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
