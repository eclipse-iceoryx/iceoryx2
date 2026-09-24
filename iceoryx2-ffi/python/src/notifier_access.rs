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

//! Admission for Python notifier operations. Never hold this mutex while calling Python.
use std::sync::{Condvar, Mutex};
use std::thread::{self, ThreadId};

#[derive(Default)]
enum State {
    #[default]
    Idle,
    Ordinary(ThreadId),
    Traversal,
}

#[derive(Clone, Copy)]
pub(crate) enum AccessKind {
    Ordinary,
    Traversal,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum AccessError {
    TraversalActive,
    Reentrant,
    Poisoned,
}

#[derive(Default)]
pub(crate) struct NotifierAccess {
    state: Mutex<State>,
    changed: Condvar,
}

pub(crate) struct Permit<'a>(&'a NotifierAccess);

impl NotifierAccess {
    /// Called while detached from Python, so a waiting caller cannot prevent the
    /// current owner from reacquiring the GIL. Returns no held MutexGuard.
    pub(crate) fn acquire(&self, kind: AccessKind) -> Result<Permit<'_>, AccessError> {
        let current = thread::current().id();
        let mut state = self.state.lock().map_err(|_| AccessError::Poisoned)?;
        loop {
            match *state {
                State::Idle => {
                    *state = match kind {
                        AccessKind::Ordinary => State::Ordinary(current),
                        AccessKind::Traversal => State::Traversal,
                    };
                    // Wake ordinary waiters to reject them if a traversal won
                    // admission. They must not keep waiting through callbacks.
                    if matches!(kind, AccessKind::Traversal) {
                        self.changed.notify_all();
                    }
                    return Ok(Permit(self));
                }
                State::Traversal => return Err(AccessError::TraversalActive),
                State::Ordinary(owner) if owner == current => return Err(AccessError::Reentrant),
                State::Ordinary(_) => {
                    state = self
                        .changed
                        .wait(state)
                        .map_err(|_| AccessError::Poisoned)?;
                }
            }
        }
    }
}

impl Drop for Permit<'_> {
    fn drop(&mut self) {
        // No Python/user code runs under the state mutex. Even if unwinding
        // poisoned it, release the state and wake waiters rather than strand them.
        *self.0.state.lock().unwrap_or_else(|e| e.into_inner()) = State::Idle;
        self.0.changed.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::mpsc, time::Duration};

    #[test]
    fn ordinary_operations_wait_and_then_succeed() {
        let access = NotifierAccess::default();
        let permit = access.acquire(AccessKind::Ordinary).unwrap();
        thread::scope(|scope| {
            let (started, ready) = mpsc::channel();
            let (done, result) = mpsc::channel();
            let access_ref = &access;
            scope.spawn(move || {
                started.send(()).unwrap();
                let permit = access_ref.acquire(AccessKind::Ordinary);
                done.send(permit.is_ok()).unwrap();
            });
            ready.recv_timeout(Duration::from_secs(2)).unwrap();
            assert!(matches!(
                result.recv_timeout(Duration::from_millis(50)),
                Err(mpsc::RecvTimeoutError::Timeout)
            ));
            drop(permit);
            assert!(result.recv_timeout(Duration::from_secs(2)).unwrap());
        });
    }

    #[test]
    fn traversal_rejects_both_kinds_on_all_threads() {
        let access = NotifierAccess::default();
        let _permit = access.acquire(AccessKind::Traversal).unwrap();
        for kind in [AccessKind::Ordinary, AccessKind::Traversal] {
            assert!(matches!(
                access.acquire(kind),
                Err(AccessError::TraversalActive)
            ));
            thread::scope(|scope| {
                assert!(
                    scope
                        .spawn(|| matches!(access.acquire(kind), Err(AccessError::TraversalActive)))
                        .join()
                        .unwrap()
                );
            });
        }
    }

    #[test]
    fn ordinary_same_thread_reentry_fails_instead_of_waiting() {
        let access = NotifierAccess::default();
        let _permit = access.acquire(AccessKind::Ordinary).unwrap();
        assert!(matches!(
            access.acquire(AccessKind::Ordinary),
            Err(AccessError::Reentrant)
        ));
        assert!(matches!(
            access.acquire(AccessKind::Traversal),
            Err(AccessError::Reentrant)
        ));
    }

    #[test]
    fn unwinding_releases_admission() {
        let access = NotifierAccess::default();
        let failure = std::panic::catch_unwind(|| {
            let _permit = access.acquire(AccessKind::Traversal).unwrap();
            panic!("simulated callback unwind");
        });
        assert!(failure.is_err());
        assert!(access.acquire(AccessKind::Ordinary).is_ok());
    }
}
