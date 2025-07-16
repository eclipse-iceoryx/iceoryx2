// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use core::time::Duration;

use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_posix::config::*;
use iceoryx2_bb_posix::directory::Directory;
use iceoryx2_bb_posix::file::{File, FileBuilder};
use iceoryx2_bb_posix::file_descriptor::FileDescriptorManagement;
use iceoryx2_bb_posix::shared_memory::Permission;
use iceoryx2_bb_posix::testing::__internal_process_guard_staged_death;
use iceoryx2_bb_posix::testing::create_test_directory;
use iceoryx2_bb_posix::unix_datagram_socket::CreationMode;
use iceoryx2_bb_posix::{process_state::*, unique_system_id::UniqueSystemId};
use iceoryx2_bb_system_types::{file_name::FileName, file_path::FilePath};
use iceoryx2_bb_testing::assert_that;

fn generate_file_path() -> FilePath {
    let mut file = FileName::new(b"process_state_tests").unwrap();
    file.push_bytes(
        UniqueSystemId::new()
            .unwrap()
            .value()
            .to_string()
            .as_bytes(),
    )
    .unwrap();

    Directory::create(&test_directory(), Permission::OWNER_ALL).unwrap();
    FilePath::from_path_and_file(&test_directory(), &file).unwrap()
}

#[test]
pub fn process_state_guard_can_be_created() {
    create_test_directory();
    let path = generate_file_path();

    let guard = ProcessGuard::new(&path).unwrap();

    assert_that!(*guard.path(), eq path);
    assert_that!(File::does_exist(&path).unwrap(), eq true);
}

#[test]
pub fn process_state_guard_removes_file_when_dropped() {
    create_test_directory();
    let path = generate_file_path();

    let guard = ProcessGuard::new(&path).unwrap();
    assert_that!(File::does_exist(&path).unwrap(), eq true);
    drop(guard);
    assert_that!(File::does_exist(&path).unwrap(), eq false);
}

#[test]
pub fn process_state_guard_cannot_use_already_existing_file() {
    create_test_directory();
    let path = generate_file_path();

    let file = FileBuilder::new(&path)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let guard = ProcessGuard::new(&path);
    assert_that!(guard.is_err(), eq true);
    assert_that!(guard.err().unwrap(), eq ProcessGuardCreateError::AlreadyExists);

    file.remove_self().unwrap();
}

#[test]
pub fn process_state_monitor_detects_dead_state() {
    create_test_directory();
    let path = generate_file_path();
    let mut cleaner_path = path.clone();
    cleaner_path.push_bytes(b"_owner_lock").unwrap();

    let guard = ProcessGuard::new(&path).unwrap();
    __internal_process_guard_staged_death(guard);

    let monitor = ProcessMonitor::new(&path).unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::Dead);
    ProcessCleaner::new(&path).unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::DoesNotExist);
}

#[test]
pub fn process_state_monitor_detects_non_existing_state() {
    create_test_directory();
    let path = generate_file_path();

    let monitor = ProcessMonitor::new(&path).unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::DoesNotExist);
}

#[test]
pub fn process_state_monitor_transitions_work_starting_from_non_existing_process() {
    create_test_directory();
    let path = generate_file_path();
    let mut cleaner_path = path.clone();
    cleaner_path.push_bytes(b"_owner_lock").unwrap();

    let monitor = ProcessMonitor::new(&path).unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::DoesNotExist);
    let file = FileBuilder::new(&path)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let cleaner_file = FileBuilder::new(&cleaner_path)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    assert_that!(monitor.state().unwrap(), eq ProcessState::Dead);
    cleaner_file.remove_self().unwrap();
    assert_that!(monitor.state().err().unwrap(), eq ProcessMonitorStateError::CorruptedState);
    file.remove_self().unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::DoesNotExist);
}

#[test]
pub fn process_state_monitor_transitions_work_starting_from_existing_process() {
    create_test_directory();
    let path = generate_file_path();
    let mut owner_lock_path = path.clone();
    owner_lock_path.push_bytes(b"_owner_lock").unwrap();

    let file = FileBuilder::new(&path)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();
    let owner_lock_file = FileBuilder::new(&owner_lock_path)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let monitor = ProcessMonitor::new(&path).unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::Dead);
    file.remove_self().unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::DoesNotExist);

    let file = FileBuilder::new(&path)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::Dead);

    file.remove_self().unwrap();
    owner_lock_file.remove_self().unwrap();
}

#[test]
pub fn process_state_monitor_detects_initialized_state() {
    create_test_directory();
    let path = generate_file_path();

    let mut file = FileBuilder::new(&path)
        .creation_mode(CreationMode::PurgeAndCreate)
        .permission(Permission::OWNER_WRITE)
        .create()
        .unwrap();

    let monitor = ProcessMonitor::new(&path).unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::Starting);
    file.set_permission(Permission::OWNER_ALL).unwrap();
    file.remove_self().unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::DoesNotExist);
}

#[test]
pub fn process_state_owner_lock_cannot_be_created_when_process_does_not_exist() {
    create_test_directory();
    let path = generate_file_path();
    let mut owner_lock_path = path.clone();
    owner_lock_path.push_bytes(b"_owner_lock").unwrap();

    let owner_lock = ProcessCleaner::new(&path);
    assert_that!(owner_lock, is_err);
    assert_that!(
        owner_lock.err().unwrap(), eq
        ProcessCleanerCreateError::DoesNotExist
    );

    let file = FileBuilder::new(&path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let owner_lock = ProcessCleaner::new(&path);
    assert_that!(owner_lock, is_err);
    assert_that!(
        owner_lock.err().unwrap(), eq
        ProcessCleanerCreateError::DoesNotExist
    );
    drop(file);
    std::thread::sleep(Duration::from_millis(100));

    let _file = FileBuilder::new(&owner_lock_path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let owner_lock = ProcessCleaner::new(&path);
    assert_that!(owner_lock, is_err);
    assert_that!(
        owner_lock.err().unwrap(), eq
        ProcessCleanerCreateError::DoesNotExist
    );
}

#[test]
pub fn process_state_cleaner_removes_state_files_on_drop() {
    create_test_directory();
    let path = generate_file_path();
    let mut owner_lock_path = path.clone();
    owner_lock_path.push_bytes(b"_owner_lock").unwrap();

    let _file = FileBuilder::new(&path)
        .has_ownership(false)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let _owner_lock_file = FileBuilder::new(&owner_lock_path)
        .has_ownership(false)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let owner_lock = ProcessCleaner::new(&path);
    assert_that!(owner_lock, is_ok);

    drop(owner_lock);

    assert_that!(File::does_exist(&path).unwrap(), eq false);
    assert_that!(File::does_exist(&owner_lock_path).unwrap(), eq false);
}

#[test]
pub fn process_state_cleaner_keeps_state_files_when_abandoned() {
    create_test_directory();
    let path = generate_file_path();
    let mut owner_lock_path = path.clone();
    owner_lock_path.push_bytes(b"_owner_lock").unwrap();

    let _file = FileBuilder::new(&path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let _owner_lock_file = FileBuilder::new(&owner_lock_path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let owner_lock = ProcessCleaner::new(&path).unwrap();
    owner_lock.abandon();

    assert_that!(File::does_exist(&path).unwrap(), eq true);
    assert_that!(File::does_exist(&owner_lock_path).unwrap(), eq true);
}

// START: OS with IPC only lock detection
//
// the lock detection does work on some OS only in the inter process context.
// In the process local context the lock is not detected when the fcntl GETLK call is originating
// from the same thread os the fcntl SETLK call. If it is called from a different thread GETLK
// blocks despite it should be non-blocking.
#[test]
#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos",
    target_os = "nto"
)))]
pub fn process_state_monitor_detects_alive_state_from_existing_process() {
    create_test_directory();
    let path = generate_file_path();

    let guard = ProcessGuard::new(&path).unwrap();
    let monitor = ProcessMonitor::new(&path).unwrap();

    assert_that!(monitor.state().unwrap(), eq ProcessState::Alive);
    drop(guard);
    assert_that!(monitor.state().unwrap(), eq ProcessState::DoesNotExist);
}

#[test]
#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos",
    target_os = "nto"
)))]
pub fn process_state_owner_lock_cannot_be_acquired_from_living_process() {
    create_test_directory();
    let path = generate_file_path();

    let _guard = ProcessGuard::new(&path).unwrap();
    let owner_lock = ProcessCleaner::new(&path);
    assert_that!(owner_lock, is_err);
    assert_that!(
        owner_lock.err().unwrap(), eq
        ProcessCleanerCreateError::ProcessIsStillAlive
    );
}

#[test]
#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos",
    target_os = "nto"
)))]
pub fn process_state_owner_lock_cannot_be_acquired_twice() {
    create_test_directory();
    let path = generate_file_path();
    let mut owner_lock_path = path.clone();
    owner_lock_path.push_bytes(b"_owner_lock").unwrap();

    let _file = FileBuilder::new(&path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let _owner_lock_file = FileBuilder::new(&owner_lock_path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let _owner_lock = ProcessCleaner::new(&path).unwrap();
    let owner_lock = ProcessCleaner::new(&path);
    assert_that!(owner_lock, is_err);
    assert_that!(
        owner_lock.err().unwrap(), eq
        ProcessCleanerCreateError::OwnedByAnotherProcess
    );
}

// END: OS with IPC only lock detection
