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

#![allow(non_camel_case_types)]
#![allow(dead_code)]

use crate::posix::{MemZeroedStruct, CPU_SETSIZE};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct cpu_set_t {
    pub __bits: [u8; CPU_SETSIZE / 8],
}
impl MemZeroedStruct for cpu_set_t {}

impl cpu_set_t {
    pub fn set(&mut self, cpu: usize) {
        if cpu > CPU_SETSIZE {
            return;
        }

        let index = cpu / 8;
        let offset = cpu % 8;

        self.__bits[index] |= 1 << offset;
    }

    pub fn has(&self, cpu: usize) -> bool {
        if cpu > CPU_SETSIZE {
            return false;
        }

        let index = cpu / 8;
        let offset = cpu % 8;
        self.__bits[index] & (1 << offset) != 0
    }

    pub(crate) fn new_allow_all() -> Self {
        Self {
            __bits: [0xff; CPU_SETSIZE / 8],
        }
    }
}
