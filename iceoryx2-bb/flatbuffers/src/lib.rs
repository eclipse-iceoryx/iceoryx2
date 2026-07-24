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

//! An iceoryx2 support library that helps to find schema files, to deduce type names and more

#![no_std]
/// Workaround to implement [`core::error::Error`] for flatbuffer error enums that do not implement them
/// in a `no_std` environment
pub mod flatbuffer_error;
/// Resizable memory that can be combined with allocators to provide a allocator backend for flatbuffers.
pub mod resizable_memory;
/// Schema file finder.
pub mod schema_finder;
/// Deduce a flatbuffer type name and namespace from a rust type name.
pub mod type_name;

pub use flatbuffer_error::*;
pub use resizable_memory::*;
pub use schema_finder::*;
pub use type_name::*;
