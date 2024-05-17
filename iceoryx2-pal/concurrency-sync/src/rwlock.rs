// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use core::{hint::spin_loop, sync::atomic::Ordering};

use crate::iox_atomic::IoxAtomicU32;
use crate::{WaitAction, WaitResult, SPIN_REPETITIONS};

const WRITE_LOCKED: u32 = u32::MAX;
const UNLOCKED: u32 = 0;

pub struct RwLockReaderPreference {
    reader_count: IoxAtomicU32,
}

impl Default for RwLockReaderPreference {
    fn default() -> Self {
        Self::new()
    }
}

impl RwLockReaderPreference {
    pub const fn new() -> Self {
        Self {
            reader_count: IoxAtomicU32::new(UNLOCKED),
        }
    }

    pub fn try_read_lock(&self) -> WaitResult {
        let reader_count = self.reader_count.load(Ordering::Relaxed);

        if reader_count == WRITE_LOCKED {
            return WaitResult::Interrupted;
        }

        if self
            .reader_count
            .compare_exchange(
                reader_count,
                reader_count + 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            WaitResult::Success
        } else {
            WaitResult::Interrupted
        }
    }

    pub fn read_lock<F: Fn(&IoxAtomicU32, &u32) -> WaitAction>(&self, wait: F) -> WaitResult {
        let mut reader_count = self.reader_count.load(Ordering::Relaxed);
        let mut retry_counter = 0;

        let mut keep_running = true;
        loop {
            loop {
                if reader_count != WRITE_LOCKED {
                    break;
                }

                if !keep_running {
                    return WaitResult::Interrupted;
                }

                if retry_counter < SPIN_REPETITIONS {
                    retry_counter += 1;
                    spin_loop();
                } else if wait(&self.reader_count, &reader_count) == WaitAction::Abort {
                    keep_running = false;
                }

                reader_count = self.reader_count.load(Ordering::Relaxed);
            }

            match self.reader_count.compare_exchange(
                reader_count,
                reader_count + 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => return WaitResult::Success,
                Err(v) => {
                    reader_count = v;
                }
            }
        }
    }

    pub fn unlock<WakeOne: Fn(&IoxAtomicU32)>(&self, wake_one: WakeOne) {
        let state = self.reader_count.load(Ordering::Relaxed);
        if state == WRITE_LOCKED {
            self.reader_count.store(UNLOCKED, Ordering::Release);
        } else {
            self.reader_count.fetch_sub(1, Ordering::Relaxed);
        }
        wake_one(&self.reader_count);
    }

    pub fn try_write_lock(&self) -> WaitResult {
        if self
            .reader_count
            .compare_exchange(UNLOCKED, WRITE_LOCKED, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            WaitResult::Success
        } else {
            WaitResult::Interrupted
        }
    }

    pub fn write_lock<F: Fn(&IoxAtomicU32, &u32) -> WaitAction>(&self, wait: F) -> WaitResult {
        let mut retry_counter = 0;
        let mut reader_count;

        let mut keep_running = true;
        loop {
            match self.reader_count.compare_exchange(
                UNLOCKED,
                WRITE_LOCKED,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Err(v) => reader_count = v,
                Ok(_) => return WaitResult::Success,
            };

            if !keep_running {
                return WaitResult::Interrupted;
            }

            if retry_counter < SPIN_REPETITIONS {
                retry_counter += 1;
                spin_loop();
            } else if wait(&self.reader_count, &reader_count) == WaitAction::Abort {
                keep_running = false;
            }
        }
    }
}

pub struct RwLockWriterPreference {
    state: IoxAtomicU32,
    writer_wake_counter: IoxAtomicU32,
}

impl Default for RwLockWriterPreference {
    fn default() -> Self {
        Self::new()
    }
}

impl RwLockWriterPreference {
    pub const fn new() -> Self {
        Self {
            state: IoxAtomicU32::new(UNLOCKED),
            writer_wake_counter: IoxAtomicU32::new(0),
        }
    }

    pub fn try_read_lock(&self) -> WaitResult {
        let state = self.state.load(Ordering::Relaxed);
        if state % 2 == 1 {
            return WaitResult::Interrupted;
        }

        if self
            .state
            .compare_exchange(state, state + 2, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            WaitResult::Success
        } else {
            WaitResult::Interrupted
        }
    }

    pub fn read_lock<F: Fn(&IoxAtomicU32, &u32) -> WaitAction>(&self, wait: F) -> WaitResult {
        let mut state = self.state.load(Ordering::Relaxed);

        let mut retry_counter = 0;
        let mut keep_running = true;
        loop {
            if state % 2 == 1 {
                if !keep_running {
                    return WaitResult::Interrupted;
                }

                if retry_counter < SPIN_REPETITIONS {
                    retry_counter += 1;
                    spin_loop();
                } else if wait(&self.state, &state) == WaitAction::Abort {
                    keep_running = false;
                }
                state = self.state.load(Ordering::Relaxed);
            } else {
                match self.state.compare_exchange(
                    state,
                    state + 2,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return WaitResult::Success,
                    Err(v) => state = v,
                }
            }
        }
    }

    pub fn unlock<WakeOne: Fn(&IoxAtomicU32), WakeAll: Fn(&IoxAtomicU32)>(
        &self,
        wake_one: WakeOne,
        wake_all: WakeAll,
    ) {
        let state = self.state.load(Ordering::Relaxed);
        if state == WRITE_LOCKED {
            self.state.store(UNLOCKED, Ordering::Release);
            self.writer_wake_counter.fetch_add(1, Ordering::Relaxed);
            wake_one(&self.writer_wake_counter);
            wake_all(&self.state);
        } else if self.state.fetch_sub(2, Ordering::Relaxed) == 3 {
            self.writer_wake_counter.fetch_add(1, Ordering::Relaxed);
            wake_one(&self.writer_wake_counter);
        }
    }

    pub fn try_write_lock(&self) -> WaitResult {
        if self
            .state
            .compare_exchange(0, WRITE_LOCKED, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            WaitResult::Success
        } else {
            WaitResult::Interrupted
        }
    }

    pub fn write_lock<
        Wait: Fn(&IoxAtomicU32, &u32) -> WaitAction,
        WakeOne: Fn(&IoxAtomicU32),
        WakeAll: Fn(&IoxAtomicU32),
    >(
        &self,
        wait: Wait,
        wake_one: WakeOne,
        wake_all: WakeAll,
    ) -> WaitResult {
        let mut state = self.state.load(Ordering::Relaxed);

        let mut keep_running = true;
        let mut retry_counter = 0;
        loop {
            if state <= 1 {
                match self.state.compare_exchange(
                    state,
                    WRITE_LOCKED,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return WaitResult::Success,
                    Err(v) => state = v,
                }
            }

            if !keep_running {
                loop {
                    if state % 2 != 0 && state != WRITE_LOCKED {
                        match self.state.compare_exchange(
                            state,
                            state - 1,
                            Ordering::Relaxed,
                            Ordering::Relaxed,
                        ) {
                            Ok(_) => {
                                self.writer_wake_counter.fetch_add(1, Ordering::Relaxed);
                                wake_one(&self.writer_wake_counter);
                                wake_all(&self.state);
                                return WaitResult::Interrupted;
                            }
                            Err(v) => state = v,
                        }
                    } else {
                        return WaitResult::Interrupted;
                    }
                }
            }

            if state % 2 == 0 {
                let _ = self.state.compare_exchange(
                    state,
                    state + 1,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                );
            }

            if retry_counter < SPIN_REPETITIONS {
                spin_loop();
                retry_counter += 1;
            } else {
                let writer_wake_counter = self.writer_wake_counter.load(Ordering::Relaxed);
                if wait(&self.writer_wake_counter, &writer_wake_counter) == WaitAction::Abort {
                    keep_running = false;
                }
            }
            state = self.state.load(Ordering::Relaxed);
        }
    }
}
