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

#[cfg(feature = "libc_platform")]
fn main() {}

#[cfg(all(not(feature = "libc_platform"), not(target_os = "linux")))]
fn main() {}

#[cfg(all(not(feature = "libc_platform"), target_os = "linux"))]
fn main() {
    extern crate bindgen;
    extern crate cc;

    use bindgen::*;
    use std::env;
    use std::path::PathBuf;

    println!("cargo:rerun-if-changed=src/c/linux.h");

    let mut builder = bindgen::Builder::default()
        .header("src/c/linux.h")
        .parse_callbacks(Box::new(CargoCallbacks::new()))
        .use_core();

    if std::env::var("DOCS_RS").is_ok() {
        builder = builder.clang_arg("-D IOX2_DOCS_RS_SUPPORT");
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    // needed for bazel but can be empty for cargo builds
    println!("cargo:rustc-env=BAZEL_BINDGEN_PATH_CORRECTION=");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("os_api_generated.rs"))
        .expect("Couldn't write bindings!");
}
