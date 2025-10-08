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
//
#[generic_tests::define]
mod reader {
    use core::alloc::Layout;
    use iceoryx2::constants::MAX_BLACKBOARD_KEY_SIZE;
    use iceoryx2::port::reader::*;
    use iceoryx2::prelude::*;
    use iceoryx2::service::builder::blackboard::KeyMemory;
    use iceoryx2::service::builder::CustomKeyMarker;
    use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
    use iceoryx2::service::Service;
    use iceoryx2::testing::*;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;
    use std::collections::HashSet;

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "service_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[test]
    fn id_is_unique<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        const MAX_READERS: usize = 8;

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .max_readers(MAX_READERS)
            .add::<u64>(0, 0)
            .create()
            .unwrap();

        let mut readers = vec![];
        let mut reader_id_set = HashSet::new();

        for _ in 0..MAX_READERS {
            let reader = sut.reader_builder().create().unwrap();
            assert_that!(reader_id_set.insert(reader.id()), eq true);
            readers.push(reader);
        }
    }

    #[test]
    fn handle_can_be_acquired_for_existing_key_value_pair<Sut: Service>() {
        type ValueType = u64;
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<ValueType>(0, 0)
            .create()
            .unwrap();

        let reader = sut.reader_builder().create().unwrap();
        let entry_handle = reader.entry::<ValueType>(0);
        assert_that!(entry_handle, is_ok);
        assert_that!(entry_handle.unwrap().get(), eq 0);
    }

    #[test]
    fn handle_cannot_be_acquired_for_non_existing_key<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .create()
            .unwrap();

        let reader = sut.reader_builder().create().unwrap();
        let entry_handle = reader.entry::<u64>(9);
        assert_that!(entry_handle, is_err);
        assert_that!(
            entry_handle.err().unwrap(),
            eq EntryHandleError::EntryDoesNotExist
        );
    }

    #[test]
    fn handle_cannot_be_acquired_for_wrong_value_type<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .create()
            .unwrap();

        let reader = sut.reader_builder().create().unwrap();
        let entry_handle = reader.entry::<i64>(0);
        assert_that!(entry_handle, is_err);
        assert_that!(
            entry_handle.err().unwrap(),
            eq EntryHandleError::EntryDoesNotExist
        );
    }

    #[repr(C)]
    #[derive(ZeroCopySend)]
    struct Foo {
        a: i32,
        b: u32,
    }

    fn cmp_for_foo(lhs: *const u8, rhs: *const u8) -> bool {
        unsafe {
            (*lhs.cast::<Foo>()).a == (*rhs.cast::<Foo>()).a
                && (*lhs.cast::<Foo>()).b == (*rhs.cast::<Foo>()).b
        }
    }

    #[test]
    fn handle_can_be_acquired_for_existing_key_value_pair_with_custom_key_type<Sut: Service>() {
        type KeyType = Foo;
        let key = Foo { a: -13, b: 13 };
        let key_ptr: *const KeyType = &key;
        let key_layout =
            Layout::from_size_align(size_of::<KeyType>(), align_of::<KeyType>()).unwrap();
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
                    key_layout,
                    value_ptr as *mut u8,
                    TypeDetail::new::<ValueType>(TypeVariant::FixedSize),
                    Box::new(|| {}),
                )
                .create()
                .unwrap()
        };
        let reader = sut.reader_builder().create().unwrap();

        let type_details = TypeDetail::new::<ValueType>(TypeVariant::FixedSize);
        let entry_handle = reader.__internal_entry(key_ptr as *const u8, key_layout, &type_details);
        assert_that!(entry_handle, is_ok);
        let mut read_value: ValueType = 9;
        let read_value_ptr: *mut ValueType = &mut read_value;
        unsafe {
            entry_handle.unwrap().get(
                read_value_ptr as *mut u8,
                size_of::<ValueType>(),
                align_of::<ValueType>(),
            );
        }
        assert_that!(read_value, eq default_value);
    }

    #[test]
    fn handle_cannot_be_acquired_for_non_existing_key_with_custom_key_type<Sut: Service>() {
        type KeyType = Foo;
        let key = Foo { a: 9, b: 9 };
        let key_ptr: *const KeyType = &key;
        let invalid_key = Foo { a: -54, b: 534 };
        let invalid_key_ptr: *const KeyType = &invalid_key;
        let key_layout =
            Layout::from_size_align(size_of::<KeyType>(), align_of::<KeyType>()).unwrap();
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
                    key_layout,
                    value_ptr as *mut u8,
                    TypeDetail::new::<ValueType>(TypeVariant::FixedSize),
                    Box::new(|| {}),
                )
                .create()
                .unwrap()
        };
        let reader = sut.reader_builder().create().unwrap();

        let type_details = TypeDetail::new::<ValueType>(TypeVariant::FixedSize);
        let entry_handle =
            reader.__internal_entry(invalid_key_ptr as *const u8, key_layout, &type_details);
        assert_that!(entry_handle, is_err);
        assert_that!(
            entry_handle.err().unwrap(),
            eq EntryHandleError::EntryDoesNotExist
        );
    }

    #[test]
    fn handle_cannot_be_acquired_for_wrong_value_type_with_custom_key_type<Sut: Service>() {
        type KeyType = Foo;
        let key = Foo { a: 0, b: 0 };
        let key_ptr: *const KeyType = &key;
        let key_layout =
            Layout::from_size_align(size_of::<KeyType>(), align_of::<KeyType>()).unwrap();
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
                    key_layout,
                    value_ptr as *mut u8,
                    TypeDetail::new::<ValueType>(TypeVariant::FixedSize),
                    Box::new(|| {}),
                )
                .create()
                .unwrap()
        };
        let reader = sut.reader_builder().create().unwrap();

        let type_details = TypeDetail::new::<i64>(TypeVariant::FixedSize);
        let entry_handle = reader.__internal_entry(key_ptr as *const u8, key_layout, &type_details);
        assert_that!(entry_handle, is_err);
        assert_that!(
            entry_handle.err().unwrap(),
            eq EntryHandleError::EntryDoesNotExist
        );
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}

    #[instantiate_tests(<iceoryx2::service::ipc_threadsafe::Service>)]
    mod ipc_threadsafe {}

    #[instantiate_tests(<iceoryx2::service::local_threadsafe::Service>)]
    mod local_threadsafe {}
}
