// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

pub fn run(target_os: &str) {
    extern crate bindgen;
    extern crate cc;

    use bindgen::*;
    use std::env;
    use std::path::PathBuf;

    println!("cargo:rerun-if-changed=src/c/posix.h");
    println!("cargo:rerun-if-changed=src/c/socket_macros.c");

    println!("Building for target: {}", target_os);

    configure_cargo(target_os);

    let mut builder = bindgen::Builder::default()
        .header("src/c/posix.h")
        .blocklist_type("max_align_t")
        .parse_callbacks(Box::new(CargoCallbacks::new()))
        .use_core();

    builder = configure_builder(target_os, builder);

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

pub fn configure_cargo(target_os: &str) {
    match target_os {
        "freebsd" => freebsd::configure_cargo(),
        "macos" => macos::configure_cargo(),
        "nto" => qnx::configure_cargo(),
        "windows" => windows::configure_cargo(),
        _ => panic!("Unsupported target OS: {}", target_os),
    }
}

pub fn configure_builder(target_os: &str, builder: bindgen::Builder) -> bindgen::Builder {
    match target_os {
        "freebsd" => freebsd::configure_builder(builder),
        "macos" => macos::configure_builder(builder),
        "nto" => qnx::configure_builder(builder),
        "windows" => windows::configure_builder(builder),
        _ => panic!("Unsupported target OS: {}", target_os),
    }
}

mod freebsd;
mod macos;
mod qnx;
mod windows;
