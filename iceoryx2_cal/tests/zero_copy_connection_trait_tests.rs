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

#[generic_tests::define]
mod zero_copy_connection {
    use std::time::{Duration, Instant};

    use iceoryx2_bb_container::semantic_string::*;
    use iceoryx2_bb_elementary::math::ToB64;
    use iceoryx2_bb_posix::barrier::{BarrierBuilder, BarrierHandle};
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_system_types::file_name::FileName;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::named_concept::*;
    use iceoryx2_cal::named_concept::{NamedConceptBuilder, NamedConceptMgmt};
    use iceoryx2_cal::shm_allocator::PointerOffset;
    use iceoryx2_cal::zero_copy_connection;
    use iceoryx2_cal::zero_copy_connection::*;

    const TIMEOUT: Duration = Duration::from_millis(25);

    fn generate_name() -> FileName {
        let mut file = FileName::new(b"test_").unwrap();
        file.push_bytes(UniqueSystemId::new().unwrap().value().to_b64().as_bytes())
            .unwrap();
        file
    }

    #[test]
    fn create_non_existing_connection_works<Sut: ZeroCopyConnection>() {
        let name = generate_name();

        assert_that!(Sut::Builder::new(&name).create_receiver(), is_ok);
        assert_that!(Sut::Builder::new(&name).create_sender(), is_ok);
    }

    #[test]
    fn establish_connection_works<Sut: ZeroCopyConnection>() {
        let name = generate_name();

        let sut_sender = Sut::Builder::new(&name).create_sender().unwrap();
        assert_that!(!sut_sender.is_connected(), eq true);

        let sut_receiver = Sut::Builder::new(&name).create_receiver().unwrap();
        assert_that!(sut_receiver.is_connected(), eq true);
        assert_that!(sut_sender.is_connected(), eq true);

        drop(sut_sender);
        assert_that!(!sut_receiver.is_connected(), eq true);
    }

    #[test]
    fn builder_sets_default_values<Sut: ZeroCopyConnection>() {
        let name = generate_name();

        let sut_sender = Sut::Builder::new(&name).create_sender().unwrap();
        assert_that!(sut_sender.buffer_size(), eq DEFAULT_BUFFER_SIZE);
        assert_that!(
            sut_sender.max_borrowed_samples(), eq
            DEFAULT_MAX_BORROWED_SAMPLES
        );
        assert_that!(
            sut_sender.has_enabled_safe_overflow(), eq
            DEFAULT_ENABLE_SAFE_OVERFLOW
        );

        let sut_receiver = Sut::Builder::new(&name).create_receiver().unwrap();
        assert_that!(sut_receiver.buffer_size(), eq DEFAULT_BUFFER_SIZE);
        assert_that!(
            sut_receiver.max_borrowed_samples(), eq
            DEFAULT_MAX_BORROWED_SAMPLES
        );
        assert_that!(
            sut_receiver.has_enabled_safe_overflow(), eq
            DEFAULT_ENABLE_SAFE_OVERFLOW
        );
    }

    #[test]
    fn multi_connections_fail<Sut: ZeroCopyConnection>() {
        let name = generate_name();

        let _sut_sender = Sut::Builder::new(&name).create_sender().unwrap();
        let _sut_receiver = Sut::Builder::new(&name).create_receiver().unwrap();

        let sut_sender = Sut::Builder::new(&name).create_sender();
        assert_that!(sut_sender, is_err);
        assert_that!(
            sut_sender.err().unwrap(), eq
            ZeroCopyCreationError::AnotherInstanceIsAlreadyConnected
        );

        let sut_receiver = Sut::Builder::new(&name).create_receiver();
        assert_that!(sut_receiver, is_err);
        assert_that!(
            sut_receiver.err().unwrap(), eq
            ZeroCopyCreationError::AnotherInstanceIsAlreadyConnected
        );
    }

    #[test]
    fn when_sender_goes_out_of_scope_another_sender_can_connect<Sut: ZeroCopyConnection>() {
        let name = generate_name();

        let _sut_sender = Sut::Builder::new(&name).create_sender().unwrap();
        let _sut_receiver = Sut::Builder::new(&name).create_receiver().unwrap();

        drop(_sut_sender);
        let sut_sender = Sut::Builder::new(&name).create_sender();
        assert_that!(sut_sender, is_ok);
    }

    #[test]
    fn when_receiver_goes_out_of_scope_another_receiver_can_connect<Sut: ZeroCopyConnection>() {
        let name = generate_name();

        let _sut_sender = Sut::Builder::new(&name).create_sender().unwrap();
        let _sut_receiver = Sut::Builder::new(&name).create_receiver().unwrap();

        drop(_sut_receiver);
        let sut_receiver = Sut::Builder::new(&name).create_receiver();
        assert_that!(sut_receiver, is_ok);
    }

    #[test]
    fn connection_is_cleaned_up_when_unused<Sut: ZeroCopyConnection + NamedConceptMgmt>() {
        let name = generate_name();

        let sut_sender = Sut::Builder::new(&name).create_sender().unwrap();
        let sut_receiver = Sut::Builder::new(&name).create_receiver().unwrap();

        drop(sut_sender);
        drop(sut_receiver);

        assert_that!(Sut::does_exist(&name), eq Ok(false));
    }

    #[test]
    fn connecting_with_incompatible_buffer_size_fails<Sut: ZeroCopyConnection>() {
        let name = generate_name();

        let _sut_sender = Sut::Builder::new(&name)
            .buffer_size(12)
            .create_sender()
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name).buffer_size(16).create_receiver();

        assert_that!(sut_receiver, is_err);
    }

    #[test]
    fn connecting_with_incompatible_borrow_max_fails<Sut: ZeroCopyConnection>() {
        let name = generate_name();

        let _sut_sender = Sut::Builder::new(&name)
            .receiver_max_borrowed_samples(2)
            .create_sender()
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .receiver_max_borrowed_samples(4)
            .create_receiver();

        assert_that!(sut_receiver, is_err);
    }

    #[test]
    fn connecting_with_incompatible_overflow_setting_fails<Sut: ZeroCopyConnection>() {
        let name = generate_name();

        let _sut_sender = Sut::Builder::new(&name)
            .enable_safe_overflow(true)
            .create_sender()
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name).create_receiver();

        assert_that!(sut_receiver, is_err);
        assert_that!(
            sut_receiver.err().unwrap(), eq
            ZeroCopyCreationError::IncompatibleOverflowSetting
        );
    }

    #[test]
    fn send_receive_and_retrieval_works<Sut: ZeroCopyConnection>() {
        let name = generate_name();

        let sut_sender = Sut::Builder::new(&name).create_sender().unwrap();
        let sut_receiver = Sut::Builder::new(&name).create_receiver().unwrap();

        assert_that!(sut_sender.try_send(PointerOffset::new(1237789)), is_ok);
        let sample = sut_receiver.receive().unwrap();
        assert_that!(sample, is_some);
        assert_that!(sample.as_ref().unwrap().value(), eq 1237789);

        assert_that!(sut_receiver.release(sample.unwrap()), is_ok);
        let retrieval = sut_sender.reclaim().unwrap();
        assert_that!(retrieval, is_some);
        assert_that!(retrieval.as_ref().unwrap().value(), eq 1237789);

        let retrieval = sut_sender.reclaim().unwrap();
        assert_that!(retrieval, is_none);
    }

    #[test]
    fn send_until_buffer_is_full_works<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        const BUFFER_SIZE: usize = 89;

        let sut_sender = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .create_sender()
            .unwrap();
        for i in 0..BUFFER_SIZE {
            assert_that!(sut_sender.try_send(PointerOffset::new(i)), is_ok);
        }
        let result = sut_sender.try_send(PointerOffset::new(9));
        assert_that!(result, is_err);
        assert_that!(result.err().unwrap(), eq ZeroCopySendError::ReceiveBufferFull);
    }

    #[test]
    fn send_until_overflow_works<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        const BUFFER_SIZE: usize = 56;

        let sut_sender = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .create_sender()
            .unwrap();
        for i in 0..BUFFER_SIZE {
            assert_that!(sut_sender.try_send(PointerOffset::new(i)), is_ok);
        }

        for i in 0..BUFFER_SIZE {
            let result = sut_sender.try_send(PointerOffset::new(9));
            assert_that!(result, is_ok);
            assert_that!(result.ok().unwrap().unwrap().value(), eq i);
        }
    }

    #[test]
    fn receive_can_acquire_data_with_late_connection<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        const BUFFER_SIZE: usize = 34;

        let sut_sender = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .create_sender()
            .unwrap();
        for i in 0..BUFFER_SIZE {
            assert_that!(sut_sender.try_send(PointerOffset::new(i)), is_ok);
        }

        let receiver = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .create_receiver()
            .unwrap();
        for i in 0..BUFFER_SIZE {
            let sample = receiver.receive();
            assert_that!(sample, is_ok);
            assert_that!(sample.ok().unwrap().unwrap().value(), eq i);
        }
    }

    #[test]
    fn new_connection_has_empty_receive_buffer<Sut: ZeroCopyConnection>() {
        let name = generate_name();

        let receiver = Sut::Builder::new(&name).create_receiver().unwrap();

        let sample = receiver.receive().unwrap();
        assert_that!(sample, is_none);
    }

    #[test]
    fn retrieve_channel_must_always_have_enough_space_left<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        const BUFFER_SIZE: usize = 34;
        const MAX_BORROWED_SAMPLES: usize = 34;

        let sut_sender = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples(MAX_BORROWED_SAMPLES)
            .create_sender()
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples(MAX_BORROWED_SAMPLES)
            .create_receiver()
            .unwrap();

        for i in 0..BUFFER_SIZE {
            assert_that!(sut_sender.try_send(PointerOffset::new(i)), is_ok);
        }

        for _ in 0..MAX_BORROWED_SAMPLES {
            let sample = sut_receiver.receive().unwrap().unwrap();
            assert_that!(sut_receiver.release(sample), is_ok);
        }

        assert_that!(sut_sender.try_send(PointerOffset::new(0)), is_ok);

        let result = sut_sender.try_send(PointerOffset::new(0));
        assert_that!(result, is_err);
        assert_that!(
            result.err().unwrap(), eq
            ZeroCopySendError::ClearRetrieveChannelBeforeSend
        );
    }

    #[test]
    fn receiver_cannot_borrow_more_samples_than_set_up<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        const BUFFER_SIZE: usize = 56;
        const MAX_BORROW: usize = 2;

        let sut_sender = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples(MAX_BORROW)
            .enable_safe_overflow(true)
            .create_sender()
            .unwrap();
        let sut_receiver = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples(MAX_BORROW)
            .enable_safe_overflow(true)
            .create_receiver()
            .unwrap();

        for _ in 0..2 {
            for i in 0..BUFFER_SIZE {
                assert_that!(sut_sender.try_send(PointerOffset::new(i)), is_ok);
            }

            let mut samples = vec![];
            for _ in 0..MAX_BORROW {
                let sample = sut_receiver.receive().unwrap();
                assert_that!(sample, is_some);
                samples.push(sample.unwrap());
            }

            let result = sut_receiver.receive();
            assert_that!(result, is_err);
            assert_that!(
                result.err().unwrap(), eq
                ZeroCopyReceiveError::ReceiveWouldExceedMaxBorrowValue
            );

            for s in samples {
                assert_that!(sut_receiver.release(s), is_ok);
                assert_that!(sut_sender.reclaim().unwrap(), is_some);
            }
        }
    }

    #[test]
    fn blocking_send_blocks<Sut: ZeroCopyConnection>() {
        let name = generate_name();

        let sut_sender = Sut::Builder::new(&name)
            .buffer_size(1)
            .create_sender()
            .unwrap();

        let handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new(2).create(&handle).unwrap();

        std::thread::scope(|s| {
            s.spawn(|| {
                let sut_receiver = Sut::Builder::new(&name)
                    .buffer_size(1)
                    .create_receiver()
                    .unwrap();

                barrier.wait();
                std::thread::sleep(TIMEOUT);
                let sample_1 = sut_receiver.receive();
                std::thread::sleep(TIMEOUT);
                let sample_2 = sut_receiver.receive();

                assert_that!(sample_1.unwrap().unwrap().value(), eq 7789);
                assert_that!(sample_2.unwrap().unwrap().value(), eq 227789);
            });

            barrier.wait();
            let now = Instant::now();

            assert_that!(sut_sender.blocking_send(PointerOffset::new(7789)), is_ok);
            assert_that!(sut_sender.blocking_send(PointerOffset::new(227789)), is_ok);
            assert_that!(now.elapsed(), time_at_least TIMEOUT);
        });
    }

    #[test]
    fn list_connections_works<Sut: ZeroCopyConnection>() {
        let mut sut_names = vec![];
        const LIMIT: usize = 8;

        for i in 0..LIMIT {
            assert_that!(<Sut as NamedConceptMgmt>::list().unwrap(), len i);
            sut_names.push(generate_name());
            assert_that!(<Sut as NamedConceptMgmt>::does_exist(&sut_names[i]), eq Ok(false));
            std::mem::forget(Sut::Builder::new(&sut_names[i]).create_sender().unwrap());
            assert_that!(<Sut as NamedConceptMgmt>::does_exist(&sut_names[i]), eq Ok(true));

            let list = <Sut as NamedConceptMgmt>::list().unwrap();
            assert_that!(<Sut as NamedConceptMgmt>::list().unwrap(), len i + 1);
            let does_exist_in_list = |value| {
                for e in &list {
                    if e == value {
                        return true;
                    }
                }
                false
            };

            for name in &sut_names {
                assert_that!(does_exist_in_list(name), eq true);
            }
        }

        assert_that!(<Sut as NamedConceptMgmt>::list().unwrap(), len LIMIT);

        for i in 0..LIMIT {
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove(&sut_names[i])}, eq Ok(true));
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove(&sut_names[i])}, eq Ok(false));
        }

        assert_that!(<Sut as NamedConceptMgmt>::list().unwrap(), len 0);
    }

    #[test]
    fn custom_suffix_keeps_connections_separated<Sut: ZeroCopyConnection>() {
        let config_1 = <Sut as NamedConceptMgmt>::Configuration::default()
            .suffix(unsafe { FileName::new_unchecked(b".s1") });
        let config_2 = <Sut as NamedConceptMgmt>::Configuration::default()
            .suffix(unsafe { FileName::new_unchecked(b".s2") });

        let sut_name = generate_name();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 0);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let sut_1 = Sut::Builder::new(&sut_name)
            .config(&config_1)
            .create_sender()
            .unwrap();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let sut_2 = Sut::Builder::new(&sut_name)
            .config(&config_2)
            .create_sender()
            .unwrap();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 1);

        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap()[0], eq sut_name);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap()[0], eq sut_name);

        assert_that!(*sut_1.name(), eq sut_name);
        assert_that!(*sut_2.name(), eq sut_name);

        std::mem::forget(sut_1);
        std::mem::forget(sut_2);

        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_1)}, eq Ok(true));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_1)}, eq Ok(false));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_2)}, eq Ok(true));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_2)}, eq Ok(false));
    }

    #[test]
    fn defaults_for_configuration_are_set_correctly<Sut: ZeroCopyConnection>() {
        let config = <Sut as NamedConceptMgmt>::Configuration::default();
        assert_that!(*config.get_suffix(), eq DEFAULT_SUFFIX);
        assert_that!(*config.get_path_hint(), eq DEFAULT_PATH_HINT);
    }

    #[instantiate_tests(<zero_copy_connection::posix_shared_memory::Connection>)]
    mod posix_shared_memory {}

    #[instantiate_tests(<zero_copy_connection::process_local::Connection>)]
    mod process_local {}
}
