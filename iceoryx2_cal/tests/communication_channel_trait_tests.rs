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
mod communication_channel {
    use iceoryx2_bb_container::semantic_string::*;
    use iceoryx2_bb_elementary::math::ToB64;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_system_types::file_name::FileName;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::communication_channel;
    use iceoryx2_cal::communication_channel::*;
    use iceoryx2_cal::named_concept::*;

    fn generate_name() -> FileName {
        let mut file = FileName::new(b"test_").unwrap();
        file.push_bytes(
            UniqueSystemId::new()
                .unwrap()
                .value()
                .to_b64()
                .to_lowercase()
                .as_bytes(),
        )
        .unwrap();
        file
    }

    #[test]
    fn names_are_set_correctly<Sut: CommunicationChannel<usize>>() {
        let storage_name = generate_name();

        let sut_receiver = Sut::Creator::new(&storage_name).create_receiver().unwrap();
        let sut_sender = Sut::Connector::new(&storage_name).open_sender().unwrap();

        assert_that!(*sut_receiver.name(), eq storage_name);
        assert_that!(*sut_sender.name(), eq storage_name);
    }

    #[test]
    fn buffer_size_is_by_default_at_least_provided_constant<Sut: CommunicationChannel<usize>>() {
        let storage_name = generate_name();

        let sut_receiver = Sut::Creator::new(&storage_name).create_receiver().unwrap();

        assert_that!(sut_receiver.buffer_size(), ge DEFAULT_RECEIVER_BUFFER_SIZE);
    }

    #[test]
    fn safe_overflow_is_disabled_by_default<Sut: CommunicationChannel<usize>>() {
        let storage_name = generate_name();

        let sut_receiver = Sut::Creator::new(&storage_name).create_receiver().unwrap();
        let sut_sender = Sut::Connector::new(&storage_name).open_sender().unwrap();

        assert_that!(!sut_receiver.does_enable_safe_overflow(), eq true);
        assert_that!(!sut_sender.does_enable_safe_overflow(), eq true);
    }

    #[test]
    fn create_remove_and_create_again_works<Sut: CommunicationChannel<usize>>() {
        let storage_name = generate_name();

        assert_that!(Sut::does_exist(&storage_name), eq Ok(false));
        let sut_receiver = Sut::Creator::new(&storage_name).create_receiver().unwrap();
        assert_that!(Sut::does_exist(&storage_name), eq Ok(true));

        drop(sut_receiver);
        assert_that!(Sut::does_exist(&storage_name), eq Ok(false));

        let sut_receiver = Sut::Creator::new(&storage_name).create_receiver();

        assert_that!(Sut::does_exist(&storage_name), eq Ok(true));
        assert_that!(sut_receiver, is_ok);
    }

    #[test]
    fn connecting_to_non_existing_channel_fails<Sut: CommunicationChannel<usize>>() {
        let storage_name = generate_name();

        let sut_sender = Sut::Connector::new(&storage_name).open_sender();
        assert_that!(sut_sender, is_err);
        assert_that!(
            sut_sender.err().unwrap(), eq
            CommunicationChannelOpenError::DoesNotExist
        );
    }

    #[test]
    fn connecting_to_receiver_works<Sut: CommunicationChannel<usize>>() {
        let storage_name = generate_name();

        let _sut_receiver = Sut::Creator::new(&storage_name).create_receiver().unwrap();
        let sut_sender = Sut::Connector::new(&storage_name).open_sender();

        assert_that!(sut_sender, is_ok);
    }

    #[test]
    fn connecting_after_first_connection_has_dropped_works<Sut: CommunicationChannel<usize>>() {
        let storage_name = generate_name();

        let _sut_receiver = Sut::Creator::new(&storage_name).create_receiver().unwrap();
        let sut_sender = Sut::Connector::new(&storage_name).open_sender().unwrap();
        drop(sut_sender);

        let sut_sender2 = Sut::Connector::new(&storage_name).open_sender();

        assert_that!(sut_sender2, is_ok);
    }

    #[test]
    fn send_and_receive_works_for_single_packets<Sut: CommunicationChannel<usize>>() {
        let storage_name = generate_name();

        let sut_receiver = Sut::Creator::new(&storage_name).create_receiver().unwrap();
        let sut_sender = Sut::Connector::new(&storage_name).open_sender().unwrap();

        const MAX_NUMBER_OF_PACKETS: usize = 16;

        for i in 0..MAX_NUMBER_OF_PACKETS {
            let data: usize = 12 * i;

            assert_that!(sut_sender.send(&data), is_ok);
            let received = sut_receiver.receive();
            assert_that!(received, is_ok);
            let received = received.unwrap();
            assert_that!(received, is_some);
            assert_that!(received.unwrap(), eq data);
        }
    }

    #[test]
    fn send_and_receive_for_multi_packets_has_queue_behavior<Sut: CommunicationChannel<usize>>() {
        let storage_name = generate_name();

        let sut_receiver = Sut::Creator::new(&storage_name)
            .buffer_size(4)
            .create_receiver()
            .unwrap();
        let sut_sender = Sut::Connector::new(&storage_name).open_sender().unwrap();

        const MAX_NUMBER_OF_PACKETS: usize = 1;

        for i in 0..MAX_NUMBER_OF_PACKETS {
            for k in 0..sut_receiver.buffer_size() {
                let data: usize = 12 * i + k;

                assert_that!(sut_sender.send(&data), is_ok);
            }

            for k in 0..sut_receiver.buffer_size() {
                let data: usize = 12 * i + k;

                let received = sut_receiver.receive();
                assert_that!(received, is_ok);
                let received = received.unwrap();
                assert_that!(received, is_some);
                assert_that!(received.unwrap(), eq data);
            }
        }
    }

    #[test]
    fn receive_without_transmission_returns_none<Sut: CommunicationChannel<usize>>() {
        let storage_name = generate_name();

        let sut_receiver = Sut::Creator::new(&storage_name).create_receiver().unwrap();
        let _sut_sender = Sut::Connector::new(&storage_name).open_sender().unwrap();

        let received = sut_receiver.receive();

        assert_that!(received, is_ok);
        assert_that!(received.unwrap(), is_none);
    }

    #[test]
    fn send_will_return_receiver_cache_full_when_cache_is_full<Sut: CommunicationChannel<usize>>() {
        let storage_name = generate_name();

        let sut_receiver = Sut::Creator::new(&storage_name)
            .buffer_size(4)
            .create_receiver()
            .unwrap();
        let sut_sender = Sut::Connector::new(&storage_name).open_sender().unwrap();

        let mut send_counter: usize = 0;
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

            if send_counter == sut_receiver.buffer_size() {
                break;
            }
        }

        let mut receive_counter: usize = 0;
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
        assert_that!(send_counter, ge sut_receiver.buffer_size());
    }

    #[test]
    fn safe_overflow_works<Sut: CommunicationChannel<usize>>() {
        if !Sut::does_support_safe_overflow() {
            return;
        }

        let storage_name = generate_name();

        let sut_receiver = Sut::Creator::new(&storage_name)
            .enable_safe_overflow()
            .create_receiver()
            .unwrap();
        let sut_sender = Sut::Connector::new(&storage_name).open_sender().unwrap();

        assert_that!(sut_sender.does_enable_safe_overflow(), eq true);
        assert_that!(sut_receiver.does_enable_safe_overflow(), eq true);

        for i in 0..sut_receiver.buffer_size() {
            assert_that!(sut_sender.send(&i), is_ok);
        }

        const NUMBER_OF_PACKETS: usize = 128;

        for i in sut_receiver.buffer_size()..NUMBER_OF_PACKETS {
            let data = sut_sender.send(&i).unwrap();

            assert_that!(data, is_some);
            assert_that!({ data.unwrap() }, eq i - sut_receiver.buffer_size());
        }
    }

    #[test]
    fn custom_buffer_size_works<Sut: CommunicationChannel<usize>>() {
        if !Sut::has_configurable_buffer_size() {
            return;
        }

        for buffer_size in 1..DEFAULT_RECEIVER_BUFFER_SIZE + 2 {
            let storage_name = generate_name();

            let sut_receiver = Sut::Creator::new(&storage_name)
                .buffer_size(buffer_size)
                .create_receiver()
                .unwrap();
            let sut_sender = Sut::Connector::new(&storage_name).open_sender().unwrap();

            assert_that!(sut_receiver.buffer_size(), ge buffer_size);

            for i in 0..buffer_size {
                assert_that!(sut_sender.send(&i), is_ok);
            }

            for i in 0..buffer_size {
                let data = sut_receiver.receive().unwrap();
                assert_that!(data, is_some);
                assert_that!(data.unwrap(), eq i);
            }
        }
    }

    #[test]
    fn custom_buffer_size_and_overflow_works<Sut: CommunicationChannel<usize>>() {
        if !Sut::has_configurable_buffer_size() || !Sut::does_support_safe_overflow() {
            return;
        }

        for buffer_size in 1..DEFAULT_RECEIVER_BUFFER_SIZE + 2 {
            let storage_name = generate_name();

            let sut_receiver = Sut::Creator::new(&storage_name)
                .buffer_size(buffer_size)
                .enable_safe_overflow()
                .create_receiver()
                .unwrap();
            let sut_sender = Sut::Connector::new(&storage_name).open_sender().unwrap();

            assert_that!(sut_receiver.buffer_size(), eq buffer_size);

            for i in 0..buffer_size {
                assert_that!(sut_sender.send(&i), is_ok);
            }

            for i in buffer_size..buffer_size * 2 {
                let result = sut_sender.send(&i).unwrap();
                assert_that!(result, is_some);
                assert_that!(result.unwrap(), eq i - buffer_size);
            }

            for i in 0..buffer_size {
                let data = sut_receiver.receive().unwrap();
                assert_that!(data, is_some);
                assert_that!(data.unwrap(), eq i + buffer_size);
            }
        }
    }

    #[test]
    fn list_channels_works<Sut: CommunicationChannel<usize>>() {
        let mut sut_names = vec![];
        let mut suts = vec![];
        const LIMIT: usize = 8;

        for i in 0..LIMIT {
            assert_that!(<Sut as NamedConceptMgmt>::list().unwrap(), len i);
            sut_names.push(generate_name());
            assert_that!(<Sut as NamedConceptMgmt>::does_exist(&sut_names[i]), eq Ok(false));
            suts.push(Sut::Creator::new(&sut_names[i]).create_receiver());
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

        std::mem::forget(suts);

        assert_that!(<Sut as NamedConceptMgmt>::list().unwrap(), len 0);
    }

    #[test]
    fn custom_suffix_keeps_channels_separated<Sut: CommunicationChannel<usize>>() {
        let config_1 = <Sut as NamedConceptMgmt>::Configuration::default()
            .suffix(unsafe { FileName::new_unchecked(b".s1") });
        let config_2 = <Sut as NamedConceptMgmt>::Configuration::default()
            .suffix(unsafe { FileName::new_unchecked(b".s2") });

        let sut_name = generate_name();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 0);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let sut_1 = Sut::Creator::new(&sut_name)
            .config(&config_1)
            .create_receiver()
            .unwrap();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let sut_2 = Sut::Creator::new(&sut_name)
            .config(&config_2)
            .create_receiver()
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
    fn defaults_for_configuration_are_set_correctly<Sut: CommunicationChannel<usize>>() {
        let config = <Sut as NamedConceptMgmt>::Configuration::default();
        assert_that!(*config.get_suffix(), eq DEFAULT_SUFFIX);
        assert_that!(*config.get_path_hint(), eq DEFAULT_PATH_HINT);
    }

    //#[cfg(not(any(target_os = "windows")))]
    #[instantiate_tests(<communication_channel::unix_datagram::Channel<usize>>)]
    mod unix_datagram {}

    #[instantiate_tests(<communication_channel::posix_shared_memory::Channel>)]
    mod posix_shared_memory {}

    #[instantiate_tests(<communication_channel::process_local::Channel>)]
    mod process_local {}

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    #[instantiate_tests(<communication_channel::message_queue::Channel<usize>>)]
    mod message_queue {}
}
