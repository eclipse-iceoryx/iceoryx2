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
    if cfg!(all(not(feature = "std"), feature = "buffer")) {
        panic!("Invalid combination: 'buffer' logger is only available for 'std' builds");
    }
    if cfg!(all(not(feature = "std"), feature = "file")) {
        panic!("Invalid combination: 'file' logger is only available for 'std' builds");
    }
    if cfg!(all(not(feature = "std"), feature = "log")) {
        panic!("Invalid combination: 'log' logger is only available for 'std' builds");
    }
    if cfg!(all(not(feature = "std"), feature = "tracing")) {
        panic!("Invalid combination: 'tracing' logger is only available for 'std' builds");
    }
}
