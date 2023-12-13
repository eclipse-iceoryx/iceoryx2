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

#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

unsafe fn swap_endianess_32(v: u32) -> u32 {
    (v & 0xff000000) >> 24 | (v & 0x00ff0000) >> 8 | (v & 0x0000ff00) << 8 | (v & 0x000000ff) << 24
}

unsafe fn swap_endianess_16(v: u16) -> u16 {
    v >> 8 | v << 8
}

pub unsafe fn htonl(hostlong: u32) -> u32 {
    swap_endianess_32(hostlong)
}

pub unsafe fn htons(hostshort: u16) -> u16 {
    swap_endianess_16(hostshort)
}

pub unsafe fn ntohl(netlong: u32) -> u32 {
    swap_endianess_32(netlong)
}

pub unsafe fn ntohs(netshort: u16) -> u16 {
    swap_endianess_16(netshort)
}
