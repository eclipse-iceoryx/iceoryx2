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

const ENV_PLATFORM_BINDING: &str = "IOX2_PLATFORM_BINDING";

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

    // needed for bazel but can be empty for cargo builds
    println!("cargo:rustc-env=BAZEL_BINDGEN_PATH_CORRECTION=");

    // define bazel_build as a valid cfg to avoid errors/warnings, but not used for cargo builds
    println!("cargo:rustc-check-cfg=cfg(bazel_build)");

    let binding = configure_platform_binding(&target_os);
    if binding == PlatformBinding::Libc {
        return;
    }

    println!("Building for target: {}", target_os);

    // the cfg guard below refers to native compilation and prevents bindgen
    // from being pulled in as a dependency when building on a non-linux host
    //
    // the target_os check refers to the target compilation which could be
    // a different platform
    #[cfg(target_os = "linux")]
    if target_os == "linux" {
        run_bindgen();
    }
}

#[cfg(target_os = "linux")]
fn run_bindgen() {
    extern crate bindgen;

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

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("os_api_generated.rs"))
        .expect("Couldn't write bindings!");
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
