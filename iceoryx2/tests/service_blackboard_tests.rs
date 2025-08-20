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

#[generic_tests::define]
mod service_blackboard {
    use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use iceoryx2::port::reader::*;
    use iceoryx2::port::writer::*;
    use iceoryx2::prelude::*;
    use iceoryx2::service::builder::blackboard::{BlackboardCreateError, BlackboardOpenError};
    use iceoryx2::service::static_config::message_type_details::TypeVariant;
    use iceoryx2::service::Service;
    use iceoryx2::testing::*;
    use iceoryx2_bb_container::byte_string::FixedSizeByteString;
    use iceoryx2_bb_posix::system_configuration::SystemInfo;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::watchdog::Watchdog;
    use std::sync::Arc;
    use std::sync::Barrier;

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "service_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[test]
    fn open_with_attributes_fails_when_service_key_types_differ<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let defined_attr = AttributeSpecifier::new();
        let attr = AttributeVerifier::new();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .create_with_attributes(&defined_attr);
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard_opener::<i64>()
            .open_with_attributes(&attr);

        assert_that!(sut2, is_err);
    }

    #[test]
    fn creating_non_existing_service_works<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .create();

        assert_that!(sut, is_ok);
        let sut = sut.unwrap();
        assert_that!(*sut.name(), eq service_name);
    }

    #[test]
    fn creating_same_service_twice_fails<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(1, 0)
            .create();
        assert_that!(sut2, is_err);
        assert_that!(sut2.err().unwrap(), eq BlackboardCreateError::AlreadyExists);
    }

    #[test]
    fn create_fails_when_no_key_value_pairs_are_provided<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .create();
        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq BlackboardCreateError::NoEntriesProvided);
    }

    #[test]
    fn create_fails_when_the_same_key_is_provided_twice<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .add::<u8>(0, 0)
            .create();
        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq BlackboardCreateError::ServiceInCorruptedState);
    }

    #[test]
    fn create_works_with_mixed_add_methods<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .add_with_default::<u8>(1)
            .create();
        assert_that!(sut, is_ok);
    }

    #[test]
    fn create_fails_when_the_same_key_is_provided_twice_with_mixed_add_methods<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .add_with_default::<u8>(0)
            .create();
        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq BlackboardCreateError::ServiceInCorruptedState);
    }

    #[test]
    fn recreate_after_drop_works<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .create();
        assert_that!(sut, is_ok);

        drop(sut);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .create();
        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_does_not_exist<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open();
        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq BlackboardOpenError::DoesNotExist);
    }

    #[test]
    fn open_succeeds_when_service_does_exist<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open();
        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_has_wrong_key_type<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard_opener::<i64>()
            .open();
        assert_that!(sut2, is_err);
        assert_that!(sut2.err().unwrap(), eq BlackboardOpenError::IncompatibleKeys);
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_max_nodes_requirement<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .max_nodes(2)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .max_nodes(3)
            .open();

        assert_that!(sut2, is_err);
        assert_that!(sut2.err().unwrap(), eq BlackboardOpenError::DoesNotSupportRequestedAmountOfNodes);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .max_nodes(1)
            .open();

        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_max_readers_requirement<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .max_readers(2)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .max_readers(3)
            .open();

        assert_that!(sut2, is_err);
        assert_that!(
    sut2.err().unwrap(), eq BlackboardOpenError::DoesNotSupportRequestedAmountOfReaders);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .max_readers(1)
            .open();

        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_does_not_fail_when_service_owner_is_dropped<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open();
        assert_that!(sut2, is_ok);

        drop(sut);

        let sut3 = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open();
        assert_that!(sut3, is_ok);
    }

    #[test]
    fn open_fails_when_all_previous_owners_have_been_dropped<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open();
        assert_that!(sut2, is_ok);

        drop(sut);
        drop(sut2);

        let sut3 = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open();
        assert_that!(sut3, is_err);
        assert_that!(sut3.err().unwrap(), eq BlackboardOpenError::DoesNotExist);
    }

    #[test]
    fn max_readers_is_set_to_config_default<Sut: Service>() {
        let service_name = generate_name();
        let node = NodeBuilder::new().create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .create()
            .unwrap();

        let defaults = &Config::global_config().defaults;

        assert_that!(sut.static_config().max_readers(), eq defaults.blackboard.max_readers);
    }

    #[test]
    fn open_uses_predefined_settings_when_nothing_is_specified<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .max_nodes(89)
            .max_readers(4)
            .create()
            .unwrap();

        assert_that!(sut.static_config().max_nodes(), eq 89);
        assert_that!(sut.static_config().max_readers(), eq 4);

        let sut2 = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open()
            .unwrap();

        assert_that!(sut2.static_config().max_nodes(), eq 89);
        assert_that!(sut2.static_config().max_readers(), eq 4);
    }

    #[test]
    fn settings_can_be_modified_via_custom_config<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let mut custom_config = config.clone();
        custom_config.defaults.blackboard.max_nodes = 2;
        custom_config.defaults.blackboard.max_readers = 9;
        let node_1 = NodeBuilder::new()
            .config(&custom_config)
            .create::<Sut>()
            .unwrap();
        let node_2 = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node_1
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .create()
            .unwrap();

        assert_that!(sut.static_config().max_nodes(), eq 2);
        assert_that!(sut.static_config().max_readers(), eq 9);

        let sut2 = node_2
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open()
            .unwrap();

        // NOTE: although node_2 did specify a config with default values, since
        // node_1 was created first, the values of that node have to be preset
        assert_that!(sut2.static_config().max_nodes(), eq 2);
        assert_that!(sut2.static_config().max_readers(), eq 9);
    }

    #[test]
    fn type_information_are_correct<Sut: Service>() {
        type KeyType = u64;
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let service_name = generate_name();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<KeyType>()
            .add::<u8>(0, 0)
            .create()
            .unwrap();

        let d = sut.static_config().type_details();
        assert_that!(d.variant(), eq TypeVariant::FixedSize);
        assert_that!(*d.type_name(), eq core::any::type_name::<KeyType>());
        assert_that!(d.size(), eq core::mem::size_of::<KeyType>());
        assert_that!(d.alignment(), eq core::mem::align_of::<KeyType>());
    }

    #[test]
    fn number_of_readers_works<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        const MAX_READERS: usize = 8;

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .max_readers(MAX_READERS)
            .create()
            .unwrap();

        let sut2 = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open()
            .unwrap();

        let mut readers = vec![];

        for i in 0..MAX_READERS / 2 {
            readers.push(sut.reader_builder().create().unwrap());
            assert_that!(sut.dynamic_config().number_of_readers(), eq 2 * i + 1);
            assert_that!(sut2.dynamic_config().number_of_readers(), eq 2 * i + 1);
            assert_that!(sut.dynamic_config().number_of_writers(), eq 0);
            assert_that!(sut2.dynamic_config().number_of_writers(), eq 0);

            readers.push(sut2.reader_builder().create().unwrap());
            assert_that!(sut.dynamic_config().number_of_readers(), eq 2 * i + 2);
            assert_that!(sut2.dynamic_config().number_of_readers(), eq 2 * i + 2);
            assert_that!(sut.dynamic_config().number_of_writers(), eq 0);
            assert_that!(sut2.dynamic_config().number_of_writers(), eq 0);
        }

        for i in 0..MAX_READERS {
            readers.pop();
            assert_that!(sut.dynamic_config().number_of_readers(), eq MAX_READERS - i - 1);
            assert_that!(sut2.dynamic_config().number_of_readers(), eq MAX_READERS - i - 1);
        }
    }

    #[test]
    fn max_number_of_nodes_works<Sut: Service>() {
        let service_name = generate_name();
        const MAX_NODES: usize = 8;

        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .max_nodes(MAX_NODES)
            .create();
        assert_that!(sut, is_ok);

        let mut nodes = vec![];
        let mut services = vec![];

        nodes.push(node);
        services.push(sut.unwrap());

        for _ in 1..MAX_NODES {
            let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
            let sut = node
                .service_builder(&service_name)
                .blackboard_opener::<u64>()
                .open();
            assert_that!(sut, is_ok);

            nodes.push(node);
            services.push(sut.unwrap());
        }

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq BlackboardOpenError::ExceedsMaxNumberOfNodes);

        nodes.pop();
        services.pop();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open();

        assert_that!(sut, is_ok);
    }

    #[test]
    fn add_with_default_stores_default_value<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        const DEFAULT: u16 = 27;
        #[repr(C)]
        #[derive(Copy, Clone, ZeroCopySend)]
        struct TestDefault {
            t: u16,
        }
        impl Default for TestDefault {
            fn default() -> Self {
                TestDefault { t: DEFAULT }
            }
        }

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add_with_default::<TestDefault>(0)
            .create()
            .unwrap();

        let reader = sut.reader_builder().create().unwrap();
        let reader_handle = reader.entry::<TestDefault>(&0).unwrap();
        assert_that!(reader_handle.get().t, eq DEFAULT);
    }

    #[test]
    fn simple_communication_works_reader_created_first<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u16>(0, 0)
            .create()
            .unwrap();

        let reader = sut.reader_builder().create().unwrap();
        let reader_handle = reader.entry::<u16>(&0).unwrap();
        let writer = sut.writer_builder().create().unwrap();
        let writer_handle = writer.entry::<u16>(&0).unwrap();

        writer_handle.update_with_copy(1234);
        assert_that!(reader_handle.get(), eq 1234);

        writer_handle.update_with_copy(4567);
        assert_that!(reader_handle.get(), eq 4567);
    }

    #[test]
    fn simple_communication_works_writer_created_first<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<i32>(9, -3)
            .create()
            .unwrap();

        let writer = sut.writer_builder().create().unwrap();
        let writer_handle = writer.entry::<i32>(&9).unwrap();
        let reader = sut.reader_builder().create().unwrap();
        let reader_handle = reader.entry::<i32>(&9).unwrap();

        writer_handle.update_with_copy(50);
        assert_that!(reader_handle.get(), eq 50);

        writer_handle.update_with_copy(-12);
        assert_that!(reader_handle.get(), eq - 12);
    }

    #[test]
    fn communication_with_max_readers<Sut: Service>() {
        const MAX_READERS: usize = 6;
        const NUMBER_OF_ITERATIONS: u64 = 128;
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .max_readers(MAX_READERS)
            .create()
            .unwrap();

        let writer = sut.writer_builder().create().unwrap();
        let writer_handle = writer.entry::<u64>(&0).unwrap();

        let mut readers = vec![];
        readers.reserve(MAX_READERS);

        for _ in 0..MAX_READERS {
            readers.push(sut.reader_builder().create().unwrap());
        }

        for counter in 0..NUMBER_OF_ITERATIONS {
            writer_handle.update_with_copy(counter);

            for reader in &readers {
                let reader_handle = reader.entry::<u64>(&0).unwrap();
                assert_that!(reader_handle.get(), eq counter);
            }
        }
    }

    #[test]
    fn communication_with_max_reader_and_writer_handles<Sut: Service>() {
        const MAX_HANDLES: usize = 6;
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .add::<u64>(1, 1)
            .add::<u64>(2, 2)
            .add::<u64>(3, 3)
            .add::<u64>(4, 4)
            .add::<u64>(5, 5)
            .add::<u64>(6, 6)
            .max_readers(MAX_HANDLES)
            .create()
            .unwrap();

        let writer = sut.writer_builder().create().unwrap();
        let mut writer_handles = vec![];
        writer_handles.reserve(MAX_HANDLES);

        let reader = sut.reader_builder().create().unwrap();
        let mut reader_handles = vec![];
        reader_handles.reserve(MAX_HANDLES);

        for i in 0..MAX_HANDLES as u64 {
            writer_handles.push(writer.entry::<u64>(&i).unwrap());
            reader_handles.push(reader.entry::<u64>(&i).unwrap());
        }

        for i in 0..MAX_HANDLES {
            writer_handles[i].update_with_copy(7);
            for j in 0..(i + 1) {
                assert_that!(reader_handles[j].get(), eq 7);
            }
            for j in (i + 1)..MAX_HANDLES {
                assert_that!(reader_handles[j].get(), eq j as u64);
            }
        }
    }

    #[test]
    fn write_and_read_different_value_types_works<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        #[repr(C)]
        #[derive(Copy, Clone, ZeroCopySend, Debug, Eq, PartialEq)]
        struct Groovy {
            a: bool,
            b: u32,
            c: isize,
        }

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .add::<i8>(1, -5)
            .add::<FixedSizeByteString<4>>(23, "Nala".try_into().unwrap())
            .add::<bool>(100, false)
            .add::<Groovy>(
                13,
                Groovy {
                    a: true,
                    b: 7127,
                    c: 609,
                },
            )
            .create()
            .unwrap();

        let writer = sut.writer_builder().create().unwrap();
        writer
            .entry::<Groovy>(&13)
            .unwrap()
            .update_with_copy(Groovy {
                a: false,
                b: 888,
                c: 906,
            });
        writer.entry::<bool>(&100).unwrap().update_with_copy(true);
        writer
            .entry::<FixedSizeByteString<4>>(&23)
            .unwrap()
            .update_with_copy("Wolf".try_into().unwrap());
        writer.entry::<i8>(&1).unwrap().update_with_copy(11);
        writer.entry::<u64>(&0).unwrap().update_with_copy(2008);

        let reader = sut.reader_builder().create().unwrap();
        assert_that!(reader.entry::<u64>(&0).unwrap().get(), eq 2008);
        assert_that!(reader.entry::<i8>(&1).unwrap().get(), eq 11);
        assert_that!(reader.entry::<FixedSizeByteString<4>>(&23).unwrap().get(), eq "Wolf");
        assert_that!(reader.entry::<bool>(&100).unwrap().get(), eq true);
        assert_that!(reader.entry::<Groovy>(&13).unwrap().get(), eq Groovy{a: false, b: 888, c: 906});
    }

    #[test]
    fn creating_max_supported_amount_of_ports_work<Sut: Service>() {
        const MAX_READERS: usize = 8;

        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .max_readers(MAX_READERS)
            .create()
            .unwrap();

        let mut readers = vec![];

        // acquire all possible ports
        let writer = sut.writer_builder().create();
        assert_that!(writer, is_ok);

        for _ in 0..MAX_READERS {
            let reader = sut.reader_builder().create();
            assert_that!(reader, is_ok);
            readers.push(reader);
        }

        // create additional ports and fail
        let writer2 = sut.writer_builder().create();
        assert_that!(writer2, is_err);
        assert_that!(
            writer2.err().unwrap(), eq
            WriterCreateError::ExceedsMaxSupportedWriters
        );

        let reader = sut.reader_builder().create();
        assert_that!(reader, is_err);
        assert_that!(
            reader.err().unwrap(), eq
            ReaderCreateError::ExceedsMaxSupportedReaders
        );

        // remove one reader and the writer
        drop(writer);
        assert_that!(readers.remove(0), is_ok);

        // create additional ports shall work again
        let writer = sut.writer_builder().create();
        assert_that!(writer, is_ok);

        let reader = sut.reader_builder().create();
        assert_that!(reader, is_ok);
    }

    #[test]
    fn set_max_nodes_to_zero_adjusts_it_to_one<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .max_nodes(0)
            .create()
            .unwrap();

        assert_that!(sut.static_config().max_nodes(), eq 1);
    }

    #[test]
    fn set_max_readers_to_zero_adjusts_it_to_one<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .max_readers(0)
            .create()
            .unwrap();

        assert_that!(sut.static_config().max_readers(), eq 1);
    }

    #[test]
    fn does_exist_works_single<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        assert_that!(Sut::does_exist(&service_name, &config, MessagingPattern::Blackboard).unwrap(), eq false);

        let _sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .create()
            .unwrap();

        assert_that!(Sut::does_exist(&service_name, &config, MessagingPattern::Blackboard).unwrap(), eq true);
        assert_that!(Sut::does_exist(&service_name, &config, MessagingPattern::Blackboard).unwrap(), eq true);

        drop(_sut);

        assert_that!(Sut::does_exist(&service_name, &config, MessagingPattern::Blackboard).unwrap(), eq false);
    }

    #[test]
    fn does_exist_works_many<Sut: Service>() {
        const NUMBER_OF_SERVICES: usize = 8;

        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let mut services = vec![];
        let mut service_names = vec![];

        for i in 0..NUMBER_OF_SERVICES {
            let service_name = generate_name();
            assert_that!(Sut::does_exist(&service_name, &config, MessagingPattern::Blackboard).unwrap(), eq false);

            services.push(
                node.service_builder(&service_name)
                    .blackboard_creator::<u64>()
                    .add::<u8>(0, 0)
                    .create()
                    .unwrap(),
            );
            service_names.push(service_name);

            for s in service_names.iter().take(i + 1) {
                assert_that!(Sut::does_exist(s, &config, MessagingPattern::Blackboard).unwrap(), eq true);
            }
        }

        for i in 0..NUMBER_OF_SERVICES {
            for s in service_names.iter().take(NUMBER_OF_SERVICES - i) {
                assert_that!(Sut::does_exist(s, &config, MessagingPattern::Blackboard).unwrap(), eq true);
            }

            for s in service_names
                .iter()
                .take(NUMBER_OF_SERVICES)
                .skip(NUMBER_OF_SERVICES - i)
            {
                assert_that!(Sut::does_exist(s, &config, MessagingPattern::Blackboard).unwrap(), eq false);
            }

            services.pop();
        }
    }

    #[test]
    fn list_works<Sut: Service>() {
        const NUMBER_OF_SERVICES: usize = 8;

        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let mut services = vec![];
        let mut service_names = vec![];

        let contains_service_names = |names, state: Vec<ServiceDetails<Sut>>| {
            for n in names {
                let mut name_found = false;
                for s in &state {
                    if *s.static_details.name() == n {
                        name_found = true;
                        break;
                    }
                }

                if !name_found {
                    return false;
                }
            }

            true
        };

        for i in 0..NUMBER_OF_SERVICES {
            let service_name = generate_name();

            services.push(
                node.service_builder(&service_name)
                    .blackboard_creator::<u64>()
                    .add::<u8>(0, 0)
                    .create()
                    .unwrap(),
            );
            service_names.push(service_name);

            let mut service_list = vec![];
            Sut::list(&config, |s| {
                service_list.push(s);
                CallbackProgression::Continue
            })
            .unwrap();
            assert_that!(service_list, len i + 1);

            assert_that!(contains_service_names(service_names.clone(), service_list), eq true);
        }

        for i in 0..NUMBER_OF_SERVICES {
            services.pop();
            service_names.pop();

            let mut service_list = vec![];
            Sut::list(&config, |s| {
                service_list.push(s);
                CallbackProgression::Continue
            })
            .unwrap();
            assert_that!(service_list, len NUMBER_OF_SERVICES - i - 1);
            assert_that!(contains_service_names(service_names.clone(), service_list), eq true);
        }
    }

    #[test]
    fn dropping_service_keeps_established_communication<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u32>(0, 0)
            .create()
            .unwrap();

        let writer = sut.writer_builder().create().unwrap();
        let writer_handle = writer.entry(&0).unwrap();
        let reader = sut.reader_builder().create().unwrap();
        let reader_handle = reader.entry::<u32>(&0).unwrap();

        drop(sut);

        const PAYLOAD: u32 = 981293;

        writer_handle.update_with_copy(PAYLOAD);
        assert_that!(reader_handle.get(), eq PAYLOAD);
    }

    #[test]
    fn ports_of_dropped_service_block_new_service_creation<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .create()
            .unwrap();

        let reader = sut.reader_builder().create().unwrap();
        let writer = sut.writer_builder().create().unwrap();

        drop(sut);

        assert_that!(node.service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .create().err().unwrap(),
            eq BlackboardCreateError::AlreadyExists);

        drop(reader);

        assert_that!(node.service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .create().err().unwrap(),
            eq BlackboardCreateError::AlreadyExists);

        drop(writer);

        assert_that!(
            node.service_builder(&service_name)
                .blackboard_creator::<u64>()
                .add::<u8>(0, 0)
                .create(),
            is_ok
        );
    }

    #[test]
    fn service_can_be_opened_when_there_is_a_writer<Sut: Service>() {
        let payload = 1809723987;
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
        let writer = sut.writer_builder().create().unwrap();
        let writer_handle = writer.entry::<u64>(&0).unwrap();

        drop(sut);
        let sut = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open();
        assert_that!(sut, is_ok);
        drop(sut);
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .create();
        assert_that!(sut.err().unwrap(), eq BlackboardCreateError::AlreadyExists);
        drop(reader);

        let sut = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open()
            .unwrap();
        let reader = sut.reader_builder().create().unwrap();
        let reader_handle = reader.entry::<u64>(&0).unwrap();
        writer_handle.update_with_copy(payload);
        assert_that!(reader_handle.get(), eq payload);

        drop(reader_handle);
        drop(reader);
        drop(sut);
        drop(writer_handle);
        drop(writer);

        let sut = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open();
        assert_that!(sut.err().unwrap(), eq BlackboardOpenError::DoesNotExist);
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .create();
        assert_that!(sut, is_ok);
    }

    #[test]
    fn service_can_be_opened_when_there_is_a_reader<Sut: Service>() {
        let payload = 325183783;
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
        let reader_handle = reader.entry::<u64>(&0).unwrap();
        let writer = sut.writer_builder().create().unwrap();

        drop(sut);
        let sut = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open();
        assert_that!(sut, is_ok);
        drop(sut);
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .create();
        assert_that!(sut.err().unwrap(), eq BlackboardCreateError::AlreadyExists);
        drop(writer);

        let sut = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open()
            .unwrap();
        let writer = sut.writer_builder().create().unwrap();
        let writer_handle = writer.entry::<u64>(&0).unwrap();
        writer_handle.update_with_copy(payload);
        assert_that!(reader_handle.get(), eq payload);

        drop(writer_handle);
        drop(writer);
        drop(sut);
        drop(reader_handle);
        drop(reader);

        let sut = node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open();
        assert_that!(sut.err().unwrap(), eq BlackboardOpenError::DoesNotExist);
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .create();
        assert_that!(sut, is_ok);
    }

    #[test]
    fn open_error_display_works<Sut: Service>() {
        assert_that!(format!("{}", BlackboardOpenError::DoesNotExist), eq
    "BlackboardOpenError::DoesNotExist");
        assert_that!(format!("{}", BlackboardOpenError::InternalFailure), eq
    "BlackboardOpenError::InternalFailure");
        assert_that!(format!("{}", BlackboardOpenError::IncompatibleKeys), eq
    "BlackboardOpenError::IncompatibleKeys");
        assert_that!(format!("{}", BlackboardOpenError::IncompatibleMessagingPattern), eq
    "BlackboardOpenError::IncompatibleMessagingPattern");
        assert_that!(format!("{}", BlackboardOpenError::IncompatibleAttributes), eq
    "BlackboardOpenError::IncompatibleAttributes");
        assert_that!(format!("{}", BlackboardOpenError::DoesNotSupportRequestedAmountOfReaders), eq
    "BlackboardOpenError::DoesNotSupportRequestedAmountOfReaders");
        assert_that!(format!("{}", BlackboardOpenError::DoesNotSupportRequestedAmountOfNodes), eq
    "BlackboardOpenError::DoesNotSupportRequestedAmountOfNodes");
        assert_that!(format!("{}", BlackboardOpenError::InsufficientPermissions), eq
    "BlackboardOpenError::InsufficientPermissions");
        assert_that!(format!("{}", BlackboardOpenError::ServiceInCorruptedState), eq
    "BlackboardOpenError::ServiceInCorruptedState");
        assert_that!(format!("{}", BlackboardOpenError::HangsInCreation), eq
    "BlackboardOpenError::HangsInCreation");
        assert_that!(format!("{}", BlackboardOpenError::ExceedsMaxNumberOfNodes), eq
    "BlackboardOpenError::ExceedsMaxNumberOfNodes");
        assert_that!(format!("{}", BlackboardOpenError::IsMarkedForDestruction), eq
    "BlackboardOpenError::IsMarkedForDestruction");
    }

    #[test]
    fn create_error_display_works<Sut: Service>() {
        assert_that!(format!("{}", BlackboardCreateError::ServiceInCorruptedState), eq
    "BlackboardCreateError::ServiceInCorruptedState");
        assert_that!(format!("{}", BlackboardCreateError::HangsInCreation), eq
    "BlackboardCreateError::HangsInCreation");
        assert_that!(format!("{}", BlackboardCreateError::AlreadyExists), eq
    "BlackboardCreateError::AlreadyExists");
        assert_that!(format!("{}", BlackboardCreateError::InsufficientPermissions), eq
    "BlackboardCreateError::InsufficientPermissions");
        assert_that!(format!("{}", BlackboardCreateError::InternalFailure), eq
    "BlackboardCreateError::InternalFailure");
        assert_that!(format!("{}", BlackboardCreateError::IsBeingCreatedByAnotherInstance), eq
    "BlackboardCreateError::IsBeingCreatedByAnotherInstance");
        assert_that!(format!("{}", BlackboardCreateError::NoEntriesProvided), eq
    "BlackboardCreateError::NoEntriesProvided");
    }

    #[test]
    fn reader_can_still_read_payload_when_writer_was_disconnected<Sut: Service>() {
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
        let writer_handle = writer.entry::<u8>(&0).unwrap();
        writer_handle.update_with_copy(5);
        drop(writer_handle);
        drop(writer);

        let reader = sut.reader_builder().create().unwrap();
        let reader_handle = reader.entry::<u8>(&0).unwrap();
        assert_that!(reader_handle.get(), eq 5);
    }

    #[test]
    fn reconnected_reader_sees_current_blackboard_status<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<usize>()
            .add::<u8>(0, 0)
            .add::<i32>(6, -9)
            .create()
            .unwrap();

        let writer = sut.writer_builder().create().unwrap();
        let writer_handle = writer.entry::<u8>(&0).unwrap();
        writer_handle.update_with_copy(5);

        let reader = sut.reader_builder().create().unwrap();
        assert_that!(reader.entry::<u8>(&0).unwrap().get(), eq 5);
        assert_that!(reader.entry::<i32>(&6).unwrap().get(), eq - 9);

        drop(reader);

        let writer_handle = writer.entry::<i32>(&6).unwrap();
        writer_handle.update_with_copy(-567);

        let reader = sut.reader_builder().create().unwrap();
        assert_that!(reader.entry::<u8>(&0).unwrap().get(), eq 5);
        assert_that!(reader.entry::<i32>(&6).unwrap().get(), eq - 567);
    }

    #[test]
    fn writer_handle_can_still_write_after_writer_was_dropped<Sut: Service>() {
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
        let writer_handle = writer.entry::<u8>(&0).unwrap();

        drop(writer);
        writer_handle.update_with_copy(1);

        let reader = sut.reader_builder().create().unwrap();
        assert_that!(reader.entry::<u8>(&0).unwrap().get(), eq 1);
    }

    #[test]
    fn reader_handle_can_still_read_after_reader_was_dropped<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<usize>()
            .add::<u8>(0, 0)
            .create()
            .unwrap();

        let reader = sut.reader_builder().create().unwrap();
        let reader_handle = reader.entry::<u8>(&0).unwrap();

        drop(reader);
        assert_that!(reader_handle.get(), eq 0);

        let writer = sut.writer_builder().create().unwrap();
        let writer_handle = writer.entry::<u8>(&0).unwrap();
        writer_handle.update_with_copy(1);
        assert_that!(reader_handle.get(), eq 1);
    }

    #[test]
    fn loan_and_write_entry_value_works<Sut: Service>() {
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
        let writer_handle = writer.entry::<u32>(&0).unwrap();
        let reader = sut.reader_builder().create().unwrap();
        let reader_handle = reader.entry::<u32>(&0).unwrap();

        let entry_value_uninit = writer_handle.loan_uninit();
        let entry_value = entry_value_uninit.write(333);
        let _writer_handle = entry_value.update();

        assert_that!(reader_handle.get(), eq 333);
    }

    #[test]
    fn writer_handle_can_be_reused_after_entry_value_was_updated<Sut: Service>() {
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
        let writer_handle = writer.entry::<u32>(&0).unwrap();
        let reader = sut.reader_builder().create().unwrap();
        let reader_handle = reader.entry::<u32>(&0).unwrap();

        let entry_value_uninit = writer_handle.loan_uninit();
        let entry_value = entry_value_uninit.write(333);
        let writer_handle = entry_value.update();
        assert_that!(reader_handle.get(), eq 333);

        writer_handle.update_with_copy(999);
        assert_that!(reader_handle.get(), eq 999);
    }

    #[test]
    fn entry_value_can_still_be_used_after_writer_was_dropped<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<usize>()
            .add::<u32>(0, 0)
            .create()
            .unwrap();

        let reader = sut.reader_builder().create().unwrap();
        let reader_handle = reader.entry::<u32>(&0).unwrap();
        let writer = sut.writer_builder().create().unwrap();
        let writer_handle = writer.entry::<u32>(&0).unwrap();
        let entry_value_uninit = writer_handle.loan_uninit();

        drop(writer);

        let entry_value = entry_value_uninit.write(333);
        let _writer_handle = entry_value.update();
        assert_that!(reader_handle.get(), eq 333);
    }

    #[test]
    fn writer_handle_can_be_reused_after_entry_value_uninit_was_discarded<Sut: Service>() {
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
        let writer_handle = writer.entry::<u32>(&0).unwrap();
        let reader = sut.reader_builder().create().unwrap();
        let reader_handle = reader.entry::<u32>(&0).unwrap();

        let entry_value_uninit = writer_handle.loan_uninit();
        let writer_handle = entry_value_uninit.discard();
        writer_handle.update_with_copy(333);

        assert_that!(reader_handle.get(), eq 333);
    }

    #[test]
    fn writer_handle_can_be_reused_after_entry_value_was_discarded<Sut: Service>() {
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
        let writer_handle = writer.entry::<u32>(&0).unwrap();
        let reader = sut.reader_builder().create().unwrap();
        let reader_handle = reader.entry::<u32>(&0).unwrap();

        let entry_value_uninit = writer_handle.loan_uninit();
        let entry_value = entry_value_uninit.write(999);
        let writer_handle = entry_value.discard();
        writer_handle.update_with_copy(333);

        assert_that!(reader_handle.get(), eq 333);
    }

    #[test]
    fn handle_can_still_be_used_after_every_previous_service_state_owner_was_dropped<
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
        let writer_handle = writer.entry::<u32>(&0).unwrap();

        drop(writer);
        drop(sut);

        writer_handle.update_with_copy(3);
        drop(writer_handle);

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<usize>()
            .add::<u32>(0, 0)
            .create()
            .unwrap();

        let reader = sut.reader_builder().create().unwrap();
        let reader_handle = reader.entry::<u32>(&0).unwrap();

        drop(reader);
        drop(sut);

        assert_that!(reader_handle.get(), eq 0);
    }

    #[test]
    fn listing_all_readers_works<S: Service>() {
        const NUMBER_OF_READERS: usize = 18;
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .max_readers(NUMBER_OF_READERS)
            .create()
            .unwrap();

        let mut readers = vec![];

        for _ in 0..NUMBER_OF_READERS {
            readers.push(sut.reader_builder().create().unwrap());
        }

        let mut reader_details = vec![];
        sut.dynamic_config().list_readers(|details| {
            reader_details.push(details.reader_id);
            CallbackProgression::Continue
        });

        assert_that!(reader_details, len NUMBER_OF_READERS);
        for reader in readers {
            assert_that!(reader_details, contains reader.id());
        }
    }

    #[test]
    fn listing_all_readers_stops_on_request<S: Service>() {
        const NUMBER_OF_READERS: usize = 16;
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .max_readers(NUMBER_OF_READERS)
            .create()
            .unwrap();

        let mut readers = vec![];

        for _ in 0..NUMBER_OF_READERS {
            readers.push(sut.reader_builder().create().unwrap());
        }

        let mut counter = 0;
        sut.dynamic_config().list_readers(|_| {
            counter += 1;
            CallbackProgression::Stop
        });

        assert_that!(counter, eq 1);
    }

    #[test]
    fn concurrent_write_and_read_of_the_same_value_works<S: Service>() {
        let _watch_dog = Watchdog::new();
        let number_of_reader_handles = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 4);

        let barrier = Barrier::new(number_of_reader_handles + 1);
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();
        let _sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .create()
            .unwrap();

        let counter = AtomicU64::new(0);
        let keep_running = AtomicBool::new(true);

        std::thread::scope(|s| {
            let t = s.spawn(|| {
                let sut = node
                    .service_builder(&service_name)
                    .blackboard_opener::<u64>()
                    .open()
                    .unwrap();
                let writer = sut.writer_builder().create().unwrap();
                let writer_handle = writer.entry::<u64>(&0).unwrap();

                barrier.wait();

                while keep_running.load(Ordering::Relaxed) {
                    counter.fetch_add(1, Ordering::Relaxed);
                    writer_handle.update_with_copy(counter.load(Ordering::Relaxed));
                }
            });
            let mut threads = vec![];
            for _ in 0..number_of_reader_handles {
                threads.push(s.spawn(|| {
                    let sut = node
                        .service_builder(&service_name)
                        .blackboard_opener::<u64>()
                        .open()
                        .unwrap();
                    let reader = sut.reader_builder().create().unwrap();
                    barrier.wait();
                    let read_value = reader.entry::<u64>(&0).unwrap().get();
                    assert_that!(read_value, ge 0);
                    assert_that!(read_value, le counter.load(Ordering::Relaxed));
                }));
            }
            for t in threads {
                t.join().unwrap();
            }
            keep_running.store(false, Ordering::Relaxed);
            t.join().unwrap();
        });
    }

    #[test]
    fn concurrent_write_of_different_values_works<S: Service>() {
        let _watch_dog = Watchdog::new();
        let number_of_writer_handles: u64 = 8;

        let barrier = Arc::new(Barrier::new(number_of_writer_handles as usize));
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .add::<u64>(1, 0)
            .add::<u64>(2, 0)
            .add::<u64>(3, 0)
            .add::<u64>(4, 0)
            .add::<u64>(5, 0)
            .add::<u64>(6, 0)
            .add::<u64>(7, 0)
            .create()
            .unwrap();
        let writer = sut.writer_builder().create().unwrap();

        std::thread::scope(|s| {
            let mut threads = vec![];
            for i in 0..number_of_writer_handles {
                let writer_handle = writer.entry::<u64>(&i).unwrap();
                let barrier_thread = barrier.clone();
                threads.push(s.spawn(move || {
                    barrier_thread.wait();
                    writer_handle.update_with_copy(i);
                }));
            }
            for t in threads {
                t.join().unwrap();
            }
        });

        let reader = sut.reader_builder().create().unwrap();
        for i in 0..number_of_writer_handles {
            assert_that!(reader.entry::<u64>(&i).unwrap().get(), eq i);
        }
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
