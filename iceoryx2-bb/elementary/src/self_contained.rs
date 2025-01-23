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

use crate::composable_marker_trait;
use iceoryx2_pal_concurrency_sync::iox_atomic::*;

composable_marker_trait! {
/// Marks types that are self-contained, meaning they do not contain pointers or references, even
/// when the pointers/references are pointing into internal structures.
/// The types are also not allowed to manage resources like file descriptors or own handles that
/// represent some external resource.
SelfContained =>
  u8, u16, u32, u64, u128, usize,
  i8, i16, i32, i64, i128, isize,
  f32, f64,
  char, bool,
  IoxAtomicU8, IoxAtomicU16, IoxAtomicU32, IoxAtomicU64, IoxAtomicUsize,
  IoxAtomicI8, IoxAtomicI16, IoxAtomicI32, IoxAtomicI64, IoxAtomicIsize
}
