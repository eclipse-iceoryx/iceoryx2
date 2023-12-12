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

pub unsafe fn htonl(hostlong: u32) -> u32 {
    windows_sys::Win32::Networking::WinSock::htonl(hostlong)
}

pub unsafe fn htons(hostshort: u16) -> u16 {
    windows_sys::Win32::Networking::WinSock::htons(hostshort)
}

pub unsafe fn ntohl(netlong: u32) -> u32 {
    windows_sys::Win32::Networking::WinSock::ntohl(netlong)
}

pub unsafe fn ntohs(netshort: u16) -> u16 {
    windows_sys::Win32::Networking::WinSock::ntohs(netshort)
}
