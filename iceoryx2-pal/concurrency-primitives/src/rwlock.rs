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

use core::{
    hint::spin_loop,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::SPIN_REPETITIONS;

const WRITE_LOCKED: u32 = u32::MAX;
const UNLOCKED: u32 = 0;

pub struct RwLockReaderPreference {
    reader_count: AtomicU32,
}

impl Default for RwLockReaderPreference {
    fn default() -> Self {
        Self {
            reader_count: AtomicU32::new(UNLOCKED),
        }
    }
}

impl RwLockReaderPreference {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn try_read_lock(&self) -> bool {
        let reader_count = self.reader_count.load(Ordering::Relaxed);

        if reader_count == WRITE_LOCKED {
            return false;
        }

        self.reader_count
            .compare_exchange(
                reader_count,
                reader_count + 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_ok()
    }

    pub fn read_lock<F: Fn(&AtomicU32, &u32) -> bool>(&self, wait: F) -> bool {
        let mut reader_count = self.reader_count.load(Ordering::Relaxed);
        let mut retry_counter = 0;

        let mut keep_running = true;
        loop {
            loop {
                if reader_count != WRITE_LOCKED {
                    break;
                }

                if !keep_running {
                    return false;
                }

                if retry_counter < SPIN_REPETITIONS {
                    retry_counter += 1;
                    spin_loop();
                } else if !wait(&self.reader_count, &reader_count) {
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
                Ok(_) => return true,
                Err(v) => {
                    reader_count = v;
                }
            }
        }
    }

    pub fn unlock<WakeOne: Fn(&AtomicU32)>(&self, wake_one: WakeOne) {
        let state = self.reader_count.load(Ordering::Relaxed);
        if state == WRITE_LOCKED {
            self.reader_count.store(UNLOCKED, Ordering::Release);
        } else {
            self.reader_count.fetch_sub(1, Ordering::Relaxed);
        }
        wake_one(&self.reader_count);
    }

    pub fn try_write_lock(&self) -> bool {
        self.reader_count
            .compare_exchange(UNLOCKED, WRITE_LOCKED, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    pub fn write_lock<F: Fn(&AtomicU32, &u32) -> bool>(&self, wait: F) -> bool {
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
                Ok(_) => return true,
            };

            if !keep_running {
                return false;
            }

            if retry_counter < SPIN_REPETITIONS {
                retry_counter += 1;
                spin_loop();
            } else if !wait(&self.reader_count, &reader_count) {
                keep_running = false;
            }
        }
    }
}

pub struct RwLockWriterPreference {
    state: AtomicU32,
    writer_wake_counter: AtomicU32,
}

impl Default for RwLockWriterPreference {
    fn default() -> Self {
        Self {
            state: AtomicU32::new(UNLOCKED),
            writer_wake_counter: AtomicU32::new(0),
        }
    }
}

impl RwLockWriterPreference {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn try_read_lock(&self) -> bool {
        let state = self.state.load(Ordering::Relaxed);
        if state % 2 == 1 {
            return false;
        }

        self.state
            .compare_exchange(state, state + 2, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    pub fn read_lock<F: Fn(&AtomicU32, &u32) -> bool>(&self, wait: F) -> bool {
        let mut state = self.state.load(Ordering::Relaxed);

        let mut retry_counter = 0;
        let mut keep_running = true;
        loop {
            if state % 2 == 1 {
                if !keep_running {
                    return false;
                }

                if retry_counter < SPIN_REPETITIONS {
                    retry_counter += 1;
                    spin_loop();
                } else if !wait(&self.state, &state) {
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
                    Ok(_) => return true,
                    Err(v) => state = v,
                }
            }
        }
    }

    pub fn unlock<WakeOne: Fn(&AtomicU32), WakeAll: Fn(&AtomicU32)>(
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

    pub fn try_write_lock(&self) -> bool {
        self.state
            .compare_exchange(0, WRITE_LOCKED, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    pub fn write_lock<
        Wait: Fn(&AtomicU32, &u32) -> bool,
        WakeOne: Fn(&AtomicU32),
        WakeAll: Fn(&AtomicU32),
    >(
        &self,
        wait: Wait,
        wake_one: WakeOne,
        wake_all: WakeAll,
    ) -> bool {
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
                    Ok(_) => return true,
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
                                return false;
                            }
                            Err(v) => state = v,
                        }
                    } else {
                        return false;
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
                if !wait(&self.writer_wake_counter, &writer_wake_counter) {
                    keep_running = false;
                }
            }
            state = self.state.load(Ordering::Relaxed);
        }
    }
}
