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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod communication_channel_trait {
    use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
    use iceoryx2_bb_posix::testing::generate_file_path;
    use iceoryx2_bb_testing::{assert_that, test_requires};
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_cal::communication_channel::*;
    use iceoryx2_cal::named_concept::*;
    use iceoryx2_cal::testing::*;
    use iceoryx2_pal_posix::posix::POSIX_SUPPORT_PERSISTENT_SHARED_MEMORY;

    #[conformance_test]
    pub fn names_are_set_correctly<Sut: CommunicationChannel<u64>>() {
        let storage_name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Creator::new(&storage_name)
            .config(&config)
            .create_receiver()
            .unwrap();
        let sut_sender = Sut::Connector::new(&storage_name)
            .config(&config)
            .open_sender()
            .unwrap();

        assert_that!(*sut_receiver.name(), eq storage_name);
        assert_that!(*sut_sender.name(), eq storage_name);
    }

    #[conformance_test]
    pub fn buffer_size_is_by_default_at_least_provided_constant<Sut: CommunicationChannel<u64>>() {
        let storage_name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Creator::new(&storage_name)
            .config(&config)
            .create_receiver()
            .unwrap();

        assert_that!(sut_receiver.buffer_size(), ge DEFAULT_RECEIVER_BUFFER_SIZE);
    }

    #[conformance_test]
    pub fn safe_overflow_is_disabled_by_default<Sut: CommunicationChannel<u64>>() {
        let storage_name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Creator::new(&storage_name)
            .config(&config)
            .create_receiver()
            .unwrap();
        let sut_sender = Sut::Connector::new(&storage_name)
            .config(&config)
            .open_sender()
            .unwrap();

        assert_that!(!sut_receiver.does_enable_safe_overflow(), eq true);
        assert_that!(!sut_sender.does_enable_safe_overflow(), eq true);
    }

    #[conformance_test]
    pub fn create_remove_and_create_again_works<Sut: CommunicationChannel<u64>>() {
        let storage_name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        assert_that!(Sut::does_exist_cfg(&storage_name, &config), eq Ok(false));
        let sut_receiver = Sut::Creator::new(&storage_name)
            .config(&config)
            .create_receiver()
            .unwrap();
        assert_that!(Sut::does_exist_cfg(&storage_name, &config), eq Ok(true));

        drop(sut_receiver);
        assert_that!(Sut::does_exist_cfg(&storage_name, &config), eq Ok(false));

        let sut_receiver = Sut::Creator::new(&storage_name)
            .config(&config)
            .create_receiver();

        assert_that!(Sut::does_exist_cfg(&storage_name, &config), eq Ok(true));
        assert_that!(sut_receiver, is_ok);
    }

    #[conformance_test]
    pub fn connecting_to_non_existing_channel_fails<Sut: CommunicationChannel<u64>>() {
        let storage_name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Connector::new(&storage_name)
            .config(&config)
            .open_sender();
        assert_that!(sut_sender, is_err);
        assert_that!(
            sut_sender.err().unwrap(), eq
            CommunicationChannelOpenError::DoesNotExist
        );
    }

    #[conformance_test]
    pub fn connecting_to_receiver_works<Sut: CommunicationChannel<u64>>() {
        let storage_name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let _sut_receiver = Sut::Creator::new(&storage_name)
            .config(&config)
            .create_receiver()
            .unwrap();
        let sut_sender = Sut::Connector::new(&storage_name)
            .config(&config)
            .open_sender();

        assert_that!(sut_sender, is_ok);
    }

    #[conformance_test]
    pub fn connecting_after_first_connection_has_dropped_works<Sut: CommunicationChannel<u64>>() {
        let storage_name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let _sut_receiver = Sut::Creator::new(&storage_name)
            .config(&config)
            .create_receiver()
            .unwrap();
        let sut_sender = Sut::Connector::new(&storage_name)
            .config(&config)
            .open_sender()
            .unwrap();
        drop(sut_sender);

        let sut_sender2 = Sut::Connector::new(&storage_name)
            .config(&config)
            .open_sender();

        assert_that!(sut_sender2, is_ok);
    }

    #[conformance_test]
    pub fn send_and_receive_works_for_single_packets<Sut: CommunicationChannel<u64>>() {
        let storage_name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Creator::new(&storage_name)
            .config(&config)
            .create_receiver()
            .unwrap();
        let sut_sender = Sut::Connector::new(&storage_name)
            .config(&config)
            .open_sender()
            .unwrap();

        const MAX_NUMBER_OF_PACKETS: usize = 16;

        for i in 0..MAX_NUMBER_OF_PACKETS {
            let data: u64 = 12 * i as u64;

            assert_that!(sut_sender.send(&data), is_ok);
            let received = sut_receiver.receive();
            assert_that!(received, is_ok);
            let received = received.unwrap();
            assert_that!(received, is_some);
            assert_that!(received.unwrap(), eq data);
        }
    }

    #[conformance_test]
    pub fn send_and_receive_for_multi_packets_has_queue_behavior<Sut: CommunicationChannel<u64>>() {
        let storage_name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Creator::new(&storage_name)
            .buffer_size(4)
            .config(&config)
            .create_receiver()
            .unwrap();
        let sut_sender = Sut::Connector::new(&storage_name)
            .config(&config)
            .open_sender()
            .unwrap();

        const MAX_NUMBER_OF_PACKETS: usize = 1;

        for i in 0..MAX_NUMBER_OF_PACKETS {
            for k in 0..sut_receiver.buffer_size() {
                let data: u64 = (12 * i + k) as u64;

                assert_that!(sut_sender.send(&data), is_ok);
            }

            for k in 0..sut_receiver.buffer_size() {
                let data: u64 = (12 * i + k) as u64;

                let received = sut_receiver.receive();
                assert_that!(received, is_ok);
                let received = received.unwrap();
                assert_that!(received, is_some);
                assert_that!(received.unwrap(), eq data);
            }
        }
    }

    #[conformance_test]
    pub fn receive_without_transmission_returns_none<Sut: CommunicationChannel<u64>>() {
        let storage_name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Creator::new(&storage_name)
            .config(&config)
            .create_receiver()
            .unwrap();
        let _sut_sender = Sut::Connector::new(&storage_name)
            .config(&config)
            .open_sender()
            .unwrap();

        let received = sut_receiver.receive();

        assert_that!(received, is_ok);
        assert_that!(received.unwrap(), is_none);
    }

    #[conformance_test]
    pub fn send_will_return_receiver_cache_full_when_cache_is_full<
        Sut: CommunicationChannel<u64>,
    >() {
        let storage_name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Creator::new(&storage_name)
            .buffer_size(4)
            .config(&config)
            .create_receiver()
            .unwrap();
        let sut_sender = Sut::Connector::new(&storage_name)
            .config(&config)
            .open_sender()
            .unwrap();

        let mut send_counter: u64 = 0;
        loop {
            let result = sut_sender.send(&send_counter);
            if result.is_err() {
                assert_that!(
                    result.err().unwrap(), eq
                    CommunicationChannelSendError::ReceiverCacheIsFull
                );
                break;
            }

            send_counter += 1;

            if send_counter as usize == sut_receiver.buffer_size() {
                break;
            }
        }

        let mut receive_counter: u64 = 0;
        loop {
            let result = sut_receiver.receive();
            assert_that!(result, is_ok);

            if result.unwrap().is_none() {
                break;
            }
            assert_that!(result.unwrap().unwrap(), eq receive_counter);
            receive_counter += 1;
        }

        assert_that!(send_counter, eq receive_counter);
        assert_that!(send_counter, ge sut_receiver.buffer_size() as u64);
    }

    #[conformance_test]
    pub fn safe_overflow_works<Sut: CommunicationChannel<u64>>() {
        if !Sut::does_support_safe_overflow() {
            return;
        }

        let storage_name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Creator::new(&storage_name)
            .enable_safe_overflow()
            .config(&config)
            .create_receiver()
            .unwrap();
        let sut_sender = Sut::Connector::new(&storage_name)
            .config(&config)
            .open_sender()
            .unwrap();

        assert_that!(sut_sender.does_enable_safe_overflow(), eq true);
        assert_that!(sut_receiver.does_enable_safe_overflow(), eq true);

        for i in 0..sut_receiver.buffer_size() {
            assert_that!(sut_sender.send(&(i as u64)), is_ok);
        }

        const NUMBER_OF_PACKETS: usize = 128;

        for i in sut_receiver.buffer_size()..NUMBER_OF_PACKETS {
            let data = sut_sender.send(&(i as u64)).unwrap();

            assert_that!(data, is_some);
            assert_that!({ data.unwrap() as usize }, eq i - sut_receiver.buffer_size());
        }
    }

    #[conformance_test]
    pub fn custom_buffer_size_works<Sut: CommunicationChannel<u64>>() {
        if !Sut::has_configurable_buffer_size() {
            return;
        }
        let config = generate_isolated_config::<Sut>();

        for buffer_size in 1..DEFAULT_RECEIVER_BUFFER_SIZE + 2 {
            let storage_name = generate_file_path().file_name();

            let sut_receiver = Sut::Creator::new(&storage_name)
                .buffer_size(buffer_size)
                .config(&config)
                .create_receiver()
                .unwrap();
            let sut_sender = Sut::Connector::new(&storage_name)
                .config(&config)
                .open_sender()
                .unwrap();

            assert_that!(sut_receiver.buffer_size(), ge buffer_size);

            for i in 0..buffer_size {
                assert_that!(sut_sender.send(&(i as u64)), is_ok);
            }

            for i in 0..buffer_size {
                let data = sut_receiver.receive().unwrap();
                assert_that!(data, is_some);
                assert_that!(data.unwrap(), eq i as u64);
            }
        }
    }

    #[conformance_test]
    pub fn custom_buffer_size_and_overflow_works<Sut: CommunicationChannel<u64>>() {
        if !Sut::has_configurable_buffer_size() || !Sut::does_support_safe_overflow() {
            return;
        }
        let config = generate_isolated_config::<Sut>();

        for buffer_size in 1..DEFAULT_RECEIVER_BUFFER_SIZE + 2 {
            let storage_name = generate_file_path().file_name();

            let sut_receiver = Sut::Creator::new(&storage_name)
                .buffer_size(buffer_size)
                .enable_safe_overflow()
                .config(&config)
                .create_receiver()
                .unwrap();
            let sut_sender = Sut::Connector::new(&storage_name)
                .config(&config)
                .open_sender()
                .unwrap();

            assert_that!(sut_receiver.buffer_size(), eq buffer_size);

            for i in 0..buffer_size {
                assert_that!(sut_sender.send(&(i as u64)), is_ok);
            }

            for i in buffer_size..buffer_size * 2 {
                let result = sut_sender.send(&(i as u64)).unwrap();
                assert_that!(result, is_some);
                assert_that!(result.unwrap() as usize, eq i - buffer_size);
            }

            for i in 0..buffer_size {
                let data = sut_receiver.receive().unwrap();
                assert_that!(data, is_some);
                assert_that!(data.unwrap() as usize, eq i + buffer_size);
            }
        }
    }

    #[conformance_test]
    pub fn defaults_for_configuration_are_set_correctly<Sut: CommunicationChannel<u64>>() {
        let config = <Sut as NamedConceptMgmt>::Configuration::default();
        assert_that!(*config.get_suffix(), eq Sut::default_suffix());
    }

    #[conformance_test]
    pub fn abandon_receiver_keeps_channel_available<Sut: CommunicationChannel<u64>>() {
        test_requires!(POSIX_SUPPORT_PERSISTENT_SHARED_MEMORY);

        let storage_name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Creator::new(&storage_name)
            .config(&config)
            .create_receiver()
            .unwrap();

        Sut::Receiver::abandon(sut_receiver);

        assert_that!(Sut::does_exist_cfg(&storage_name, &config), eq Ok(true));
        assert_that!(unsafe { Sut::remove_cfg(&storage_name, &config).unwrap() }, eq true);
    }

    #[conformance_test]
    pub fn abandon_sender_keeps_channel_available<Sut: CommunicationChannel<u64>>() {
        let storage_name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Creator::new(&storage_name)
            .config(&config)
            .create_receiver()
            .unwrap();

        let sut_sender = Sut::Connector::new(&storage_name)
            .config(&config)
            .open_sender()
            .unwrap();

        Sut::Sender::abandon(sut_sender);
        assert_that!(Sut::does_exist_cfg(&storage_name, &config), eq Ok(true));

        let sut_sender = Sut::Connector::new(&storage_name)
            .config(&config)
            .open_sender()
            .unwrap();

        const MAX_NUMBER_OF_PACKETS: usize = 16;

        for i in 0..MAX_NUMBER_OF_PACKETS {
            let data: u64 = 12 * i as u64;

            assert_that!(sut_sender.send(&data), is_ok);
            let received = sut_receiver.receive();
            assert_that!(received, is_ok);
            let received = received.unwrap();
            assert_that!(received, is_some);
            assert_that!(received.unwrap(), eq data);
        }

        assert_that!(Sut::does_exist_cfg(&storage_name, &config), eq Ok(true));
    }
}
