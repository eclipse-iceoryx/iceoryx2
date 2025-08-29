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
mod writer {
    use core::sync::atomic::{AtomicU64, Ordering};
    use iceoryx2::port::writer::*;
    use iceoryx2::prelude::*;
    use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
    use iceoryx2::service::Service;
    use iceoryx2::testing::*;
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

    #[test]
    fn handle_can_be_acquired_for_existing_key_value_pair<Sut: Service>() {
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

        let writer = sut.writer_builder().create().unwrap();
        let entry_handle_mut = writer.entry::<u64>(&9);
        assert_that!(entry_handle_mut, is_err);
        assert_that!(
            entry_handle_mut.err().unwrap(),
            eq EntryHandleMutError::EntryDoesNotExist
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

        let writer = sut.writer_builder().create().unwrap();
        let entry_handle_mut = writer.entry::<i64>(&0);
        assert_that!(entry_handle_mut, is_err);
        assert_that!(
            entry_handle_mut.err().unwrap(),
            eq EntryHandleMutError::EntryDoesNotExist
        );
    }

    #[test]
    fn entry_handle_mut_cannot_be_acquired_twice<Sut: Service>() {
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

    #[test]
    fn entry_handle_mut_prevents_another_writer<Sut: Service>() {
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

    #[test]
    fn entry_value_can_still_be_used_after_every_previous_service_state_owner_was_dropped<
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

        let entry_value = entry_value_uninit.write(333);
        let _entry_handle_mut = entry_value.update();
    }

    #[test]
    fn concurrent_writer_creation_succeeds_only_once<Sut: Service>() {
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

    // TODO [#817] replace u64 with CustomKeyMarker
    #[test]
    fn handle_can_be_acquired_for_existing_key_value_pair_with_custom_key_type<Sut: Service>() {
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

        let type_details = TypeDetail::new::<u64>(TypeVariant::FixedSize);
        let entry_handle_mut = writer.__internal_entry(&0, &type_details);
        assert_that!(entry_handle_mut, is_ok);
    }

    // TODO [#817] replace u64 with CustomKeyMarker
    #[test]
    fn handle_cannot_be_acquired_for_non_existing_key_with_custom_key_type<Sut: Service>() {
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

        let type_details = TypeDetail::new::<u64>(TypeVariant::FixedSize);
        let entry_handle_mut = writer.__internal_entry(&9, &type_details);
        assert_that!(entry_handle_mut, is_err);
        assert_that!(
            entry_handle_mut.err().unwrap(),
            eq EntryHandleMutError::EntryDoesNotExist
        );
    }

    // TODO [#817] replace u64 with CustomKeyMarker
    #[test]
    fn handle_cannot_be_acquired_for_wrong_value_type_with_custom_key_type<Sut: Service>() {
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

        let type_details = TypeDetail::new::<i64>(TypeVariant::FixedSize);
        let entry_handle_mut = writer.__internal_entry(&0, &type_details);
        assert_that!(entry_handle_mut, is_err);
        assert_that!(
            entry_handle_mut.err().unwrap(),
            eq EntryHandleMutError::EntryDoesNotExist
        );
    }

    // TODO [#817] replace u64 with CustomKeyMarker
    #[test]
    fn entry_handle_mut_cannot_be_acquired_twice_with_custom_key_type<Sut: Service>() {
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

        let type_details = TypeDetail::new::<u64>(TypeVariant::FixedSize);
        let entry_handle_mut1 = writer.__internal_entry(&0, &type_details);
        assert_that!(entry_handle_mut1, is_ok);
        let entry_handle_mut2 = writer.__internal_entry(&0, &type_details);
        assert_that!(entry_handle_mut2, is_err);
        assert_that!(
            entry_handle_mut2.err().unwrap(),
            eq EntryHandleMutError::HandleAlreadyExists
        );

        drop(entry_handle_mut1);
        let entry_handle_mut2 = writer.__internal_entry(&0, &type_details);
        assert_that!(entry_handle_mut2, is_ok);
    }

    // TODO [#817] replace u64 with CustomKeyMarker
    #[test]
    fn entry_handle_mut_prevents_another_writer_with_custom_key_type<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u8>(0, 0)
            .create()
            .unwrap();
        let writer = sut.writer_builder().create().unwrap();

        let type_details = TypeDetail::new::<u8>(TypeVariant::FixedSize);
        let _entry_handle_mut = writer.__internal_entry(&0, &type_details);

        drop(writer);

        let res = sut.writer_builder().create();
        assert_that!(res, is_err);
        assert_that!(res.err().unwrap(), eq WriterCreateError::ExceedsMaxSupportedWriters);
    }

    // TODO [#817] replace u64 with CustomKeyMarker
    #[test]
    fn entry_value_can_still_be_used_after_every_previous_service_state_owner_was_dropped_with_custom_key_type<
        Sut: Service,
    >() {
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

        let type_details = TypeDetail::new::<u32>(TypeVariant::FixedSize);
        let entry_handle_mut = writer.__internal_entry(&0, &type_details).unwrap();

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

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}

    #[instantiate_tests(<iceoryx2::service::ipc_threadsafe::Service>)]
    mod ipc_threadsafe {}

    #[instantiate_tests(<iceoryx2::service::local_threadsafe::Service>)]
    mod local_threadsafe {}
}
