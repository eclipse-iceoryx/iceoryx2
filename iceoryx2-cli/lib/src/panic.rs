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

#[macro_export]
macro_rules! install_panic_handlers {
    () => {
        #[cfg(not(debug_assertions))]
        {
            ::human_panic::setup_panic!();
        }
        #[cfg(debug_assertions)]
        {
            ::better_panic::Settings::debug()
                .most_recent_first(false)
                .lineno_suffix(true)
                .verbosity(::better_panic::Verbosity::Full)
                .install();
        }
    };
}
