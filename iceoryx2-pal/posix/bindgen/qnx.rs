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

extern crate bindgen;

use std::env;

pub fn configure_cargo() {}

pub fn configure_builder(builder: bindgen::Builder) -> bindgen::Builder {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();

    let mut builder = builder;

    // NOTE: needs to live as long as compiler_args
    let target_triple_flag: String;

    // Common compiler defines for QNX
    let mut compiler_args = vec![
        "-D__QNXNTO__",
        "-D__NO_INLINE__",
        "-D__DEPRECATED",
        "-D__unix__",
        "-D__unix",
        "-D__ELF__",
        "-D__LITTLEENDIAN__",
    ];

    // Version-specific compiler defines for QNX
    match target_env.as_str() {
        "nto71" => {
            compiler_args.push("-D__QNX__");
            compiler_args.push("-D__GNUC__=8");
            compiler_args.push("-D__GNUC_MINOR__=3");
            compiler_args.push("-D__GNUC_PATCHLEVEL__=0");
        }
        "nto80" => {
            let target_triple = std::env::var("TARGET").unwrap().replace("qnx800", "800");
            target_triple_flag = format!("--target={}", target_triple);
            compiler_args.push(&target_triple_flag);
            compiler_args.push("-D__QNX__=800");
            compiler_args.push("-D__GNUC__=12");
            compiler_args.push("-D__GNUC_MINOR__=2");
            compiler_args.push("-D__GNUC_PATCHLEVEL__=0");
        }
        _ => {
            panic!(
                    "Unsupported QNX target environment: {target_env}. Only nto71 and nto80 are supported.",
                );
        }
    }

    // Architecture-specific compiler defines for QNX
    if target_arch == "x86_64" {
        compiler_args.push("-D__X86_64__");
    }

    for arg in &compiler_args {
        builder = builder.clang_arg(*arg);
    }

    if let Ok(sysroot) = env::var("QNX_TARGET") {
        builder = builder.clang_arg(format!("--sysroot={sysroot}"));
        builder = builder.clang_arg(format!("-I{sysroot}/usr/include"));
        builder = builder.clang_arg(format!("-I{sysroot}/usr/include/c++/v1"));
    } else {
        panic!("QNX_TARGET environment variable not set for QNX build")
    }

    builder
}
