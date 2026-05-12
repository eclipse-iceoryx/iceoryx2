#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! imperative = "1.0.7"
//! ```

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

// Checks whether a single word is in imperative mood.
//
// Usage: check_imperative_mood.rs <word>
// Exits 0 if the word is imperative or unknown to the dictionary.
// Exits 1 if the word is recognized as non-imperative

use imperative::Mood;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(word) = args.next() else {
        eprintln!("usage: check_imperative_mood.rs <word>");
        return ExitCode::from(2);
    };

    let lowered = word.to_lowercase();
    match Mood::new().is_imperative(&lowered) {
        Some(true) => ExitCode::SUCCESS,
        Some(false) => {
            eprintln!("'{word}' is not in imperative mood");
            ExitCode::FAILURE
        }
        None => ExitCode::SUCCESS,
    }
}
