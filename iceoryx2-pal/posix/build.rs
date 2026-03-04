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

const ENV_PLATFORM_PATH: &str = "IOX2_CUSTOM_POSIX_PLATFORM_PATH";

fn main() {
    // #[cfg(any(...))] does not work when cross-compiling
    // when cross compiling, 'target_os' is set to the environment the build script
    // is executed; to get the actual target OS, use the cargo 'CARGO_CFG_TARGET_OS' env variable
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "none" {
        return;
    }

    // needed for bazel but can be empty for cargo builds
    println!("cargo:rustc-env=BAZEL_BINDGEN_PATH_CORRECTION=");

    // define bazel_build as a valid cfg to avoid errors/warnings, but not used for cargo builds
    println!("cargo:rustc-check-cfg=cfg(bazel_build)");

    configure_platform_override();

    // the cfg guard below refers to native compilation and prevents bindgen
    // from being pulled in as a dependency when building using the libc crate
    //
    // the target_os check refers for the target compilation which could be
    // a different platform
    #[cfg(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "freebsd",
        target_os = "nto"
    ))]
    if target_os != "android" {
        bindgen::run(target_os.as_str());
    }
}

fn configure_platform_override() {
    println!("cargo:rustc-check-cfg=cfg(platform_override)");

    if let Ok(platform_path) = std::env::var(ENV_PLATFORM_PATH) {
        let module_path = std::path::Path::new(&platform_path).join("os.rs");
        if !module_path.exists() {
            panic!("The path '{platform_path}' does not contain an 'os.rs' file");
        }

        println!(
            "cargo:warning=Building with custom POSIX abstraction at: {}",
            platform_path
        );

        println!("cargo:rustc-cfg=platform_override");
        println!("cargo:rustc-env={}={}", ENV_PLATFORM_PATH, platform_path);
        println!("cargo:rerun-if-env-changed={}", ENV_PLATFORM_PATH);
        println!("cargo:rerun-if-changed={}", platform_path);
    }
}

// #[cfg(any(...))] does not work when cross-compiling
// when cross compiling, 'target_os' is set to the environment the build script
// is executed; to get the actual target OS, use the cargo 'TARGET' env variable
#[cfg(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "freebsd",
    target_os = "nto"
))]
mod bindgen;
