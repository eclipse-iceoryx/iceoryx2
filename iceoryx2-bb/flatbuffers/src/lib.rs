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
/// Schema file finder.
pub mod schema_finder;
/// Deduce a flatbuffer type name and namespace from a rust type name.
pub mod type_name;

pub use schema_finder::*;
pub use type_name::*;
