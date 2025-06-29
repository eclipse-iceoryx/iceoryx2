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

use crate::tests::*;

use core::{slice, str};

#[test]
fn basic_node_name_test() -> Result<(), Box<dyn core::error::Error>> {
    unsafe {
        let expected_node_name = NodeName::new("hypnotaod")?;

        let mut node_name_handle: iox2_node_name_h = core::ptr::null_mut();
        let ret_val = iox2_node_name_new(
            core::ptr::null_mut(),
            expected_node_name.as_str().as_ptr() as *const _,
            expected_node_name.len() as _,
            &mut node_name_handle,
        );
        assert_that!(ret_val, eq(IOX2_OK));

        let mut node_name_len = 0;
        let node_name_chars = iox2_node_name_as_chars(
            iox2_cast_node_name_ptr(node_name_handle),
            &mut node_name_len,
        );

        let slice = slice::from_raw_parts(node_name_chars as *const _, node_name_len as _);
        let node_name = str::from_utf8(slice)?;

        assert_that!(node_name, eq(expected_node_name.as_str()));

        iox2_node_name_drop(node_name_handle);

        let _foo = &(*(node_name_handle as *mut _ as *mut iox2_node_name_t)).value;

        Ok(())
    }
}
