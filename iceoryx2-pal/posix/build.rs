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

    println!("cargo:rerun-if-changed=src/c/posix.h");
    println!("cargo:rerun-if-changed=src/c/socket_macros.c");

    // needed for bazel but can be empty for cargo builds
    println!("cargo:rustc-env=BAZEL_BINDGEN_PATH_CORRECTION=");

    // #[cfg(any(...))] does not work when cross-compiling
    // when cross compiling, 'target_os' is set to the environment the build script
    // is executed; to get the actual target OS, use the cargo 'TARGET' env variable
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os.as_str() == "none" {
        return;
    }

    println!("Building for target: {}", target_os);

    configure_cargo(target_os.as_str());

    let mut builder = bindgen::Builder::default()
        .header("src/c/posix.h")
        .blocklist_type("max_align_t")
        .parse_callbacks(Box::new(CargoCallbacks::new()))
        .use_core();

    builder = configure_builder(target_os.as_str(), builder);

    if std::env::var("DOCS_RS").is_ok() {
        builder = builder.clang_arg("-D IOX2_DOCS_RS_SUPPORT");
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("posix_generated.rs"))
        .expect("Couldn't write bindings!");

    cc::Build::new()
        .file("src/c/socket_macros.c")
        .compile("libsocket_macros.a");
}

#[cfg(not(feature = "libc_platform"))]
fn configure_cargo(target_os: &str) {
    match target_os {
        "freebsd" => {
            freebsd::configure_cargo();
        }
        "linux" => {
            linux::configure_cargo();
        }
        "macos" => {
            macos::configure_cargo();
        }
        "nto" => {
            qnx::configure_cargo();
        }
        "windows" => {
            windows::configure_cargo();
        }
        "android" => {
            android::configure_cargo();
        }
        _ => panic!("Unsupported target OS: {}", target_os),
    }
}

#[cfg(not(feature = "libc_platform"))]
fn configure_builder(target_os: &str, builder: bindgen::Builder) -> bindgen::Builder {
    match target_os {
        "freebsd" => freebsd::configure_builder(builder),
        "linux" => linux::configure_builder(builder),
        "macos" => macos::configure_builder(builder),
        "nto" => qnx::configure_builder(builder),
        "windows" => windows::configure_builder(builder),
        "android" => android::configure_builder(builder),
        _ => panic!("Unsupported target OS: {}", target_os),
    }
}

#[cfg(not(feature = "libc_platform"))]
#[path = "bindgen/linux.rs"]
mod linux;

#[cfg(not(feature = "libc_platform"))]
#[path = "bindgen/freebsd.rs"]
mod freebsd;

#[cfg(not(feature = "libc_platform"))]
#[path = "bindgen/macos.rs"]
mod macos;

#[cfg(not(feature = "libc_platform"))]
#[path = "bindgen/qnx.rs"]
mod qnx;

#[cfg(not(feature = "libc_platform"))]
#[path = "bindgen/windows.rs"]
mod windows;

#[cfg(not(feature = "libc_platform"))]
#[path = "bindgen/android.rs"]
mod android;
