// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

    use core::ffi::{c_char, c_void};
    use core::mem::{align_of, size_of};

    use iceoryx2::prelude::*;
    use iceoryx2::service::Service;
    use iceoryx2::service::service_hash::ServiceHash;
    use iceoryx2_bb_testing::assert_that;

    type KeyType = u64;

    unsafe extern "C" fn key_cmp(lhs: *const c_void, rhs: *const c_void) -> bool {
        unsafe { *(lhs as *const KeyType) == *(rhs as *const KeyType) }
    }

    #[test]
    fn basic_service_builder_blackboard_test<S: Service + ServiceTypeMapping>() {
        unsafe {
            let node_handle = create_node::<S>("foo");

            let service_name = "all/glory/to/hypnotoad/blackboard";
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

            let service_builder_handle =
                iox2_service_builder_blackboard_creator(service_builder_handle);

            // key type
            let key_type_name = "uint64_t";
            let ret_val = iox2_service_builder_blackboard_creator_set_key_type_details(
                &service_builder_handle,
                key_type_name.as_ptr() as *const c_char,
                key_type_name.len(),
                size_of::<KeyType>(),
                align_of::<KeyType>(),
            );
            assert_that!(ret_val, eq(IOX2_OK));

            iox2_service_builder_blackboard_creator_set_key_eq_comparison_function(
                &service_builder_handle,
                key_cmp,
            );

            let key_0: KeyType = 0;
            let mut value_0: f32 = 0.25;
            let value_0_type_name = "float";
            iox2_service_builder_blackboard_creator_add(
                &service_builder_handle,
                &key_0 as *const KeyType as *const c_void,
                &mut value_0 as *mut f32 as *mut c_void,
                None,
                value_0_type_name.as_ptr() as *const c_char,
                value_0_type_name.len(),
                size_of::<f32>(),
                align_of::<f32>(),
            );

            let key_1: KeyType = 1;
            let mut value_1: u32 = 100;
            let value_1_type_name = "uint32_t";
            iox2_service_builder_blackboard_creator_add(
                &service_builder_handle,
                &key_1 as *const KeyType as *const c_void,
                &mut value_1 as *mut u32 as *mut c_void,
                None,
                value_1_type_name.as_ptr() as *const c_char,
                value_1_type_name.len(),
                size_of::<u32>(),
                align_of::<u32>(),
            );

            let mut blackboard_factory: iox2_port_factory_blackboard_h = core::ptr::null_mut();
            let ret_val = iox2_service_builder_blackboard_create(
                service_builder_handle,
                core::ptr::null_mut(),
                &mut blackboard_factory as *mut _,
            );
            assert_that!(ret_val, eq(IOX2_OK));

            iox2_port_factory_blackboard_drop(blackboard_factory);
            iox2_node_drop(node_handle);
        }
    }

    #[test]
    fn basic_port_factory_blackboard_service_hash_test<S: Service + ServiceTypeMapping>() {
        const BUFFER_LENGTH: usize = 20;
        const SERVICE_HASH_LENGTH: usize = BUFFER_LENGTH - 7;

        unsafe {
            let node_handle = create_node::<S>("foo");

            let service_name = "all/glory/to/hypnotoad/blackboard";
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

            let service_builder_handle =
                iox2_service_builder_blackboard_creator(service_builder_handle);

            // key type
            let key_type_name = "uint64_t";
            let ret_val = iox2_service_builder_blackboard_creator_set_key_type_details(
                &service_builder_handle,
                key_type_name.as_ptr() as *const c_char,
                key_type_name.len(),
                size_of::<KeyType>(),
                align_of::<KeyType>(),
            );
            assert_that!(ret_val, eq(IOX2_OK));

            iox2_service_builder_blackboard_creator_set_key_eq_comparison_function(
                &service_builder_handle,
                key_cmp,
            );

            let key_0: KeyType = 0;
            let mut value_0: f32 = 0.25;
            let value_0_type_name = "float";
            iox2_service_builder_blackboard_creator_add(
                &service_builder_handle,
                &key_0 as *const KeyType as *const c_void,
                &mut value_0 as *mut f32 as *mut c_void,
                None,
                value_0_type_name.as_ptr() as *const c_char,
                value_0_type_name.len(),
                size_of::<f32>(),
                align_of::<f32>(),
            );

            let key_1: KeyType = 1;
            let mut value_1: u32 = 100;
            let value_1_type_name = "uint32_t";
            iox2_service_builder_blackboard_creator_add(
                &service_builder_handle,
                &key_1 as *const KeyType as *const c_void,
                &mut value_1 as *mut u32 as *mut c_void,
                None,
                value_1_type_name.as_ptr() as *const c_char,
                value_1_type_name.len(),
                size_of::<u32>(),
                align_of::<u32>(),
            );

            let mut blackboard_factory: iox2_port_factory_blackboard_h = core::ptr::null_mut();
            iox2_service_builder_blackboard_create(
                service_builder_handle,
                core::ptr::null_mut(),
                &mut blackboard_factory as *mut _,
            );

            let result_hash = &mut ['=' as c_char; BUFFER_LENGTH]; // equal sign does not occur in service hash
            let expected_hash = ServiceHash::new::<<S>::ServiceNameHasher>(
                &ServiceName::new(service_name).expect("valid service name"),
                MessagingPattern::Blackboard,
            );
            let mut expected_hash_vec = expected_hash.as_str().as_bytes()
                [..SERVICE_HASH_LENGTH - 1]
                .iter()
                .map(|&b| b as c_char)
                .collect::<Vec<_>>();
            expected_hash_vec.push(0);
            let expected_hash_slice: &[c_char] = expected_hash_vec.as_slice();

            let handle: iox2_port_factory_blackboard_h_ref = &blackboard_factory;
            iox2_port_factory_blackboard_service_hash(
                handle,
                result_hash.as_mut_slice().as_mut_ptr(),
                SERVICE_HASH_LENGTH,
            );

            assert_that!(&result_hash[..SERVICE_HASH_LENGTH], eq(expected_hash_slice));
            assert_that!(&result_hash[SERVICE_HASH_LENGTH..], eq(['=' as c_char; 7]));

            iox2_port_factory_blackboard_drop(blackboard_factory);
            iox2_node_drop(node_handle);
        }
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
