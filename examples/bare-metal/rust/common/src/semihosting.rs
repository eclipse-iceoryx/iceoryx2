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

const SYS_WRITE0: usize = 0x04;
const SYS_EXIT: usize = 0x18;

#[inline(always)]
pub unsafe fn syscall(operation: usize, arg: usize) -> usize {
    let result: usize;
    core::arch::asm!(
        "svc 0x123456",
        inout("r0") operation => result,
        in("r1") arg,
        options(nostack)
    );

    result
}

pub fn write0(s: &str) {
    let mut buffer = [0u8; 256];
    let bytes = s.as_bytes();
    let len = bytes.len().min(255);
    buffer[..len].copy_from_slice(&bytes[..len]);
    buffer[len] = 0; // null terminator

    unsafe {
        syscall(SYS_WRITE0, buffer.as_ptr() as usize);
    }
}

pub fn exit(code: usize) -> ! {
    unsafe {
        syscall(SYS_EXIT, code);
    }
    loop {}
}
