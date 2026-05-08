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

use alloc::string::ToString;

use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_elementary_traits::zeroable::Zeroable;
use iceoryx2_bb_posix::clock::nanosleep;
use iceoryx2_bb_posix::config::*;
use iceoryx2_bb_posix::file::{File, FileBuilder};
use iceoryx2_bb_posix::file_descriptor::FileDescriptorManagement;
use iceoryx2_bb_posix::process::UniqueProcessId;
use iceoryx2_bb_posix::shared_memory::Permission;
use iceoryx2_bb_posix::testing::create_test_directory;
use iceoryx2_bb_posix::unix_datagram_socket::CreationMode;
use iceoryx2_bb_posix::{process_state::*, unique_system_id::UniqueSystemId};
use iceoryx2_bb_system_types::{file_name::FileName, file_path::FilePath};
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::leakable::Leakable;
use iceoryx2_bb_testing_macros::test;

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

    create_test_directory();
    FilePath::from_path_and_file(&TEST_DIRECTORY, &file).unwrap()
}

#[test]
pub fn guard_can_be_created() {
    create_test_directory();
    let path = generate_file_path();

    let guard = ProcessGuardBuilder::new().create(&path).unwrap();

    assert_that!(*guard.path(), eq path);
    assert_that!(File::does_exist(&path).unwrap(), eq true);
}

#[test]
pub fn guard_can_be_created_with_default() {
    create_test_directory();
    let path = generate_file_path();

    let guard = ProcessGuardBuilder::default().create(&path).unwrap();

    assert_that!(*guard.path(), eq path);
    assert_that!(File::does_exist(&path).unwrap(), eq true);
}

#[test]
pub fn guard_removes_file_when_dropped() {
    create_test_directory();
    let path = generate_file_path();

    let guard = ProcessGuardBuilder::new().create(&path).unwrap();
    assert_that!(File::does_exist(&path).unwrap(), eq true);
    drop(guard);
    assert_that!(File::does_exist(&path).unwrap(), eq false);
}

#[test]
pub fn guard_cannot_use_already_existing_file() {
    create_test_directory();
    let path = generate_file_path();

    let file = FileBuilder::new(&path)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let guard = ProcessGuardBuilder::new().create(&path);
    assert_that!(guard.is_err(), eq true);
    assert_that!(guard.err().unwrap(), eq ProcessGuardCreateError::AlreadyExists);

    file.remove_self().unwrap();
}

#[test]
pub fn monitor_returns_the_path_under_which_it_was_created() {
    create_test_directory();
    let path = generate_file_path();

    let monitor = ProcessMonitor::new(&path).unwrap();
    assert_that!(*monitor.path(), eq path);
}

#[test]
pub fn monitor_detects_dead_state() {
    create_test_directory();
    let path = generate_file_path();

    let guard = ProcessGuardBuilder::new().create(&path).unwrap();
    ProcessGuard::leak(guard);

    let monitor = ProcessMonitor::new(&path).unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::Dead);
    ProcessCleaner::new(&path).unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::DoesNotExist);
}

#[test]
pub fn monitor_detects_non_existing_state() {
    create_test_directory();
    let path = generate_file_path();

    let monitor = ProcessMonitor::new(&path).unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::DoesNotExist);
}

#[test]
pub fn monitor_transitions_work_starting_from_non_existing_process() {
    create_test_directory();
    let path = generate_file_path();
    let mut owner_lock_path = path;
    owner_lock_path.push_bytes(b"_owner_lock").unwrap();
    let mut context_path = path;
    context_path.push_bytes(b"_context").unwrap();

    let monitor = ProcessMonitor::new(&path).unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::DoesNotExist);

    let mut context_file = FileBuilder::new(&context_path)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();
    context_file
        .write_val(&UniqueProcessId::new_zeroed())
        .unwrap();
    let state_file = FileBuilder::new(&path)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();
    let owner_lock_file = FileBuilder::new(&owner_lock_path)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    assert_that!(monitor.state().unwrap(), eq ProcessState::Dead);
    owner_lock_file.remove_self().unwrap();
    assert_that!(monitor.state().err().unwrap(), eq ProcessMonitorStateError::CorruptedState);
    state_file.remove_self().unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::CleaningUp);
    context_file.remove_self().unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::DoesNotExist);
}

#[test]
pub fn monitor_transitions_work_starting_from_existing_process() {
    create_test_directory();
    let path = generate_file_path();
    let mut owner_lock_path = path;
    owner_lock_path.push_bytes(b"_owner_lock").unwrap();
    let mut context_path = path;
    context_path.push_bytes(b"_context").unwrap();

    let mut context_file = FileBuilder::new(&context_path)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();
    context_file
        .write_val(&UniqueProcessId::new_zeroed())
        .unwrap();
    let state_file = FileBuilder::new(&path)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();
    let owner_lock_file = FileBuilder::new(&owner_lock_path)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let monitor = ProcessMonitor::new(&path).unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::Dead);
    state_file.remove_self().unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::CleaningUp);
    owner_lock_file.remove_self().unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::CleaningUp);
    context_file.remove_self().unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::DoesNotExist);
}

#[test]
pub fn monitor_transitions_to_corrupted_state_works_for_existing_process() {
    create_test_directory();
    let path = generate_file_path();
    let mut owner_lock_path = path;
    owner_lock_path.push_bytes(b"_owner_lock").unwrap();
    let mut context_path = path;
    context_path.push_bytes(b"_context").unwrap();

    let mut context_file = FileBuilder::new(&context_path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();
    context_file
        .write_val(&UniqueProcessId::new_zeroed())
        .unwrap();
    let state_file = FileBuilder::new(&path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();
    let owner_lock_file = FileBuilder::new(&owner_lock_path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let monitor = ProcessMonitor::new(&path).unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::Dead);
    owner_lock_file.remove_self().unwrap();
    assert_that!(monitor.state().err(), eq Some(ProcessMonitorStateError::CorruptedState));
    state_file.remove_self().unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::CleaningUp);
    context_file.remove_self().unwrap();
    assert_that!(monitor.state().unwrap(), eq ProcessState::DoesNotExist);
}

#[test]
pub fn monitor_detects_initialized_state() {
    create_test_directory();
    let path = generate_file_path();
    let mut context_path = path;
    context_path.push_bytes(b"_context").unwrap();

    let mut file = FileBuilder::new(&context_path)
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
pub fn owner_lock_cannot_be_created_when_process_does_not_exist() {
    create_test_directory();
    let path = generate_file_path();
    let mut owner_lock_path = path;
    owner_lock_path.push_bytes(b"_owner_lock").unwrap();

    let owner_lock = ProcessCleaner::new(&path);
    assert_that!(owner_lock, is_err);
    assert_that!(
        owner_lock.err().unwrap(), eq
        ProcessCleanerCreateError::DoesNotExist
    );

    let state_file = FileBuilder::new(&path)
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
    drop(state_file);
    nanosleep(core::time::Duration::from_millis(100)).expect("failed to sleep");

    let mut owner_lock_file = FileBuilder::new(&owner_lock_path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();
    owner_lock_file
        .write_val(&UniqueProcessId::new_zeroed())
        .unwrap();

    let owner_lock = ProcessCleaner::new(&path);
    assert_that!(owner_lock, is_err);
    assert_that!(
        owner_lock.err().unwrap(), eq
        ProcessCleanerCreateError::DoesNotExist
    );
}

#[test]
pub fn cleaner_removes_state_files_on_drop() {
    create_test_directory();
    let path = generate_file_path();
    let mut owner_lock_path = path;
    owner_lock_path.push_bytes(b"_owner_lock").unwrap();
    let mut context_path = path;
    context_path.push_bytes(b"_context").unwrap();

    let mut context_file = FileBuilder::new(&context_path)
        .has_ownership(false)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();
    context_file
        .write_val(&UniqueProcessId::new_zeroed())
        .unwrap();

    let _state_file = FileBuilder::new(&path)
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
    assert_that!(File::does_exist(&context_path).unwrap(), eq false);
}

#[test]
pub fn cleaner_keeps_state_files_when_abandoned() {
    create_test_directory();
    let path = generate_file_path();
    let mut owner_lock_path = path;
    owner_lock_path.push_bytes(b"_owner_lock").unwrap();
    let mut context_path = path;
    context_path.push_bytes(b"_context").unwrap();

    let mut context_file = FileBuilder::new(&context_path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();
    context_file
        .write_val(&UniqueProcessId::new_zeroed())
        .unwrap();

    let _state_file = FileBuilder::new(&path)
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

#[test]
pub fn monitor_detects_alive_state_from_existing_process() {
    create_test_directory();
    let path = generate_file_path();

    let guard = ProcessGuardBuilder::new().create(&path).unwrap();
    let monitor = ProcessMonitor::new(&path).unwrap();

    assert_that!(monitor.state().unwrap(), eq ProcessState::Alive);
    drop(guard);
    assert_that!(monitor.state().unwrap(), eq ProcessState::DoesNotExist);
}

#[test]
pub fn owner_lock_cannot_be_acquired_from_living_process() {
    create_test_directory();
    let path = generate_file_path();

    let _guard = ProcessGuardBuilder::new().create(&path).unwrap();
    let owner_lock = ProcessCleaner::new(&path);
    assert_that!(owner_lock, is_err);
    assert_that!(
        owner_lock.err().unwrap(), eq
        ProcessCleanerCreateError::ProcessIsStillAlive
    );
}

#[test]
pub fn cleaner_cannot_be_created_with_a_process_state_currently_being_initialized() {
    create_test_directory();
    let path = generate_file_path();
    let mut context_path = path;
    context_path.push_bytes(b"_context").unwrap();

    let _context_file = FileBuilder::new(&context_path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .permission(Permission::OWNER_WRITE)
        .create()
        .unwrap();

    assert_that!(ProcessCleaner::new(&path).err(), eq Some(ProcessCleanerCreateError::ProcessIsInitializedOrCrashedDuringInitialization));
}

#[test]
pub fn cleaner_cannot_be_created_when_another_process_is_currently_cleaning_up() {
    create_test_directory();
    let path = generate_file_path();
    let mut owner_lock_path = path;
    owner_lock_path.push_bytes(b"_owner_lock").unwrap();
    let mut context_path = path;
    context_path.push_bytes(b"_context").unwrap();

    let mut context_file = FileBuilder::new(&context_path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();
    context_file
        .write_val(&UniqueProcessId::new_zeroed())
        .unwrap();

    let _owner_lock_file = FileBuilder::new(&owner_lock_path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    assert_that!(ProcessCleaner::new(&path).err(), eq Some(ProcessCleanerCreateError::ProcessIsBeingCleanedUpOrCrashedDuringCleanup));
}

#[test]
pub fn cleaner_cannot_acquire_a_corrupted_process() {
    create_test_directory();
    let path = generate_file_path();
    let mut context_path = path;
    context_path.push_bytes(b"_context").unwrap();

    let mut context_file = FileBuilder::new(&context_path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();
    context_file
        .write_val(&UniqueProcessId::new_zeroed())
        .unwrap();

    let _state_file = FileBuilder::new(&path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    assert_that!(ProcessCleaner::new(&path).err(), eq Some(ProcessCleanerCreateError::ProcessMonitorStateError(ProcessMonitorStateError::CorruptedState)));
}

// START: OS with IPC only lock detection
//
// the lock detection does work on some OS only in the inter process context.
// In the process local context the lock is not detected when the fcntl GETLK call is originating
// from the same thread os the fcntl SETLK call. If it is called from a different thread GETLK
// blocks despite it should be non-blocking.
#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos",
    target_os = "nto"
)))]
#[test]
pub fn owner_lock_cannot_be_acquired_twice() {
    create_test_directory();
    let path = generate_file_path();
    let mut owner_lock_path = path;
    owner_lock_path.push_bytes(b"_owner_lock").unwrap();
    let mut context_path = path;
    context_path.push_bytes(b"_context").unwrap();

    let mut context_file = FileBuilder::new(&context_path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();
    context_file
        .write_val(&UniqueProcessId::new_zeroed())
        .unwrap();

    let _state_file = FileBuilder::new(&path)
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
        ProcessCleanerCreateError::ProcessIsBeingCleanedUpOrCrashedDuringCleanup
    );
}

// END: OS with IPC only lock detection

#[test]
pub fn leaking_a_process_guard_results_in_a_dead_process() {
    create_test_directory();
    let path = generate_file_path();

    let guard = ProcessGuardBuilder::new().create(&path).unwrap();
    ProcessGuard::leak(guard);

    let state = ProcessMonitor::new(&path).unwrap().state().unwrap();
    assert_that!(state, eq ProcessState::Dead);

    ProcessCleaner::new(&path).unwrap();
}

#[test]
pub fn leaking_a_process_cleaner_allows_to_reacquire_the_cleaner() {
    create_test_directory();
    let path = generate_file_path();

    let guard = ProcessGuardBuilder::new().create(&path).unwrap();
    ProcessGuard::leak(guard);
    let cleaner = ProcessCleaner::new(&path).unwrap();
    ProcessCleaner::leak(cleaner);

    assert_that!(ProcessCleaner::new(&path), is_ok);
}
