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

//! Link/rpath the rosidl-generated C libraries the message crate refers to.

fn main() {
    let prefix_path = std::env::var("AMENT_PREFIX_PATH")
        .expect("AMENT_PREFIX_PATH not set - source the ROS 2 workspace before building");
    for prefix in prefix_path.split(':') {
        let lib = format!("{prefix}/lib");
        println!("cargo:rustc-link-search=native={lib}");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{lib}");
    }
}
