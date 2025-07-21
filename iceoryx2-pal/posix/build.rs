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

#[cfg(feature = "libc_platform")]
fn main() {}

#[cfg(not(feature = "libc_platform"))]
fn main() {
    extern crate bindgen;
    extern crate cc;

    use bindgen::*;
    use std::env;
    use std::path::PathBuf;

    // #[cfg(any(...))] does not work when cross-compiling
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "linux" || target_os == "freebsd" {
        println!("cargo:rustc-link-lib=pthread");
    }

    println!("cargo:rerun-if-changed=src/c/posix.h");

    let mut builder = bindgen::Builder::default()
        .header("src/c/posix.h")
        .blocklist_type("max_align_t")
        .parse_callbacks(Box::new(CargoCallbacks::new()))
        .use_core();

    if std::env::var("DOCS_RS").is_ok() {
        builder = builder.clang_arg("-D IOX2_DOCS_RS_SUPPORT");
    }

    if target_os == "nto" {
        let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
        let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();

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
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    // needed for bazel but can be empty for cargo builds
    println!("cargo:rustc-env=BAZEL_BINDGEN_PATH_CORRECTION=");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("posix_generated.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=src/c/socket_macros.c");
    cc::Build::new()
        .file("src/c/socket_macros.c")
        .compile("libsocket_macros.a");
}
