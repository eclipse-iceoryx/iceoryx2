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

extern crate bindgen;
extern crate cc;

use bindgen::*;
use std::env;
use std::path::PathBuf;

fn main() {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    println!("cargo:rustc-link-lib=pthread");

    #[cfg(all(target_os = "linux", feature = "acl"))]
    println!("cargo:rustc-link-lib=acl");
    println!("cargo:rerun-if-changed=src/c/posix.h");

    let bindings = if std::env::var("DOCS_RS").is_ok() {
        bindgen::Builder::default()
            .header("src/c/posix.h")
            .parse_callbacks(Box::new(CargoCallbacks::new()))
            .clang_arg("-D IOX2_DOCS_RS_SUPPORT")
            .use_core()
            .generate()
            .expect("Unable to generate bindings")
    } else {
        #[cfg(not(feature = "acl"))]
        {
            bindgen::Builder::default()
                .header("src/c/posix.h")
                .parse_callbacks(Box::new(CargoCallbacks::new()))
                .use_core()
                .generate()
                .expect("Unable to generate bindings")
        }

        #[cfg(feature = "acl")]
        {
            bindgen::Builder::default()
                .header("src/c/posix.h")
                .parse_callbacks(Box::new(CargoCallbacks::new()))
                .clang_arg("-D IOX2_ACL_SUPPORT")
                .generate()
                .expect("Unable to generate bindings")
        }
    };

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("posix_generated.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=src/c/sigaction.c");
    cc::Build::new()
        .file("src/c/sigaction.c")
        .compile("libsigaction.a");

    println!("cargo:rerun-if-changed=src/c/socket_macros.c");
    cc::Build::new()
        .file("src/c/socket_macros.c")
        .compile("libsocket_macros.a");

    println!("cargo:rerun-if-changed=src/c/dirent.c");
    cc::Build::new()
        .file("src/c/dirent.c")
        .compile("libdirent.a");
}
