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

// iox2-156: shm_fd → state_fd mapping for permission trampoline routing.

use alloc::collections::BTreeMap;
use std::sync::Mutex;

use crate::posix::types::int;

#[doc(hidden)]
pub struct ShmFdTranslator {
    // shm_fd -> state_fd
    entries: Mutex<BTreeMap<int, int>>,
}

impl ShmFdTranslator {
    const fn new() -> Self {
        Self {
            entries: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn get_instance() -> &'static Self {
        static INSTANCE: ShmFdTranslator = ShmFdTranslator::new();
        &INSTANCE
    }

    pub fn register(&self, shm_fd: int, state_fd: int) -> bool {
        let mut entries = self.entries.lock().expect("ShmFdTranslator mutex poisoned");
        if entries.contains_key(&shm_fd) {
            return false;
        }
        entries.insert(shm_fd, state_fd);
        true
    }

    pub fn lookup_state_fd(&self, shm_fd: int) -> Option<int> {
        let entries = self.entries.lock().expect("ShmFdTranslator mutex poisoned");
        entries.get(&shm_fd).copied()
    }

    pub fn unregister(&self, shm_fd: int) -> Option<int> {
        let mut entries = self.entries.lock().expect("ShmFdTranslator mutex poisoned");
        entries.remove(&shm_fd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_lookup_unregister_roundtrip() {
        let t = ShmFdTranslator::new();
        assert!(t.register(100, 200));
        assert_eq!(t.lookup_state_fd(100), Some(200));
        assert_eq!(t.unregister(100), Some(200));
        assert_eq!(t.lookup_state_fd(100), None);
        assert_eq!(t.unregister(100), None);
    }

    #[test]
    fn duplicate_register_is_rejected() {
        let t = ShmFdTranslator::new();
        assert!(t.register(7, 8));
        assert!(!t.register(7, 9));
        assert_eq!(t.lookup_state_fd(7), Some(8));
    }

    #[test]
    fn many_entries_independent() {
        // BTreeMap has no fixed capacity, so go well past the old 1024 array limit.
        let t = ShmFdTranslator::new();
        for i in 0..5000 {
            assert!(t.register(i, i + 10_000));
        }
        for i in 0..5000 {
            assert_eq!(t.lookup_state_fd(i), Some(i + 10_000));
        }
        for i in (0..5000).rev() {
            assert_eq!(t.unregister(i), Some(i + 10_000));
        }
    }

    #[test]
    fn reregister_after_unregister_succeeds() {
        // A closed shm fd may be reused by the OS for a new shm later.
        let t = ShmFdTranslator::new();
        assert!(t.register(42, 100));
        assert_eq!(t.unregister(42), Some(100));
        assert!(t.register(42, 200));
        assert_eq!(t.lookup_state_fd(42), Some(200));
    }
}
