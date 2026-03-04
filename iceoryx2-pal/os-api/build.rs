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

fn main() {
    // #[cfg(any(...))] does not work when cross-compiling
    // when cross compiling, 'target_os' is set to the environment the build script
    // is executed; to get the actual target OS, use the cargo 'CARGO_CFG_TARGET_OS' env variable
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    // needed for bazel but can be empty for cargo builds
    println!("cargo:rustc-env=BAZEL_BINDGEN_PATH_CORRECTION=");

    // define bazel_build as a valid cfg to avoid errors/warnings, but not used for cargo builds
    println!("cargo:rustc-check-cfg=cfg(bazel_build)");

    println!("Building for target: {}", target_os);
}
