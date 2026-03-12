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
const BINDGEN_PLATFORMS: &[&str] = &["windows", "macos", "freebsd", "nto"];
const LIBC_PLATFORMS: &[&str] = &["android", "linux", "vxworks"];

fn main() {
    // when cross compiling, 'CARGO_CFG_TARGET_OS' is set to the compilation
    // target in the environment of the build script
    //
    // #[cfg(any(...))] cannot be used for this purpose as it refers to the
    // (cross-) compilation host
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    // define bazel_build as a valid cfg to avoid errors/warnings, but not used for cargo builds
    println!("cargo:rustc-check-cfg=cfg(bazel_build)");

    configure_platform_override();
    configure_platform_binding(&target_os);

    // the cfg guard below refers to native compilation of build.rs and prevents
    // bindgen from being pulled in as a dependency when a (cross-) compilation
    // host does not support it
    //
    // the target_os check refers for the target compilation which could be
    // a different platform
    #[cfg(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos",
        target_os = "freebsd",
        target_os = "nto"
    ))]
    if BINDGEN_PLATFORMS.contains(&target_os.as_str()) {
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

fn configure_platform_binding(target_os: &str) {
    println!("cargo:rustc-check-cfg=cfg(platform_binding, values(\"libc\", \"bindgen\"))");
    if BINDGEN_PLATFORMS.contains(&target_os) {
        println!("cargo:rustc-cfg=platform_binding=\"bindgen\"");
    }
    if LIBC_PLATFORMS.contains(&target_os) {
        println!("cargo:rustc-cfg=platform_binding=\"libc\"");
    }
}

// the cfg guard below refers to native compilation of build.rs and prevents
// bindgen from being pulled in as a dependency when a (cross-) compilation
// host does not support it
#[cfg(any(
    target_os = "linux",
    target_os = "windows",
    target_os = "macos",
    target_os = "freebsd",
    target_os = "nto"
))]
mod bindgen;
