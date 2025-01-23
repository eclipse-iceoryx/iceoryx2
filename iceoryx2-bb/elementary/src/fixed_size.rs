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

use iceoryx2_pal_concurrency_sync::iox_atomic::*;

use crate::composable_marker_trait;

composable_marker_trait! {
/// Marks types that have the same size on every architecture. For instance [`usize`] can have a
/// size of 32-bit or 64-bit depending on the architecture and is therefore not marked. A pointer
/// is another example of a type where the size depends on the architecture.
FixedSize =>
  u8, u16, u32, u64, u128,
  i8, i16, i32, i64, i128,
  f32, f64,
  char, bool,
  IoxAtomicU8, IoxAtomicU16, IoxAtomicU32, IoxAtomicU64,
  IoxAtomicI8, IoxAtomicI16, IoxAtomicI32, IoxAtomicI64
}
