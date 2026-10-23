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

//! The XCDR1 encoding of ROS 2 messages.
//!
//! A message is the 4-byte encapsulation header followed by its members in
//! definition order. Primitives are aligned to their size relative to the
//! byte after the header. A sequence is a `u32` length followed by its
//! elements, a string a `u32` length followed by its bytes and a NUL.

mod writer;

pub use writer::{WriteError, Writer};

/// The length of the encapsulation header.
pub(crate) const ENCAPSULATION_LEN: usize = 4;
