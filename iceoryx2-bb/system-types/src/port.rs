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

use core::fmt::Display;

use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

#[derive(Clone, Copy, PartialEq, Eq, Debug, ZeroCopySend)]
#[repr(C)]
pub struct Port(u16);

impl Display for Port {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub const UNSPECIFIED: Port = Port::new(0);

impl Port {
    pub const fn new(port: u16) -> Self {
        Self(port)
    }

    pub const fn as_u16(&self) -> u16 {
        self.0
    }

    pub const fn is_unspecified(&self) -> bool {
        self.0 == UNSPECIFIED.0
    }

    pub const fn is_system(&self) -> bool {
        self.0 != 0 && self.0 <= 1023
    }

    pub const fn is_registered(&self) -> bool {
        1024 <= self.0 && self.0 <= 49151
    }

    pub const fn is_dynamic(&self) -> bool {
        49152 <= self.0
    }
}
