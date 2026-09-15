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

//! Counts the updates some state has gone through, so what an update saw
//! can be told from what it did not.
//!
//! ```
//! use iceoryx2_bb_elementary::epoch::Epoch;
//!
//! let mut epoch = Epoch::default();
//! let seen = epoch;
//!
//! // an update
//! epoch = epoch.next();
//!
//! assert!(seen < epoch);
//! ```

/// A count of the updates some state has gone through. What an update
/// sees is stamped with its epoch, what carries an older stamp was not
/// seen since.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Epoch(u64);

impl Epoch {
    /// The epoch of the next update.
    pub fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}
