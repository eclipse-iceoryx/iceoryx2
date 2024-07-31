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
mod node {
    use crate::{iox2_cast_node_ref_h, tests::*};

    use core::{slice, str};

    #[test]
    fn basic_node_api_test<S: Service + ServiceTypeMapping>() {
        unsafe {
            let node_handle = create_node::<S>("");

            assert_that!(node_handle, ne(std::ptr::null_mut()));

            iox2_node_drop(node_handle);
        }
    }

    #[test]
    fn basic_node_config_test<S: Service + ServiceTypeMapping>() {
        unsafe {
            let node_handle = create_node::<S>("");

            let expected_config = Config::global_config();

            let config = iox2_node_config(iox2_cast_node_ref_h(node_handle));

            assert_that!(*(config as *const Config), eq(*expected_config));

            iox2_node_drop(node_handle);
        }
    }

    #[test]
    fn basic_node_name_test<S: Service + ServiceTypeMapping>(
    ) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let node_handle = create_node::<S>("hypnotoad");

            let node_name = iox2_node_name(iox2_cast_node_ref_h(node_handle));

            let mut node_name_len = 0;
            let node_name_c_str = iox2_node_name_as_c_str(node_name, &mut node_name_len);

            let slice = slice::from_raw_parts(node_name_c_str as *const _, node_name_len as _);
            let node_name_str = str::from_utf8(slice)?;

            assert_that!(node_name_str, eq("hypnotoad"));

            iox2_node_drop(node_handle);

            Ok(())
        }
    }

    #[derive(Default)]
    struct NodeListCtx {
        alive: u64,
        dead: u64,
        inaccessible: u64,
        undefined: u64,
    }

    extern "C" fn node_list_callback(
        node_state: iox2_node_state_e,
        _node_id_ptr: iox2_node_id_ptr,
        _node_name_ptr: iox2_node_name_ptr,
        _config_ptr: iox2_config_ptr,
        ctx: iox2_callback_context,
    ) -> iox2_callback_progression_e {
        let ctx = unsafe { &mut *(ctx as *mut NodeListCtx) };

        match node_state {
            iox2_node_state_e::ALIVE => {
                ctx.alive += 1;
            }
            iox2_node_state_e::DEAD => {
                ctx.dead += 1;
            }
            iox2_node_state_e::INACCESSIBLE => {
                ctx.inaccessible += 1;
            }
            iox2_node_state_e::UNDEFINED => {
                ctx.undefined += 1;
            }
        }

        iox2_callback_progression_e::CONTINUE
    }

    #[test]
    fn basic_node_list_test<S: Service + ServiceTypeMapping>() {
        unsafe {
            let mut ctx = NodeListCtx::default();
            let node_handle = create_node::<S>("");
            let config = iox2_node_config(iox2_cast_node_ref_h(node_handle));

            let ret_val = iox2_node_list(
                S::service_type(),
                config,
                node_list_callback,
                &mut ctx as *mut _ as *mut _,
            );

            iox2_node_drop(node_handle);

            assert_that!(ret_val, eq(IOX2_OK));

            assert_that!(ctx.alive, eq(1));
            assert_that!(ctx.dead, eq(0));
            assert_that!(ctx.inaccessible, eq(0));
            assert_that!(ctx.undefined, eq(0));
        }
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
