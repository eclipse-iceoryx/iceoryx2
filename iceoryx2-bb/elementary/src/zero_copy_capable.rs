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

use crate::{identifiable::Identifiable, relocatable::Relocatable};

/// Marks types that can be used for shared-memory zero-copy communication.
///
/// # Safety
///
/// The user must ensure that
///  * the types are self-contained, no pointers, references or handles to resources that are not
///    part of the type
///  * the types are relocatable, no pointers, references or handles to manage internal structures
///  * the types have the same type layout independent of the compilation unit, e.g. are
///    annotated with `#[repr(C)]`
///
pub unsafe trait ZeroCopyCapable {}

unsafe impl<T: Relocatable + Identifiable> ZeroCopyCapable for T {}
