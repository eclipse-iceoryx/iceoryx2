// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

extern crate cbindgen;

use std::env;
use std::path::Path;

fn main() {
    // this is the out dir of the iceoryx2-ffi-c crate not the workspace out dir,
    // therefore we need to traverse to a known location and create the path for the header
    let out_dir = env::var("OUT_DIR").expect("Target output directory");

    let mut header_path = Path::new(&out_dir)
        .join("../../../")
        .canonicalize()
        .expect("Path to iceoryx2 base dir for header generation");
    header_path.push("iceoryx2-ffi-c-cbindgen/include/iox2/iceoryx2.h");

    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("Cargo manifest dir");

    cbindgen::generate(crate_dir)
        .expect("Unable to generate c bindings")
        .write_to_file(header_path);
}
