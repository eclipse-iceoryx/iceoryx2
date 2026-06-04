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

use std::sync::Mutex;

use crate::posix::types::int;

const MAX_SHM_ENTRIES: usize = 1024;

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShmEntry {
    pub shm_fd: int,
    pub state_fd: int,
}

#[doc(hidden)]
pub struct ShmFdTranslator {
    entries: Mutex<[Option<ShmEntry>; MAX_SHM_ENTRIES]>,
}

impl ShmFdTranslator {
    const fn new() -> Self {
        const INIT: Option<ShmEntry> = None;
        Self {
            entries: Mutex::new([INIT; MAX_SHM_ENTRIES]),
        }
    }

    pub fn get_instance() -> &'static Self {
        static INSTANCE: ShmFdTranslator = ShmFdTranslator::new();
        &INSTANCE
    }

    pub fn register(&self, shm_fd: int, state_fd: int) -> bool {
        let mut entries = self.entries.lock().expect("ShmFdTranslator mutex poisoned");
        let mut free_slot: Option<usize> = None;
        for (idx, slot) in entries.iter().enumerate() {
            match slot {
                Some(e) if e.shm_fd == shm_fd => return false,
                None if free_slot.is_none() => free_slot = Some(idx),
                _ => {}
            }
        }
        match free_slot {
            Some(idx) => {
                entries[idx] = Some(ShmEntry { shm_fd, state_fd });
                true
            }
            None => false,
        }
    }

    pub fn lookup_state_fd(&self, shm_fd: int) -> Option<int> {
        let entries = self.entries.lock().expect("ShmFdTranslator mutex poisoned");
        entries.iter().find_map(|slot| match slot {
            Some(e) if e.shm_fd == shm_fd => Some(e.state_fd),
            _ => None,
        })
    }

    pub fn unregister(&self, shm_fd: int) -> Option<int> {
        let mut entries = self.entries.lock().expect("ShmFdTranslator mutex poisoned");
        for slot in entries.iter_mut() {
            if let Some(e) = *slot
                && e.shm_fd == shm_fd
            {
                *slot = None;
                return Some(e.state_fd);
            }
        }
        None
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
        let t = ShmFdTranslator::new();
        for i in 0..256 {
            assert!(t.register(i, i + 10_000));
        }
        for i in 0..256 {
            assert_eq!(t.lookup_state_fd(i), Some(i + 10_000));
        }
        for i in (0..256).rev() {
            assert_eq!(t.unregister(i), Some(i + 10_000));
        }
    }
}
