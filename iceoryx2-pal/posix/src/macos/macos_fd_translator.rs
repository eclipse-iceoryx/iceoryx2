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
//
// The table lives in a static of this crate, so a process that links the crate
// more than once (an executable plus Rust cdylibs it loads, each statically
// linked) has one table per copy. A descriptor opened through one copy can be
// closed through another, which removes nothing from the opening copy's table:
// the entry outlives its descriptor and the kernel hands the number out again.
// Two rules keep such a stale entry from being used:
//
// * `register` replaces an existing entry. `shm_open` has just returned this
//   number, so no open descriptor of this process uses it and the old entry is
//   stale. The replaced state fd is closed when it still refers to the file it
//   was registered with.
// * `lookup_state_fd` answers only while the descriptor is a POSIX shared
//   memory object (`fstat` reports no file-type bits for those on macOS). For
//   any other descriptor the entry is stale; it is removed and the caller
//   operates on the descriptor itself. The check cannot tell two shared memory
//   objects apart, so a stale entry whose number another copy reused for its
//   own `shm_open` still answers until that number is closed or registered
//   here again.

use alloc::collections::BTreeMap;
use std::sync::Mutex;

use crate::posix::types::int;

/// Device and inode of an open descriptor.
type FileId = (libc::dev_t, libc::ino_t);

fn fstat_native(fd: int) -> Option<libc::stat> {
    let mut st = core::mem::MaybeUninit::<libc::stat>::zeroed();
    if unsafe { libc::fstat(fd, st.as_mut_ptr()) } == 0 {
        Some(unsafe { st.assume_init() })
    } else {
        None
    }
}

fn file_id(fd: int) -> Option<FileId> {
    fstat_native(fd).map(|st| (st.st_dev, st.st_ino))
}

fn is_posix_shared_memory(fd: int) -> bool {
    fstat_native(fd).is_some_and(|st| st.st_mode & libc::S_IFMT == 0)
}

struct Entry {
    state_fd: int,
    // `None` when the state fd could not be stat'ed at registration; such a
    // state fd is never closed on behalf of a stale entry.
    state_id: Option<FileId>,
}

impl Entry {
    /// The state fd, if it is still the file this entry was registered with.
    fn live_state_fd(&self) -> Option<int> {
        match self.state_id {
            Some(id) if file_id(self.state_fd) == Some(id) => Some(self.state_fd),
            _ => None,
        }
    }

    /// Close the state fd of an entry that no longer has a shm descriptor.
    fn release(self) {
        if let Some(state_fd) = self.live_state_fd() {
            unsafe { libc::close(state_fd) };
        }
    }
}

#[doc(hidden)]
pub struct ShmFdTranslator {
    // shm_fd -> state_fd
    entries: Mutex<BTreeMap<int, Entry>>,
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

    fn entries(&self) -> std::sync::MutexGuard<'_, BTreeMap<int, Entry>> {
        self.entries.lock().expect("ShmFdTranslator mutex poisoned")
    }

    /// Map `shm_fd`, just returned by `shm_open`, to its state fd. An entry
    /// already present under `shm_fd` is stale and is replaced.
    pub fn register(&self, shm_fd: int, state_fd: int) {
        let entry = Entry {
            state_fd,
            state_id: file_id(state_fd),
        };
        let replaced = self.entries().insert(shm_fd, entry);
        // The stale state fd number can be the one just opened for this entry
        // when it was closed outside the table and reused; it is then live.
        if let Some(stale) = replaced.filter(|stale| stale.state_fd != state_fd) {
            stale.release();
        }
    }

    /// The state fd of `shm_fd` while `shm_fd` is a shared memory object.
    pub fn lookup_state_fd(&self, shm_fd: int) -> Option<int> {
        let mut entries = self.entries();
        let state_fd = entries.get(&shm_fd)?.state_fd;
        if is_posix_shared_memory(shm_fd) {
            return Some(state_fd);
        }
        let stale = entries.remove(&shm_fd);
        drop(entries);
        if let Some(stale) = stale {
            stale.release();
        }
        None
    }

    /// Remove the entry of `shm_fd` and return its state fd for the caller to
    /// close, unless that state fd no longer refers to the registered file.
    pub fn unregister(&self, shm_fd: int) -> Option<int> {
        let entry = self.entries().remove(&shm_fd)?;
        entry.live_state_fd()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The tests assert which number the kernel hands out next. Every descriptor
    // they open is moved to a range no other test of this crate uses, and the
    // tests run one at a time, so "lowest free number in the range" is exact.
    static SERIAL: Mutex<()> = Mutex::new(());
    const RANGE_START: int = 200;

    fn serial() -> std::sync::MutexGuard<'static, ()> {
        SERIAL.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn into_range(fd: int) -> int {
        assert!(fd >= 0, "open failed");
        let moved = unsafe { libc::fcntl(fd, libc::F_DUPFD, RANGE_START) };
        unsafe { libc::close(fd) };
        assert!(moved >= RANGE_START, "F_DUPFD failed");
        moved
    }

    /// Closes the descriptor on drop.
    struct Fd(int);

    impl Drop for Fd {
        fn drop(&mut self) {
            unsafe { libc::close(self.0) };
        }
    }

    fn shm(tag: &str) -> int {
        let name = format!("/iox2_fdt_{tag}_{}\0", std::process::id());
        let fd = unsafe {
            libc::shm_open(
                name.as_ptr().cast(),
                libc::O_CREAT | libc::O_RDWR,
                0o600 as libc::c_uint,
            )
        };
        unsafe { libc::shm_unlink(name.as_ptr().cast()) };
        into_range(fd)
    }

    fn file() -> int {
        into_range(unsafe { libc::open(c"/dev/null".as_ptr(), libc::O_RDONLY) })
    }

    fn is_open(fd: int) -> bool {
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        flags != -1
    }

    #[test]
    fn register_lookup_unregister_roundtrip() {
        let _serial = serial();
        let t = ShmFdTranslator::new();
        let (s, st) = (Fd(shm("roundtrip")), Fd(file()));
        t.register(s.0, st.0);
        assert_eq!(t.lookup_state_fd(s.0), Some(st.0));
        assert_eq!(t.unregister(s.0), Some(st.0));
        assert_eq!(t.lookup_state_fd(s.0), None);
        assert_eq!(t.unregister(s.0), None);
    }

    #[test]
    fn register_replaces_a_stale_entry_and_closes_its_state_fd() {
        let _serial = serial();
        let t = ShmFdTranslator::new();
        let s = Fd(shm("replace"));
        // Closed by the table, so not wrapped in `Fd`.
        let old_state = file();
        let new_state = Fd(file());
        t.register(s.0, old_state);
        // Another copy closed the shm fd and the kernel returned the number.
        t.register(s.0, new_state.0);
        assert_eq!(t.lookup_state_fd(s.0), Some(new_state.0));
        assert!(!is_open(old_state), "the replaced state fd must be closed");
        assert!(is_open(new_state.0));
    }

    #[test]
    fn a_replaced_state_fd_reused_for_another_file_is_not_closed() {
        let _serial = serial();
        let t = ShmFdTranslator::new();
        let s = Fd(shm("reused"));
        let old_state = file();
        t.register(s.0, old_state);
        // The state fd is closed outside the table and its number reused.
        unsafe { libc::close(old_state) };
        let other = Fd(shm("reused_other"));
        assert_eq!(other.0, old_state, "the number must be handed out again");
        let new_state = Fd(file());
        t.register(s.0, new_state.0);
        assert!(is_open(other.0), "an unrelated descriptor must stay open");
    }

    #[test]
    fn a_replaced_state_fd_reused_as_the_new_state_fd_stays_open() {
        let _serial = serial();
        let t = ShmFdTranslator::new();
        let s = Fd(shm("same_state"));
        let state = Fd(file());
        t.register(s.0, state.0);
        // The same shared memory object is opened again and its state file
        // lands on the number the stale entry still holds.
        t.register(s.0, state.0);
        assert!(is_open(state.0), "the new state fd must stay open");
        assert_eq!(t.lookup_state_fd(s.0), Some(state.0));
    }

    #[test]
    fn lookup_drops_an_entry_whose_number_now_names_a_regular_file() {
        let _serial = serial();
        let t = ShmFdTranslator::new();
        let number = shm("lookup");
        // Closed by the table, so not wrapped in `Fd`.
        let state = file();
        t.register(number, state);
        // Another copy closes the shm fd; the number is reused for a file.
        unsafe { libc::close(number) };
        let reused = Fd(file());
        assert_eq!(reused.0, number, "the number must be handed out again");
        assert_eq!(t.lookup_state_fd(reused.0), None);
        assert!(!is_open(state), "the stale entry's state fd must be closed");
        assert_eq!(t.unregister(reused.0), None);
    }

    #[test]
    fn many_entries_independent() {
        let _serial = serial();
        // BTreeMap has no fixed capacity, so go well past the old 1024 array limit.
        let t = ShmFdTranslator::new();
        let state = Fd(file());
        for i in 0..5000 {
            t.register(100_000 + i, state.0);
        }
        for i in (0..5000).rev() {
            assert_eq!(t.unregister(100_000 + i), Some(state.0));
        }
        assert_eq!(t.unregister(100_000), None);
    }

    #[test]
    fn reregister_after_unregister_succeeds() {
        let _serial = serial();
        // A closed shm fd may be reused by the OS for a new shm later.
        let t = ShmFdTranslator::new();
        let (s, a, b) = (Fd(shm("rereg")), Fd(file()), Fd(file()));
        t.register(s.0, a.0);
        assert_eq!(t.unregister(s.0), Some(a.0));
        t.register(s.0, b.0);
        assert_eq!(t.lookup_state_fd(s.0), Some(b.0));
        assert!(is_open(a.0));
    }
}
