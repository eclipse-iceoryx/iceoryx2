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
fn basic_service_name_test() -> Result<(), Box<dyn core::error::Error>> {
    unsafe {
        let expected_service_name = ServiceName::new("all/glory/to/hypnotaod")?;

        let mut service_name_handle: iox2_service_name_h = core::ptr::null_mut();
        let ret_val = iox2_service_name_new(
            core::ptr::null_mut(),
            expected_service_name.as_str().as_ptr() as *const _,
            expected_service_name.len() as _,
            &mut service_name_handle,
        );
        assert_that!(ret_val, eq(IOX2_OK));

        let mut service_name_len = 0;
        let service_name_chars = iox2_service_name_as_chars(
            iox2_cast_service_name_ptr(service_name_handle),
            &mut service_name_len,
        );

        let slice = slice::from_raw_parts(service_name_chars as *const _, service_name_len as _);
        let service_name = str::from_utf8(slice)?;

        assert_that!(service_name, eq(expected_service_name.as_str()));

        iox2_service_name_drop(service_name_handle);

        let _foo = &(*(service_name_handle as *mut _ as *mut iox2_service_name_t)).value;

        Ok(())
    }
}
