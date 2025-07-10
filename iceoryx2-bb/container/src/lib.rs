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

#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]
#![warn(missing_docs)]

//! # iceoryx2 Building Blocks (BB) Container
//!
//! This is a support library for iceoryx2 which comes with containers that are
//! compatible with shared memory and can be used to construct custom payload types for
//! inter-process communication.
//!
//! Most containers come in 3 variations:
//!  1. `FixedSize*Container*`, compile-time fixed size version. The capacity must be known at compile
//!     time. Those fixed-size constructs are always self-contained, meaning that the required
//!     memory is part of the constructs and usually stored in some kind of array.
//!  2. `Relocatable*Container*`, run-time fixed size version that is shared memory compatible. The
//!     capacity must be known when the object is created. **This object is not movable!**
//!  3. `*Container*`, run-time fixed size version that is **not** shared memory compatible but can be
//!     moved. The memory is by default stored on the heap.
//!
//! # Example
//!
//! ## 1. Compile-Time FixedSize Containers
//!
//! We create a struct consisting of compile-time fixed size containers that can be used for
//! zero copy inter-process communication.
//!
//! ```
//! use iceoryx2_bb_container::byte_string::*;
//! use iceoryx2_bb_container::vec::*;
//!
//! const TEXT_CAPACITY: usize = 123;
//! const DATA_CAPACITY: usize = 456;
//!
//! #[repr(C)]
//! struct MyMessageType {
//!     some_text: FixedSizeByteString<TEXT_CAPACITY>,
//!     some_data: FixedSizeVec<u64, DATA_CAPACITY>,
//! }
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let my_message = MyMessageType {
//!     some_text: FixedSizeByteString::from_bytes(b"Hello World")?,
//!     some_data: FixedSizeVec::new(),
//! };
//! # Ok(())
//! # }
//! ```
//!
//! ## 2. Shared Memory Compatible Run-Time FixedSize Containers
//!
//! Despite that the containers are already implemented, iceoryx2 itself does not yet support
//! run-time fixed size types. It is planned and will be part of an upcoming release.
//!
//! ## 3. Run-Time FixedSize Containers
//!
//! We create a struct consisting of run-time fixed size containers. This can be interesting when
//! it shall be used in a safety-critical environment where everything must be pre-allocated to
//! ensure that required memory is always available.
//!
//! ```
//! use iceoryx2_bb_container::queue::*;
//!
//! const QUEUE_CAPACITY: usize = 123;
//!
//! #[repr(C)]
//! struct MyType {
//!     some_queue: Queue<u64>,
//! }
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let my_thing = MyType {
//!     some_queue: Queue::new(QUEUE_CAPACITY),
//! };
//! # Ok(())
//! # }
//! ```

extern crate alloc;

/// A byte string similar to [`std::string::String`] but it does not support UTF-8
pub mod byte_string;
/// A queue similar to [`std::collections::VecDeque`]
pub mod queue;
/// A container with persistent unique keys to access values.
pub mod slotmap;
/// Extends the [ByteString](crate::byte_string) so that custom string types with a semantic
/// ruleset on their content can be realized.
#[macro_use]
pub mod semantic_string;
/// A container to store key-value pairs.
pub mod flatmap;
/// A vector similar to [`std::vec::Vec`]
pub mod vec;
