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
    configure_configuration_override();
}

fn configure_configuration_override() {
    println!("cargo:rustc-check-cfg=cfg(configuration_override)");

    if let Ok(configuration_path) = std::env::var("IOX2_CUSTOM_PLATFORM_CONFIGURATION_PATH") {
        println!(
            "cargo:warning=Building with custom configuration: {}",
            configuration_path
        );

        // expose configuration_override as cfg option
        println!("cargo:rustc-cfg=configuration_override");
        println!(
            "cargo:rustc-env=IOX2_CUSTOM_PLATFORM_CONFIGURATION_PATH={}",
            configuration_path
        );
        println!("cargo:rerun-if-env-changed=IOX2_CUSTOM_PLATFORM_CONFIGURATION_PATH");
        println!("cargo:rerun-if-changed={}", configuration_path);
    }
}
