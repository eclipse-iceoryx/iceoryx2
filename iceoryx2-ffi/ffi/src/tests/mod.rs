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

mod iceoryx2_settings_tests;
mod listener_tests;
mod node_builder_tests;
mod node_name_tests;
mod node_tests;
mod notifier_tests;
mod service_builder_event_tests;
mod service_builder_pub_sub_tests;
mod service_name_tests;

use crate::*;
use iceoryx2::prelude::*;
use iceoryx2_bb_testing::assert_that;

trait ServiceTypeMapping {
    fn service_type() -> iox2_service_type_e;
}

impl ServiceTypeMapping for iceoryx2::service::ipc::Service {
    fn service_type() -> iox2_service_type_e {
        iox2_service_type_e::IPC
    }
}

impl ServiceTypeMapping for iceoryx2::service::local::Service {
    fn service_type() -> iox2_service_type_e {
        iox2_service_type_e::LOCAL
    }
}

fn create_node<S: Service + ServiceTypeMapping>(node_name: &str) -> iox2_node_h {
    unsafe {
        let node_builder_handle = iox2_node_builder_new(core::ptr::null_mut());

        let mut node_name_handle = core::ptr::null_mut();
        let ret_val = iox2_node_name_new(
            core::ptr::null_mut(),
            node_name.as_ptr() as *const _,
            node_name.len() as _,
            &mut node_name_handle,
        );
        assert_that!(ret_val, eq(IOX2_OK));
        iox2_node_builder_set_name(
            &node_builder_handle,
            iox2_cast_node_name_ptr(node_name_handle),
        );
        iox2_node_name_drop(node_name_handle);

        let mut node_handle: iox2_node_h = core::ptr::null_mut();
        let ret_val = iox2_node_builder_create(
            node_builder_handle,
            core::ptr::null_mut(),
            S::service_type(),
            &mut node_handle as *mut iox2_node_h,
        );

        assert_that!(ret_val, eq(IOX2_OK));

        node_handle
    }
}

fn create_event_service(
    node_handle: iox2_node_h_ref,
    service_name: &str,
) -> iox2_port_factory_event_h {
    unsafe {
        let mut service_name_handle: iox2_service_name_h = core::ptr::null_mut();
        let ret_val = iox2_service_name_new(
            core::ptr::null_mut(),
            service_name.as_ptr() as *const _,
            service_name.len(),
            &mut service_name_handle,
        );
        assert_that!(ret_val, eq(IOX2_OK));

        let service_builder_handle = iox2_node_service_builder(
            node_handle,
            core::ptr::null_mut(),
            iox2_cast_service_name_ptr(service_name_handle),
        );
        iox2_service_name_drop(service_name_handle);

        let service_builder_handle = iox2_service_builder_event(service_builder_handle);
        iox2_service_builder_event_set_max_notifiers(&service_builder_handle, 10);
        iox2_service_builder_event_set_max_listeners(&service_builder_handle, 10);

        let mut event_factory: iox2_port_factory_event_h = core::ptr::null_mut();
        let ret_val = iox2_service_builder_event_open_or_create(
            service_builder_handle,
            core::ptr::null_mut(),
            &mut event_factory as *mut _,
        );
        assert_that!(ret_val, eq(IOX2_OK));

        event_factory
    }
}
