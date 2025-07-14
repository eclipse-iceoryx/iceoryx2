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
        let writer_handle = writer.entry::<u64>(&0);
        assert_that!(writer_handle, is_ok);
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
        let writer_handle = writer.entry::<u64>(&9);
        assert_that!(writer_handle, is_err);
        assert_that!(
            writer_handle.err().unwrap(),
            eq WriterHandleError::EntryDoesNotExist
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
        let writer_handle = writer.entry::<i64>(&0);
        assert_that!(writer_handle, is_err);
        assert_that!(
            writer_handle.err().unwrap(),
            eq WriterHandleError::EntryDoesNotExist
        );
    }

    #[test]
    fn writer_handle_cannot_be_acquired_twice<Sut: Service>() {
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
        let writer_handle1 = writer.entry::<u64>(&0);
        assert_that!(writer_handle1, is_ok);
        let writer_handle2 = writer.entry::<u64>(&0);
        assert_that!(writer_handle2, is_err);
        assert_that!(
            writer_handle2.err().unwrap(),
            eq WriterHandleError::HandleAlreadyExists
        );

        drop(writer_handle1);
        let writer_handle2 = writer.entry::<u64>(&0);
        assert_that!(writer_handle2, is_ok);
    }

    #[test]
    fn writer_handle_prevents_another_writer<Sut: Service>() {
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
        let _writer_handle = writer.entry::<u8>(&0).unwrap();

        drop(writer);

        let res = sut.writer_builder().create();
        assert_that!(res, is_err);
        assert_that!(res.err().unwrap(), eq WriterCreateError::ExceedsMaxSupportedWriters);
    }

    // TODO: compile test?
    //#[test]
    //fn writer_handle_cannot_loan_entry_value_twice<Sut: Service>() {
    //let service_name = generate_name();
    //let config = generate_isolated_config();
    //let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

    //let sut = node
    //.service_builder(&service_name)
    //.blackboard_creator::<usize>()
    //.add::<u32>(0, 0)
    //.create()
    //.unwrap();

    //let writer = sut.writer_builder().create().unwrap();
    //let writer_handle = writer.entry::<u32>(&0).unwrap();

    //let entry_value_uninit = writer_handle.loan_uninit();
    //let res = writer_handle.loan_uninit();
    ////assert_that!(res, is_err);
    ////assert_that!(res.err().unwrap(), eq WriterHandleError::HandleAlreadyLoansEntry);

    ////drop(entry_value);
    ////assert_that!(writer_handle.loan_uninit(), is_ok);
    //}

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
        let writer_handle = writer.entry::<u32>(&0).unwrap();
        let entry_value_uninit = writer_handle.loan_uninit();

        drop(writer);
        drop(sut);

        let entry_value = entry_value_uninit.write(333);
        let _writer_handle = entry_value.update();
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

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}

    #[instantiate_tests(<iceoryx2::service::ipc_threadsafe::Service>)]
    mod ipc_threadsafe {}

    #[instantiate_tests(<iceoryx2::service::local_threadsafe::Service>)]
    mod local_threadsafe {}
}
