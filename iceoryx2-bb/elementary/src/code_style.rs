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

extern crate alloc;

use alloc::string::String;

/// Converts a string from snake_case to UpperCamelCase.
pub fn snake_to_upper_camel_case(input: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    for (pos, c) in input.chars().enumerate() {
        if pos == 0 && c.is_lowercase() {
            for upper_c in c.to_uppercase() {
                result.push(upper_c);
            }
        } else if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            for upper_c in c.to_uppercase() {
                result.push(upper_c);
            }
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// Converts a string from CamelCase to snake_case
pub fn camel_to_snake_case(input: &str) -> String {
    let mut result = String::new();
    let mut prev_char: Option<char> = None;

    for c in input.chars() {
        if c.is_uppercase() {
            if let Some(prev) = prev_char {
                if prev.is_lowercase() || prev.is_numeric() {
                    result.push('_');
                }
            }
            for lower_c in c.to_lowercase() {
                result.push(lower_c);
            }
        } else {
            result.push(c);
        }
        prev_char = Some(c);
    }
    result
}
