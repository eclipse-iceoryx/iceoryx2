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
    use crate::api::*;
    use crate::tests::{ServiceTypeMapping, create_node};
    use core::ffi::{c_char, c_int};
    use iceoryx2::prelude::*;
    use iceoryx2::service::Service;
    use iceoryx2::service::builder::event::EventOpenOrCreateError;
    use iceoryx2::service::service_hash::ServiceHash;
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn basic_service_builder_event_test<S: Service + ServiceTypeMapping>() {
        unsafe {
            let node_handle = create_node::<S>("foo");

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

            iox2_port_factory_event_drop(event_factory);
            iox2_node_drop(node_handle);
        }
    }

    #[test]
    fn basic_port_factory_event_service_hash_test<S: Service + ServiceTypeMapping>() {
        const BUFFER_LENGTH: usize = 20;
        const SERVICE_HASH_LENGTH: usize = BUFFER_LENGTH - 7;

        unsafe {
            let node_handle = create_node::<S>("bar");

            let service_name = "all/glory/to/hypnotaod";

            let mut service_name_handle: iox2_service_name_h = core::ptr::null_mut();
            iox2_service_name_new(
                core::ptr::null_mut(),
                service_name.as_ptr() as *const _,
                service_name.len(),
                &mut service_name_handle,
            );

            let service_builder_handle = iox2_node_service_builder(
                &node_handle,
                core::ptr::null_mut(),
                iox2_cast_service_name_ptr(service_name_handle),
            );
            iox2_service_name_drop(service_name_handle);

            let service_builder_handle = iox2_service_builder_event(service_builder_handle);

            let mut event_factory: iox2_port_factory_event_h = core::ptr::null_mut();
            iox2_service_builder_event_open_or_create(
                service_builder_handle,
                core::ptr::null_mut(),
                &mut event_factory as *mut _,
            );

            let result_hash = &mut [b'=' as c_char; BUFFER_LENGTH]; // equal sign does not occur in service hash
            let expected_hash = ServiceHash::new::<<S>::ServiceNameHasher>(
                &ServiceName::new(service_name).expect("valid service name"),
                MessagingPattern::Event,
            );
            let mut expected_hash_vec = expected_hash.as_str().as_bytes()
                [..SERVICE_HASH_LENGTH - 1]
                .iter()
                .map(|&b| b as c_char)
                .collect::<Vec<_>>();
            expected_hash_vec.push(0);
            let expected_hash_slice: &[c_char] = expected_hash_vec.as_slice();

            let handle: iox2_port_factory_event_h_ref = &event_factory;
            iox2_port_factory_event_service_hash(
                handle,
                result_hash.as_mut_slice().as_mut_ptr(),
                SERVICE_HASH_LENGTH,
            );

            assert_that!(&result_hash[..SERVICE_HASH_LENGTH], eq(expected_hash_slice));
            assert_that!(&result_hash[SERVICE_HASH_LENGTH..], eq([b'=' as c_char; 7]));

            iox2_port_factory_event_drop(event_factory);
            iox2_node_drop(node_handle);
        }
    }

    #[test]
    fn open_or_create_error_system_in_flux_converts_to_c_int<S: Service + ServiceTypeMapping>() {
        assert_that!(
            EventOpenOrCreateError::SystemInFlux.into_c_int(),
            eq(iox2_event_open_or_create_error_e::SYSTEM_IN_FLUX as c_int)
        );
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
