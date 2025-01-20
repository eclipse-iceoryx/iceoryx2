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

#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

//! Abstraction of POSIX constructs with a safe API

use barrier::BarrierCreationError;
use clock::ClockError;
use directory::DirectoryError;
use file::FileError;
use file_lock::FileLockError;
use group::GroupError;
use iceoryx2_bb_elementary::enum_gen;
use memory_lock::MemoryLockError;
use mutex::MutexError;
use process::ProcessError;
use read_write_mutex::ReadWriteMutexError;
use semaphore::SemaphoreError;
use shared_memory::SharedMemoryCreationError;
use signal::SignalError;
use thread::ThreadError;
use unix_datagram_socket::UnixDatagramError;
use user::UserError;

pub mod access_mode;
pub mod adaptive_wait;
pub mod barrier;
pub mod clock;
pub mod config;
pub mod creation_mode;
pub mod socket_pair;
pub mod udp_socket;
#[macro_use]
pub mod handle_errno;
pub mod deadline_queue;
pub mod directory;
pub mod file;
pub mod file_descriptor;
pub mod file_descriptor_set;
pub mod file_lock;
pub mod file_type;
pub mod group;
pub mod ipc_capable;
pub mod memory;
pub mod memory_lock;
pub mod metadata;
pub mod mutex;
pub mod ownership;
pub mod permission;
pub mod process;
pub mod process_state;
pub mod read_write_mutex;
pub mod scheduler;
pub mod semaphore;
pub mod shared_memory;
pub mod signal;
pub mod socket_ancillary;
pub mod system_configuration;
#[doc(hidden)]
pub mod testing;
pub mod thread;
pub mod unique_system_id;
pub mod unix_datagram_socket;
pub mod user;

enum_gen! {Error
  generalization:
    Barrier <= BarrierCreationError,
    Clock <= ClockError,
    Directory <= DirectoryError,
    File <= FileError,
    FileLock <= FileLockError,
    Group <= GroupError,
    MemoryLock <= MemoryLockError,
    Mutex <= MutexError,
    Process <= ProcessError,
    ReadWriteMutex <= ReadWriteMutexError,
    Semaphore <= SemaphoreError,
    SharedMemory <= SharedMemoryCreationError,
    Signal <= SignalError,
    Thread <= ThreadError,
    User <= UserError,
    UnixDatagramSocket <= UnixDatagramError
}
