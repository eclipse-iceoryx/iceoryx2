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

pub mod cpu_set_t;
pub(crate) mod error_enum_generator;
pub mod mem_zeroed_struct;
pub mod sockaddr_in;
pub(crate) mod string_operations;

#[cfg(not(target_os = "windows"))]
pub(crate) mod scandir;
