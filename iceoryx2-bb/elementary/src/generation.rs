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

//! Tracks whether a set of data may have changed since it was last looked
//! at, as a [`Generation`] its writer advances on every change.
//!
//! ```
//! use iceoryx2_bb_elementary::generation::*;
//!
//! let counter = GenerationCounter::new();
//! let seen = counter.current();
//!
//! // the writer changes the data
//! counter.advance();
//!
//! assert!(!counter.current().unchanged_since(seen));
//! ```

use iceoryx2_bb_concurrency::atomic::{AtomicU64, Ordering};

/// Tracks generation of a particular set of data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Generation {
    Untracked,
    At(u64),
}

impl Generation {
    /// Whether this is a tracked generation equal to `other`.
    pub fn unchanged_since(self, other: Generation) -> bool {
        match (self, other) {
            (Generation::At(current), Generation::At(previous)) => current == previous,
            _ => false,
        }
    }
}

/// A tracked generation advanced from any thread.
#[derive(Debug, Default)]
pub struct GenerationCounter {
    count: AtomicU64,
}

impl GenerationCounter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Advances the generation.
    pub fn advance(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    /// The generation as of now.
    pub fn current(&self) -> Generation {
        Generation::At(self.count.load(Ordering::Relaxed))
    }
}
