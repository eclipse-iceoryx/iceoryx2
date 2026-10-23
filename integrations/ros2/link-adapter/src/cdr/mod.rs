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
//! definition order.
//!
//! * A primitive is aligned to its size, counted from the byte after the
//!   header.
//! * An array is its elements.
//! * A sequence is a `u32` length followed by its elements. An empty
//!   sequence is its length alone.
//! * A string is a `u32` length followed by its bytes and a NUL.
//!
//! The rmw reports its size estimate as the length, so a message may be
//! followed by bytes that are not part of it.

mod reader;
mod writer;

pub use reader::{Endianness, ReadError, Reader};
pub use writer::{WriteError, Writer};

use core::ops::Range;

/// The bytes of the encapsulation header holding the representation
/// identifier.
const IDENTIFIER_BYTES: Range<usize> = 0..2;
/// The bytes of the encapsulation header holding the representation
/// options, reserved and always zero.
const OPTIONS_BYTES: Range<usize> = 2..4;
/// The length of the encapsulation header.
const ENCAPSULATION_LEN: usize = OPTIONS_BYTES.end;

/// The representation identifier of big-endian plain CDR.
const CDR_BE: [u8; 2] = [0x00, 0x00];
/// The representation identifier of little-endian plain CDR.
const CDR_LE: [u8; 2] = [0x00, 0x01];
/// The representation identifier of plain CDR in the host's byte order,
/// the one the writer stamps.
#[cfg(target_endian = "little")]
const CDR_NE: [u8; 2] = CDR_LE;
#[cfg(target_endian = "big")]
const CDR_NE: [u8; 2] = CDR_BE;
