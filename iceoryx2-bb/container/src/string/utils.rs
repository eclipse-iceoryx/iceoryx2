// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use alloc::vec;
use alloc::vec::Vec;

/// Returns the length of a c string
///
/// # Safety
///
///  * The string must be '\0' (null) terminated.
///
pub unsafe fn strnlen(ptr: *const core::ffi::c_char, len: usize) -> usize {
    const NULL_TERMINATION: core::ffi::c_char = 0;
    for i in 0..len {
        if *ptr.add(i) == NULL_TERMINATION {
            return i;
        }
    }

    len
}

/// Adds escape characters to the string so that it can be used for console output.
pub fn as_escaped_string(bytes: &[u8]) -> alloc::string::String {
    unsafe {
        alloc::string::String::from_utf8_unchecked(
            bytes
                .iter()
                .flat_map(|c| match *c {
                    b'\t' => vec![b'\\', b't'].into_iter(),
                    b'\r' => vec![b'\\', b'r'].into_iter(),
                    b'\n' => vec![b'\\', b'n'].into_iter(),
                    b'\x20'..=b'\x7f' => vec![*c].into_iter(),
                    _ => {
                        let hex_digits: &[u8; 16] = b"0123456789abcdef";
                        vec![
                            b'\\',
                            b'x',
                            hex_digits[(c >> 4) as usize],
                            hex_digits[(c & 0xf) as usize],
                        ]
                        .into_iter()
                    }
                })
                .collect::<Vec<u8>>(),
        )
    }
}
