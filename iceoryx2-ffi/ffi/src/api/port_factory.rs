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

#![allow(non_camel_case_types)]

// BEGIN types definition

#[repr(C)]
pub struct iox2_port_factory_t;

#[repr(C)]
pub struct iox2_port_factory_h_t;
/// The owning handle for `iox2_port_factory_t`. Passing the handle to an function transfers the ownership.
pub type iox2_port_factory_h = *mut iox2_port_factory_h_t;

pub struct iox2_port_factory_ref_h_t;
/// The non-owning handle for `iox2_port_factory_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_port_factory_ref_h = *mut iox2_port_factory_ref_h_t;

// END type definition

// BEGIN C API

// END C API
