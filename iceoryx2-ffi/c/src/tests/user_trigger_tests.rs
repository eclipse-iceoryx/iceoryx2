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
mod user_trigger {

    use crate::api::*;
    use crate::tests::{ServiceTypeMapping, create_node};
    use iceoryx2::prelude::*;
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn basic_user_trigger_test<S: Service + ServiceTypeMapping>() {
        unsafe {
            let node_handle = create_node::<S>("bar");

            let mut user_trigger_handle = core::ptr::null_mut();
            let ret_val = iox2_node_create_user_trigger(
                &node_handle,
                core::ptr::null_mut(),
                &mut user_trigger_handle,
            );
            assert_that!(ret_val, eq(IOX2_OK));

            assert_that!(
                iox2_user_trigger_get_file_descriptor(&user_trigger_handle).is_null(),
                eq(false)
            );

            let mut was_notified = true;
            let ret_val = iox2_user_trigger_reset(&user_trigger_handle, &mut was_notified);
            assert_that!(ret_val, eq(IOX2_OK));
            assert_that!(was_notified, eq(false));

            assert_that!(iox2_user_trigger_notify(&user_trigger_handle), eq(IOX2_OK));
            let ret_val = iox2_user_trigger_reset(&user_trigger_handle, &mut was_notified);
            assert_that!(ret_val, eq(IOX2_OK));
            assert_that!(was_notified, eq(true));

            iox2_user_trigger_drop(user_trigger_handle);
            iox2_node_drop(node_handle);
        }
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
