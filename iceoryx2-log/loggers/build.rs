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
    // Validate platform selection
    if cfg!(all(feature = "std", feature = "posix")) {
        panic!("Cannot enable both 'std' and 'posix' features simultaneously");
    }

    if cfg!(all(feature = "std", feature = "bare_metal")) {
        panic!("Cannot enable both 'std' and 'bare_metal' features simultaneously");
    }

    if cfg!(all(feature = "posix", feature = "bare_metal")) {
        panic!("Cannot enable both 'posix' and 'bare_metal' features simultaneously");
    }
    if !cfg!(any(
        feature = "std",
        feature = "posix",
        feature = "bare_metal"
    )) {
        println!("cargo:warning=No platform feature selected ('std', 'posix', or 'bare_metal'). Log output may be discarded.");
    }

    // Validate logger selection
    if cfg!(all(feature = "buffer", feature = "file")) {
        panic!("Cannot enable both 'buffer' and 'file' features simultaneously");
    }

    if cfg!(all(feature = "buffer", feature = "console")) {
        panic!("Cannot enable both 'buffer' and 'console' features simultaneously");
    }

    if cfg!(all(feature = "buffer", feature = "log")) {
        panic!("Cannot enable both 'buffer' and 'log' features simultaneously");
    }

    if cfg!(all(feature = "buffer", feature = "tracing")) {
        panic!("Cannot enable both 'buffer' and 'tracing' features simultaneously");
    }

    if cfg!(all(feature = "file", feature = "console")) {
        panic!("Cannot enable both 'file' and 'console' features simultaneously");
    }

    if cfg!(all(feature = "file", feature = "log")) {
        panic!("Cannot enable both 'file' and 'log' features simultaneously");
    }

    if cfg!(all(feature = "file", feature = "tracing")) {
        panic!("Cannot enable both 'file' and 'tracing' features simultaneously");
    }

    if cfg!(all(feature = "console", feature = "log")) {
        panic!("Cannot enable both 'console' and 'log' features simultaneously");
    }

    if cfg!(all(feature = "console", feature = "tracing")) {
        panic!("Cannot enable both 'console' and 'tracing' features simultaneously");
    }

    if cfg!(all(feature = "log", feature = "tracing")) {
        panic!("Cannot enable both 'log' and 'tracing' features simultaneously");
    }

    // Prevent invalid platform-logger combinations
    if cfg!(all(feature = "posix", feature = "buffer")) {
        panic!("Invalid combination: 'posix' does not support 'buffer' logger");
    }

    if cfg!(all(feature = "posix", feature = "file")) {
        panic!("Invalid combination: 'posix' does not support 'file' logger");
    }

    if cfg!(all(feature = "posix", feature = "log")) {
        panic!("Invalid combination: 'posix' does not support 'log' (requires std)");
    }

    if cfg!(all(feature = "posix", feature = "tracing")) {
        panic!("Invalid combination: 'posix' does not support 'tracing' (requires std)");
    }

    if cfg!(all(feature = "bare_metal", feature = "console")) {
        panic!("Invalid combination: 'bare_metal' does not support 'console' logger");
    }

    if cfg!(all(feature = "bare_metal", feature = "buffer")) {
        panic!("Invalid combination: 'bare_metal' does not support 'buffer' logger");
    }

    if cfg!(all(feature = "bare_metal", feature = "file")) {
        panic!("Invalid combination: 'bare_metal' does not support 'file' logger");
    }

    if cfg!(all(feature = "bare_metal", feature = "log")) {
        panic!("Invalid combination: 'bare_metal' does not support 'log' (requires std)");
    }

    if cfg!(all(feature = "bare_metal", feature = "tracing")) {
        panic!("Invalid combination: 'bare_metal' does not support 'tracing' (requires std)");
    }
}
