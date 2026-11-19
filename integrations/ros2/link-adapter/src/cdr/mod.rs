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

//! Reading and writing ROS 2 messages in their XCDR1 encoding.
//!
//! # Encoding
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
//! A message with these members and values
//!
//! ```text
//! uint8    a   1
//! uint64   b   2
//! string   s   "hi"
//! uint16[] q   [3]
//! ```
//!
//! is encoded little-endian as the bytes below. `b` starts at offset 12
//! rather than 8 since alignment counts from byte 4.
//!
//! ```text
//! offset   0           4  5                    12                      20          24       27 28          32
//!          ┌───────────┬──┬────────────────────┬───────────────────────┬───────────┬────────┬──┬───────────┬─────┐
//!          │00 01 00 00│01│00 00 00 00 00 00 00│02 00 00 00 00 00 00 00│03 00 00 00│68 69 00│00│01 00 00 00│03 00│
//!          └───────────┴──┴────────────────────┴───────────────────────┴───────────┴────────┴──┴───────────┴─────┘
//!           header      a  padding to 8         b                       s length    "hi" NUL pad q length   q[0]
//! ```
//!
//! # Buffers
//!
//! The writer fills a [`ForwardRegion`] over the caller's buffer. The
//! length of a message is not known until it is written, so the region
//! grows by each value and ends after all elements are written.
//!
//! ```text
//! caller's buffer, e.g. a `Vec<u8>`
//! ┌────────┬──────────────────────┐
//! │ header │ values and padding   │  resize(n) on each value
//! └────────┴──────────────────────┘
//!                                 ^ cursor
//! ```
//!
//! The reader borrows the bytes the rmw delivers. The rmw sizes them by
//! its estimate of the message, so the message may end before the bytes
//! do.
//!
//! ```text
//! rmw serialized message
//! ┌────────┬──────────────────────┬──────────┐
//! │ header │ read                 │ unread   │
//! └────────┴──────────────────────┴──────────┘
//!                                 ^ cursor
//!                                            ^ end
//! ```
//!
//! # Cursors
//!
//! A primitive is written in three steps.
//!
//! 1. Pad with zeros up to the primitive's alignment.
//! 2. Grow the region by the primitive's size.
//! 3. Copy the primitive's bytes in the host's byte order.
//!
//! A primitive is read in the same three steps.
//!
//! 1. Skip the padding up to the primitive's alignment.
//! 2. Check that the primitive's bytes remain.
//! 3. Decode the bytes in the header's byte order.
//!
//! An array or sequence takes its elements as one block aligned to a
//! single element.
//!
//! [`ForwardRegion`]: iceoryx2_link_backend::wire::region::ForwardRegion

pub mod reader;
pub mod writer;

pub use reader::Reader;
pub use writer::Writer;

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
