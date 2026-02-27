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

const ENV_PLATFORM_BINDING: &str = "IOX2_PLATFORM_BINDING";
const ENV_PLATFORM_PATH: &str = "IOX2_CUSTOM_POSIX_PLATFORM_PATH";

#[derive(PartialEq)]
enum PlatformBinding {
    Bindgen,
    Libc,
}

impl PlatformBinding {
    fn from_env() -> Self {
        println!("cargo:rerun-if-env-changed={}", ENV_PLATFORM_BINDING);

        match std::env::var(ENV_PLATFORM_BINDING).as_deref() {
            Ok("libc") => PlatformBinding::Libc,
            Ok("bindgen") | Ok("") | Err(_) => PlatformBinding::Bindgen,
            Ok(other) => panic!(
                "Unknown {} value: '{}', expected 'libc' or 'bindgen'",
                ENV_PLATFORM_BINDING, other
            ),
        }
    }
}

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
    let binding = configure_platform_binding(&target_os);
    if binding == PlatformBinding::Libc {
        return;
    }

    // the cfg guard below refers to native compilation and prevents bindgen
    // from being pulled in as a dependency when building on android
    //
    // the target_os check refers for the target compilation which could be
    // a different platform
    #[cfg(not(target_os = "android"))]
    if target_os != "android" {
        run_bindgen(target_os.as_str());
    }
}

// #[cfg(any(...))] does not work when cross-compiling
// when cross compiling, 'target_os' is set to the environment the build script
// is executed; to get the actual target OS, use the cargo 'TARGET' env variable
#[cfg(not(target_os = "android"))]
fn run_bindgen(target_os: &str) {
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

fn configure_platform_binding(target_os: &str) -> PlatformBinding {
    println!("cargo:rustc-check-cfg=cfg(platform_binding, values(\"libc\", \"bindgen\"))");

    let binding = if target_os == "android" {
        // android builds always leverage libc
        PlatformBinding::Libc
    } else {
        PlatformBinding::from_env()
    };

    match &binding {
        PlatformBinding::Libc => {
            println!("cargo:warning=Using libc crate for platform binding");
            println!("cargo:rustc-cfg=platform_binding=\"libc\"");
        }
        PlatformBinding::Bindgen => {
            println!("cargo:warning=Using bindgen for platform binding");
            println!("cargo:rustc-cfg=platform_binding=\"bindgen\"");
        }
    }

    binding
}

fn configure_cargo(target_os: &str) {
    match target_os {
        "freebsd" => freebsd::configure_cargo(),
        "linux" => linux::configure_cargo(),
        "macos" => macos::configure_cargo(),
        "nto" => qnx::configure_cargo(),
        "windows" => windows::configure_cargo(),
        "android" => android::configure_cargo(),
        _ => panic!("Unsupported target OS: {}", target_os),
    }
}

fn configure_builder(target_os: &str, builder: bindgen::Builder) -> bindgen::Builder {
    match target_os {
        "freebsd" => freebsd::configure_builder(builder),
        "linux" => linux::configure_builder(builder),
        "macos" => macos::configure_builder(builder),
        "nto" => qnx::configure_builder(builder),
        "windows" => windows::configure_builder(builder),
        _ => panic!("Unsupported target OS: {}", target_os),
    }
}

#[path = "bindgen/linux.rs"]
mod linux;

#[path = "bindgen/freebsd.rs"]
mod freebsd;

#[path = "bindgen/macos.rs"]
mod macos;

#[path = "bindgen/qnx.rs"]
mod qnx;

#[path = "bindgen/windows.rs"]
mod windows;

#[path = "bindgen/android.rs"]
mod android;
