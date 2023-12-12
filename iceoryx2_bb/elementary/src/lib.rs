// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

//! Contains basic constructs which do not have any kind of dependency.

#[macro_use]
pub mod enum_gen;
pub mod allocator;
pub mod lazy_singleton;
pub mod math;
pub mod owning_pointer;
pub mod pointer_trait;
pub mod relocatable_container;
pub mod relocatable_ptr;
pub mod scope_guard;
pub mod unique_id;
