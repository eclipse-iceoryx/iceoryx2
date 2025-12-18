// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_conformance_test_macros::conformance_test_module;

#[allow(clippy::module_inception)]
#[conformance_test_module]
pub mod writer {

    use iceoryx2::constants::MAX_BLACKBOARD_KEY_SIZE;
    use iceoryx2::port::writer::*;
    use iceoryx2::prelude::*;
    use iceoryx2::service::builder::blackboard::KeyMemory;
    use iceoryx2::service::builder::CustomKeyMarker;
    use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
    use iceoryx2::service::Service;
    use iceoryx2::testing::*;
    use iceoryx2_bb_concurrency::atomic::AtomicU64;
    use iceoryx2_bb_concurrency::atomic::Ordering;
    use iceoryx2_bb_conformance_test_macros::conformance_test;
    use iceoryx2_bb_posix::system_configuration::SystemInfo;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::watchdog::Watchdog;
    use std::sync::Barrier;

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "service_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[conformance_test]
    pub fn handle_can_be_acquired_for_existing_key_value_pair<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .create()
            .unwrap();

        let writer = sut.writer_builder().create().unwrap();
        let entry_handle_mut = writer.entry::<u64>(&0);
        assert_that!(entry_handle_mut, is_ok);
    }

    #[conformance_test]
    pub fn handle_cannot_be_acquired_for_non_existing_key<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .create()
            .unwrap();

        let writer = sut.writer_builder().create().unwrap();
        let entry_handle_mut = writer.entry::<u64>(&9);
        assert_that!(entry_handle_mut, is_err);
        assert_that!(
            entry_handle_mut.err().unwrap(),
            eq EntryHandleMutError::EntryDoesNotExist
        );
    }

    #[conformance_test]
    pub fn handle_cannot_be_acquired_for_wrong_value_type<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .create()
            .unwrap();

        let writer = sut.writer_builder().create().unwrap();
        let entry_handle_mut = writer.entry::<i64>(&0);
        assert_that!(entry_handle_mut, is_err);
        assert_that!(
            entry_handle_mut.err().unwrap(),
            eq EntryHandleMutError::EntryDoesNotExist
        );
    }

    #[conformance_test]
    pub fn entry_handle_mut_cannot_be_acquired_twice<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .create()
            .unwrap();

        let writer = sut.writer_builder().create().unwrap();
        let entry_handle_mut1 = writer.entry::<u64>(&0);
        assert_that!(entry_handle_mut1, is_ok);
        let entry_handle_mut2 = writer.entry::<u64>(&0);
        assert_that!(entry_handle_mut2, is_err);
        assert_that!(
            entry_handle_mut2.err().unwrap(),
            eq EntryHandleMutError::HandleAlreadyExists
        );

        drop(entry_handle_mut1);
        let entry_handle_mut2 = writer.entry::<u64>(&0);
        assert_that!(entry_handle_mut2, is_ok);
    }

    #[conformance_test]
    pub fn entry_handle_mut_prevents_another_writer<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<usize>()
            .add::<u8>(0, 0)
            .create()
            .unwrap();

        let writer = sut.writer_builder().create().unwrap();
        let _entry_handle_mut = writer.entry::<u8>(&0).unwrap();

        drop(writer);

        let res = sut.writer_builder().create();
        assert_that!(res, is_err);
        assert_that!(res.err().unwrap(), eq WriterCreateError::ExceedsMaxSupportedWriters);
    }

    #[conformance_test]
    pub fn entry_value_can_still_be_used_after_every_previous_service_state_owner_was_dropped<
        Sut: Service,
    >() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<usize>()
            .add::<u32>(0, 0)
            .create()
            .unwrap();

        let writer = sut.writer_builder().create().unwrap();
        let entry_handle_mut = writer.entry::<u32>(&0).unwrap();
        let entry_value_uninit = entry_handle_mut.loan_uninit();

        drop(writer);
        drop(sut);

        let _entry_handle_mut = entry_value_uninit.update_with_copy(333);
    }

    #[conformance_test]
    pub fn concurrent_writer_creation_succeeds_only_once<Sut: Service>() {
        let _watch_dog = Watchdog::new();
        let number_of_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 4);
        let barrier_start = Barrier::new(number_of_threads);
        let barrier_end = Barrier::new(number_of_threads);
        let counter = AtomicU64::new(0);

        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let _sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .create()
            .unwrap();

        std::thread::scope(|s| {
            let mut threads = vec![];
            for _ in 0..number_of_threads {
                threads.push(s.spawn(|| {
                    let sut = node
                        .service_builder(&service_name)
                        .blackboard_opener::<u64>()
                        .open()
                        .unwrap();
                    barrier_start.wait();
                    let writer = sut.writer_builder().create();
                    match writer {
                        Ok(_) => {
                            let _ = counter.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(e) => assert_that!(e, eq WriterCreateError::ExceedsMaxSupportedWriters),
                    }
                    barrier_end.wait();
                }));
            }
            for t in threads {
                t.join().unwrap();
            }
        });

        assert_that!(counter.load(Ordering::Relaxed), eq 1);
    }

    #[repr(C)]
    #[derive(ZeroCopySend)]
    struct Foo {
        a: u64,
        b: i64,
    }

    fn cmp_for_foo(lhs: *const u8, rhs: *const u8) -> bool {
        unsafe {
            (*lhs.cast::<Foo>()).a == (*rhs.cast::<Foo>()).a
                && (*lhs.cast::<Foo>()).b == (*rhs.cast::<Foo>()).b
        }
    }

    #[conformance_test]
    pub fn handle_can_be_acquired_for_existing_key_value_pair_with_custom_key_type<Sut: Service>() {
        type KeyType = Foo;
        let key = Foo {
            a: 28763,
            b: -62759340,
        };
        let key_ptr: *const KeyType = &key;
        type ValueType = u64;
        let default_value = ValueType::default();
        let value_ptr: *const ValueType = &default_value;

        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = unsafe {
            node.service_builder(&service_name)
                .blackboard_creator::<CustomKeyMarker>()
                .__internal_set_key_type_details(&TypeDetail::new::<KeyType>(
                    TypeVariant::FixedSize,
                ))
                .__internal_set_key_eq_cmp_func(Box::new(move |lhs, rhs| {
                    KeyMemory::<MAX_BLACKBOARD_KEY_SIZE>::key_eq_comparison(lhs, rhs, &cmp_for_foo)
                }))
                .__internal_add(
                    key_ptr as *const u8,
                    value_ptr as *mut u8,
                    TypeDetail::new::<ValueType>(TypeVariant::FixedSize),
                    Box::new(|| {}),
                )
                .create()
                .unwrap()
        };
        let writer = sut.writer_builder().create().unwrap();

        let type_details = TypeDetail::new::<ValueType>(TypeVariant::FixedSize);
        let entry_handle_mut =
            unsafe { writer.__internal_entry(key_ptr as *const u8, &type_details) };
        assert_that!(entry_handle_mut, is_ok);
    }

    #[conformance_test]
    pub fn handle_cannot_be_acquired_for_non_existing_key_with_custom_key_type<Sut: Service>() {
        type KeyType = Foo;
        let key = Foo { a: 8, b: 64 };
        let key_ptr: *const KeyType = &key;
        let invalid_key = Foo { a: 9, b: 9 };
        let invalid_key_ptr: *const KeyType = &invalid_key;
        type ValueType = u64;
        let default_value = ValueType::default();
        let value_ptr: *const ValueType = &default_value;

        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = unsafe {
            node.service_builder(&service_name)
                .blackboard_creator::<CustomKeyMarker>()
                .__internal_set_key_type_details(&TypeDetail::new::<KeyType>(
                    TypeVariant::FixedSize,
                ))
                .__internal_set_key_eq_cmp_func(Box::new(move |lhs, rhs| {
                    KeyMemory::<MAX_BLACKBOARD_KEY_SIZE>::key_eq_comparison(lhs, rhs, &cmp_for_foo)
                }))
                .__internal_add(
                    key_ptr as *const u8,
                    value_ptr as *mut u8,
                    TypeDetail::new::<ValueType>(TypeVariant::FixedSize),
                    Box::new(|| {}),
                )
                .create()
                .unwrap()
        };
        let writer = sut.writer_builder().create().unwrap();

        let type_details = TypeDetail::new::<ValueType>(TypeVariant::FixedSize);
        let entry_handle_mut =
            unsafe { writer.__internal_entry(invalid_key_ptr as *const u8, &type_details) };
        assert_that!(entry_handle_mut, is_err);
        assert_that!(
            entry_handle_mut.err().unwrap(),
            eq EntryHandleMutError::EntryDoesNotExist
        );
    }

    #[conformance_test]
    pub fn handle_cannot_be_acquired_for_wrong_value_type_with_custom_key_type<Sut: Service>() {
        type KeyType = Foo;
        let key = Foo { a: 1, b: -452 };
        let key_ptr: *const KeyType = &key;
        type ValueType = u64;
        let default_value = ValueType::default();
        let value_ptr: *const ValueType = &default_value;

        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = unsafe {
            node.service_builder(&service_name)
                .blackboard_creator::<CustomKeyMarker>()
                .__internal_set_key_type_details(&TypeDetail::new::<KeyType>(
                    TypeVariant::FixedSize,
                ))
                .__internal_set_key_eq_cmp_func(Box::new(move |lhs, rhs| {
                    KeyMemory::<MAX_BLACKBOARD_KEY_SIZE>::key_eq_comparison(lhs, rhs, &cmp_for_foo)
                }))
                .__internal_add(
                    key_ptr as *const u8,
                    value_ptr as *mut u8,
                    TypeDetail::new::<ValueType>(TypeVariant::FixedSize),
                    Box::new(|| {}),
                )
                .create()
                .unwrap()
        };
        let writer = sut.writer_builder().create().unwrap();

        let type_details = TypeDetail::new::<i64>(TypeVariant::FixedSize);
        let entry_handle_mut =
            unsafe { writer.__internal_entry(key_ptr as *const u8, &type_details) };
        assert_that!(entry_handle_mut, is_err);
        assert_that!(
            entry_handle_mut.err().unwrap(),
            eq EntryHandleMutError::EntryDoesNotExist
        );
    }

    #[conformance_test]
    pub fn entry_handle_mut_cannot_be_acquired_twice_with_custom_key_type<Sut: Service>() {
        type KeyType = Foo;
        let key = Foo { a: 23, b: 4 };
        let key_ptr: *const KeyType = &key;
        type ValueType = u64;
        let default_value = ValueType::default();
        let value_ptr: *const ValueType = &default_value;

        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = unsafe {
            node.service_builder(&service_name)
                .blackboard_creator::<CustomKeyMarker>()
                .__internal_set_key_type_details(&TypeDetail::new::<KeyType>(
                    TypeVariant::FixedSize,
                ))
                .__internal_set_key_eq_cmp_func(Box::new(move |lhs, rhs| {
                    KeyMemory::<MAX_BLACKBOARD_KEY_SIZE>::key_eq_comparison(lhs, rhs, &cmp_for_foo)
                }))
                .__internal_add(
                    key_ptr as *const u8,
                    value_ptr as *mut u8,
                    TypeDetail::new::<ValueType>(TypeVariant::FixedSize),
                    Box::new(|| {}),
                )
                .create()
                .unwrap()
        };
        let writer = sut.writer_builder().create().unwrap();

        let type_details = TypeDetail::new::<ValueType>(TypeVariant::FixedSize);
        let entry_handle_mut1 =
            unsafe { writer.__internal_entry(key_ptr as *const u8, &type_details) };
        assert_that!(entry_handle_mut1, is_ok);
        let entry_handle_mut2 =
            unsafe { writer.__internal_entry(key_ptr as *const u8, &type_details) };
        assert_that!(entry_handle_mut2, is_err);
        assert_that!(
            entry_handle_mut2.err().unwrap(),
            eq EntryHandleMutError::HandleAlreadyExists
        );

        drop(entry_handle_mut1);
        let entry_handle_mut2 =
            unsafe { writer.__internal_entry(key_ptr as *const u8, &type_details) };
        assert_that!(entry_handle_mut2, is_ok);
    }

    #[conformance_test]
    pub fn entry_handle_mut_prevents_another_writer_with_custom_key_type<Sut: Service>() {
        type KeyType = Foo;
        let key = Foo { a: 0, b: 0 };
        let key_ptr: *const KeyType = &key;
        type ValueType = u8;
        let default_value = ValueType::default();
        let value_ptr: *const ValueType = &default_value;

        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = unsafe {
            node.service_builder(&service_name)
                .blackboard_creator::<CustomKeyMarker>()
                .__internal_set_key_type_details(&TypeDetail::new::<KeyType>(
                    TypeVariant::FixedSize,
                ))
                .__internal_set_key_eq_cmp_func(Box::new(move |lhs, rhs| {
                    KeyMemory::<MAX_BLACKBOARD_KEY_SIZE>::key_eq_comparison(lhs, rhs, &cmp_for_foo)
                }))
                .__internal_add(
                    key_ptr as *const u8,
                    value_ptr as *mut u8,
                    TypeDetail::new::<ValueType>(TypeVariant::FixedSize),
                    Box::new(|| {}),
                )
                .create()
                .unwrap()
        };
        let writer = sut.writer_builder().create().unwrap();

        let type_details = TypeDetail::new::<ValueType>(TypeVariant::FixedSize);
        let _entry_handle_mut =
            unsafe { writer.__internal_entry(key_ptr as *const u8, &type_details) };

        drop(writer);

        let res = sut.writer_builder().create();
        assert_that!(res, is_err);
        assert_that!(res.err().unwrap(), eq WriterCreateError::ExceedsMaxSupportedWriters);
    }

    #[conformance_test]
    pub fn entry_value_can_still_be_used_after_every_previous_service_state_owner_was_dropped_with_custom_key_type<
        Sut: Service,
    >() {
        type KeyType = Foo;
        let key = Foo { a: 89, b: -98 };
        let key_ptr: *const KeyType = &key;
        type ValueType = u32;
        let default_value = ValueType::default();
        let value_ptr: *const ValueType = &default_value;

        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = unsafe {
            node.service_builder(&service_name)
                .blackboard_creator::<CustomKeyMarker>()
                .__internal_set_key_type_details(&TypeDetail::new::<KeyType>(
                    TypeVariant::FixedSize,
                ))
                .__internal_set_key_eq_cmp_func(Box::new(move |lhs, rhs| {
                    KeyMemory::<MAX_BLACKBOARD_KEY_SIZE>::key_eq_comparison(lhs, rhs, &cmp_for_foo)
                }))
                .__internal_add(
                    key_ptr as *const u8,
                    value_ptr as *mut u8,
                    TypeDetail::new::<ValueType>(TypeVariant::FixedSize),
                    Box::new(|| {}),
                )
                .create()
                .unwrap()
        };
        let writer = sut.writer_builder().create().unwrap();

        let type_details = TypeDetail::new::<ValueType>(TypeVariant::FixedSize);
        let entry_handle_mut = unsafe {
            writer
                .__internal_entry(key_ptr as *const u8, &type_details)
                .unwrap()
        };

        let entry_value_uninit =
            entry_handle_mut.loan_uninit(type_details.size(), type_details.alignment());

        drop(writer);
        drop(sut);

        let write_ptr = entry_value_uninit.write_cell();
        unsafe {
            *write_ptr = 8;
        }
        let _entry_handle_mut = entry_value_uninit.update();
    }
}
