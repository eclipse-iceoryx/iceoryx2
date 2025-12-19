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

core::arch::global_asm!(
    r#"

.section .text.startup
.global _start
.global _vectors
.code 32
.align 0
// Work around https://github.com/rust-lang/rust/issues/127269
.fpu vfp3-d16

_vectors:
    LDR     pc, STARTUP                     @ Reset vector - loads PC with address of _start

STARTUP:
    .word  _start                           @ Address of startup function

_start:
    // Set stack pointer
    ldr sp, =_stack_top

    // Allow VFP coprocessor access
    mrc p15, 0, r0, c1, c0, 2
    orr r0, r0, #0xF00000
    mcr p15, 0, r0, c1, c0, 2

    // Enable VFP
    mov r0, #0x40000000
    vmsr fpexc, r0

    // Jump to application
    bl kmain

    // In case the application returns, loop forever
    b .
"#
);
