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

use crate::relocatable::Relocatable;

/// Marks types that can be safely stored in shared memory and consumed from multiple process
/// using different address spaces.
pub trait SharedMemorySafe {
    fn type_name() -> &'static str {
        core::any::type_name::<Self>()
    }
}

impl<T: Relocatable> SharedMemorySafe for T {}
