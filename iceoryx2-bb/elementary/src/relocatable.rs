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

use crate::{fixed_size::FixedSize, self_contained::SelfContained};

/// Marks types that are relocatable into another process space. They are allowed to own resources
/// that are located outside of the type but when mapped into a different process space the
/// resource must be still accessible. An example is a list where all elements are in shared memory
/// and the elements are pointed to via
/// [`RelocatablePointer`](crate::relocatable_ptr::RelocatablePointer).
pub trait Relocatable: FixedSize {}

impl<T: SelfContained + FixedSize> Relocatable for T {}
