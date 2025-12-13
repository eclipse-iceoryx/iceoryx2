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

pub fn configure_cargo() {
    // for Android a libc base platform abstraction is used;
    // to simplify the build process, the 'libc_platform' feature flag is set here
    // instead of requiring the user to set it
    println!("cargo::rustc-cfg=feature=\"libc_platform\"");
}
