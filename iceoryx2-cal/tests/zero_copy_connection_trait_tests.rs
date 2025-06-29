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
    use core::sync::atomic::Ordering;
    use core::time::Duration;
    use std::collections::HashSet;
    use std::sync::{Arc, Barrier, Mutex};
    use std::time::Instant;

    use iceoryx2_bb_container::semantic_string::*;
    use iceoryx2_bb_posix::barrier::*;
    use iceoryx2_bb_system_types::file_name::FileName;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::watchdog::Watchdog;
    use iceoryx2_cal::named_concept::*;
    use iceoryx2_cal::named_concept::{NamedConceptBuilder, NamedConceptMgmt};
    use iceoryx2_cal::shm_allocator::{PointerOffset, SegmentId};
    use iceoryx2_cal::testing::{generate_isolated_config, generate_name};
    use iceoryx2_cal::zero_copy_connection::{self};
    use iceoryx2_cal::zero_copy_connection::{ChannelId, *};

    const TIMEOUT: Duration = Duration::from_millis(25);
    const SAMPLE_SIZE: usize = 123;
    const NUMBER_OF_SAMPLES: usize = 2345;

    #[test]
    fn create_non_existing_connection_works<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        assert_that!(
            Sut::Builder::new(&name)
                .config(&config)
                .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
                .create_receiver(),
            is_ok
        );
        assert_that!(
            Sut::Builder::new(&name)
                .config(&config)
                .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
                .create_sender(),
            is_ok
        );
    }

    #[test]
    fn establish_connection_works<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender()
            .unwrap();
        assert_that!(!sut_sender.is_connected(), eq true);

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver()
            .unwrap();
        assert_that!(sut_receiver.is_connected(), eq true);
        assert_that!(sut_sender.is_connected(), eq true);

        drop(sut_sender);
        assert_that!(!sut_receiver.is_connected(), eq true);
    }

    #[test]
    fn builder_sets_default_values<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .config(&config)
            .create_sender()
            .unwrap();
        assert_that!(sut_sender.buffer_size(), eq DEFAULT_BUFFER_SIZE);
        assert_that!(
            sut_sender.max_borrowed_samples(), eq
            DEFAULT_MAX_BORROWED_SAMPLES_PER_CHANNEL
        );
        assert_that!(
            sut_sender.has_enabled_safe_overflow(), eq
            DEFAULT_ENABLE_SAFE_OVERFLOW
        );
        assert_that!(sut_sender.number_of_channels(), eq DEFAULT_NUMBER_OF_CHANNELS);

        let sut_receiver = Sut::Builder::new(&name)
            .config(&config)
            .create_receiver()
            .unwrap();
        assert_that!(sut_receiver.buffer_size(), eq DEFAULT_BUFFER_SIZE);
        assert_that!(
            sut_receiver.max_borrowed_samples(), eq
            DEFAULT_MAX_BORROWED_SAMPLES_PER_CHANNEL
        );
        assert_that!(
            sut_receiver.has_enabled_safe_overflow(), eq
            DEFAULT_ENABLE_SAFE_OVERFLOW
        );
        assert_that!(sut_receiver.number_of_channels(), eq DEFAULT_NUMBER_OF_CHANNELS);
    }

    #[test]
    fn setting_number_of_channels_works<Sut: ZeroCopyConnection>() {
        const NUMBER_OF_CHANNELS: usize = 12;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .config(&config)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .create_sender()
            .unwrap();
        assert_that!(sut_sender.number_of_channels(), eq NUMBER_OF_CHANNELS);

        let sut_receiver = Sut::Builder::new(&name)
            .config(&config)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .create_receiver()
            .unwrap();
        assert_that!(sut_receiver.number_of_channels(), eq NUMBER_OF_CHANNELS);
    }

    #[test]
    fn multi_connections_fail<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let _sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender()
            .unwrap();
        let _sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver()
            .unwrap();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender();
        assert_that!(sut_sender, is_err);
        assert_that!(
            sut_sender.err().unwrap(), eq
            ZeroCopyCreationError::AnotherInstanceIsAlreadyConnected
        );

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver();
        assert_that!(sut_receiver, is_err);
        assert_that!(
            sut_receiver.err().unwrap(), eq
            ZeroCopyCreationError::AnotherInstanceIsAlreadyConnected
        );
    }

    #[test]
    fn when_sender_goes_out_of_scope_another_sender_can_connect<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let _sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender()
            .unwrap();
        let _sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver()
            .unwrap();

        drop(_sut_sender);
        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender();
        assert_that!(sut_sender, is_ok);
    }

    #[test]
    fn when_receiver_goes_out_of_scope_another_receiver_can_connect<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let _sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender()
            .unwrap();
        let _sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver()
            .unwrap();

        drop(_sut_receiver);
        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver();
        assert_that!(sut_receiver, is_ok);
    }

    #[test]
    fn connection_is_cleaned_up_when_unused<Sut: ZeroCopyConnection + NamedConceptMgmt>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender()
            .unwrap();
        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver()
            .unwrap();

        drop(sut_sender);
        drop(sut_receiver);

        assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(false));
    }

    #[test]
    fn connecting_with_incompatible_buffer_size_fails<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let _sut_sender = Sut::Builder::new(&name)
            .buffer_size(12)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender()
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .buffer_size(16)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver();

        assert_that!(sut_receiver, is_err);
    }

    #[test]
    fn connecting_with_incompatible_borrow_max_fails<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let _sut_sender = Sut::Builder::new(&name)
            .receiver_max_borrowed_samples_per_channel(2)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender()
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .receiver_max_borrowed_samples_per_channel(4)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver();

        assert_that!(sut_receiver, is_err);
    }

    #[test]
    fn connecting_with_incompatible_overflow_setting_fails<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let _sut_sender = Sut::Builder::new(&name)
            .enable_safe_overflow(true)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender()
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver();

        assert_that!(sut_receiver, is_err);
        assert_that!(
            sut_receiver.err().unwrap(), eq
            ZeroCopyCreationError::IncompatibleOverflowSetting
        );
    }

    #[test]
    fn connecting_with_incompatible_number_of_samples_fails<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let _sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender()
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES * 2)
            .config(&config)
            .create_receiver();

        assert_that!(sut_receiver, is_err);
    }

    #[test]
    fn connecting_with_incompatible_number_of_channels_fails<Sut: ZeroCopyConnection>() {
        const NUMBER_OF_CHANNELS: usize = 5;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let _sut_sender = Sut::Builder::new(&name)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .config(&config)
            .create_sender()
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_channels(NUMBER_OF_CHANNELS + 1)
            .config(&config)
            .create_receiver();

        assert_that!(sut_receiver.err(), eq Some(ZeroCopyCreationError::IncompatibleNumberOfChannels));

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_channels(NUMBER_OF_CHANNELS - 1)
            .config(&config)
            .create_receiver();

        assert_that!(sut_receiver.err(), eq Some(ZeroCopyCreationError::IncompatibleNumberOfChannels));
    }

    #[test]
    fn connecting_with_compatible_number_of_channels_works<Sut: ZeroCopyConnection>() {
        const NUMBER_OF_CHANNELS: usize = 9;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let _sut_sender = Sut::Builder::new(&name)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .config(&config)
            .create_sender()
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .config(&config)
            .create_receiver();

        assert_that!(sut_receiver, is_ok);
    }

    #[test]
    fn send_receive_and_retrieval_works<Sut: ZeroCopyConnection>() {
        let id = ChannelId::new(0);
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender()
            .unwrap();
        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver()
            .unwrap();

        let sample_offset = SAMPLE_SIZE * 2;
        assert_that!(
            sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE, id),
            is_ok
        );
        let sample = sut_receiver.receive(id).unwrap();
        assert_that!(sample, is_some);
        assert_that!(sample.as_ref().unwrap().offset(), eq sample_offset);

        assert_that!(sut_receiver.release(sample.unwrap(), id), is_ok);
        let retrieval = sut_sender.reclaim(id).unwrap();
        assert_that!(retrieval, is_some);
        assert_that!(retrieval.as_ref().unwrap().offset(), eq sample_offset);

        let retrieval = sut_sender.reclaim(id).unwrap();
        assert_that!(retrieval, is_none);
    }

    #[test]
    fn send_receive_and_retrieval_works_for_multiple_channels<Sut: ZeroCopyConnection>() {
        const NUMBER_OF_CHANNELS: usize = 7;
        const ITERATIONS: usize = 5;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .config(&config)
            .create_sender()
            .unwrap();
        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .config(&config)
            .create_receiver()
            .unwrap();

        for iteration in 0..ITERATIONS {
            for channel_id in 0..NUMBER_OF_CHANNELS {
                let id = ChannelId::new(channel_id);
                let sample_offset = SAMPLE_SIZE * (iteration + 1) * (channel_id + 1);

                assert_that!(
                    sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE, id),
                    is_ok
                );
                assert_that!(sut_receiver.borrow_count(id), eq 0);
                let sample = sut_receiver.receive(id).unwrap();
                assert_that!(sample, is_some);
                assert_that!(sample.as_ref().unwrap().offset(), eq sample_offset);
                assert_that!(sut_receiver.borrow_count(id), eq 1);

                assert_that!(sut_receiver.release(sample.unwrap(), id), is_ok);
                assert_that!(sut_receiver.borrow_count(id), eq 0);
                let retrieval = sut_sender.reclaim(id).unwrap();
                assert_that!(retrieval, is_some);
                assert_that!(retrieval.as_ref().unwrap().offset(), eq sample_offset);

                let retrieval = sut_sender.reclaim(id).unwrap();
                assert_that!(retrieval, is_none);
            }
        }
    }

    #[test]
    fn same_offset_can_be_sent_received_and_reclaimed_via_multiple_channels_in_parallel<
        Sut: ZeroCopyConnection,
    >() {
        const NUMBER_OF_CHANNELS: usize = 7;
        const ITERATIONS: usize = 5;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .config(&config)
            .create_sender()
            .unwrap();
        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .config(&config)
            .create_receiver()
            .unwrap();

        for iteration in 0..ITERATIONS {
            let sample_offset = SAMPLE_SIZE * (iteration + 1);
            // send out first the same sample_offset through all channels
            for channel_id in 0..NUMBER_OF_CHANNELS {
                let id = ChannelId::new(channel_id);

                assert_that!(
                    sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE, id),
                    is_ok
                );
            }

            let mut samples = vec![];
            // receive the sample_offset on all channels
            for channel_id in 0..NUMBER_OF_CHANNELS {
                let id = ChannelId::new(channel_id);
                let sample = sut_receiver.receive(id).unwrap();
                assert_that!(sample, is_some);
                assert_that!(sample.as_ref().unwrap().offset(), eq sample_offset);
                samples.push(sample);
            }

            // release the received samples
            for channel_id in 0..NUMBER_OF_CHANNELS {
                let id = ChannelId::new(channel_id);
                assert_that!(
                    sut_receiver.release(samples[channel_id].unwrap(), id),
                    is_ok
                );
            }

            // reclaim them
            for channel_id in 0..NUMBER_OF_CHANNELS {
                let id = ChannelId::new(channel_id);
                let retrieval = sut_sender.reclaim(id).unwrap();
                assert_that!(retrieval, is_some);
                assert_that!(retrieval.as_ref().unwrap().offset(), eq sample_offset);
            }
        }
    }

    #[test]
    fn when_data_was_sent_receiver_has_data<Sut: ZeroCopyConnection>() {
        let id = ChannelId::new(0);
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender()
            .unwrap();
        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver()
            .unwrap();

        let sample_offset = SAMPLE_SIZE * 2;
        assert_that!(sut_receiver.has_data(id), eq false);
        assert_that!(
            sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE, id),
            is_ok
        );

        assert_that!(sut_receiver.has_data(id), eq true);
    }

    #[test]
    fn data_can_be_received_only_via_the_same_channel<Sut: ZeroCopyConnection>() {
        const ITERATIONS: usize = 8;
        const NUMBER_OF_CHANNELS: usize = 4;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .config(&config)
            .create_sender()
            .unwrap();
        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .config(&config)
            .create_receiver()
            .unwrap();

        for i in 0..ITERATIONS {
            let sample_offset = SAMPLE_SIZE * i;
            for sender_channel_id in 0..NUMBER_OF_CHANNELS {
                // send data via a specific channel
                assert_that!(
                    sut_sender.try_send(
                        PointerOffset::new(sample_offset),
                        SAMPLE_SIZE,
                        ChannelId::new(sender_channel_id)
                    ),
                    is_ok
                );

                // try to receive the data on all channels but only on the same channel it shall
                // succeed
                for receiver_channel_id in 0..NUMBER_OF_CHANNELS {
                    let id = ChannelId::new(receiver_channel_id);
                    if sender_channel_id == receiver_channel_id {
                        assert_that!(sut_receiver.has_data(id), eq true);
                        let sample = sut_receiver.receive(id).unwrap().unwrap();
                        assert_that!(sample.offset(), eq sample_offset);
                        assert_that!(
                            sut_receiver.release(PointerOffset::new(sample_offset), id),
                            is_ok
                        );
                    } else {
                        assert_that!(sut_receiver.has_data(id), eq false);
                    }
                }

                // try to reclaim the data from all channels but only on the same channel it shall
                // succeed
                for sender_channel_reclaim_id in 0..NUMBER_OF_CHANNELS {
                    let id = ChannelId::new(sender_channel_reclaim_id);
                    if sender_channel_reclaim_id == sender_channel_id {
                        let sample = sut_sender.reclaim(id).unwrap().unwrap();
                        assert_that!(sample.offset(), eq sample_offset);
                    } else {
                        assert_that!(sut_sender.reclaim(id).unwrap(), is_none);
                    }
                }
            }
        }
    }

    #[test]
    fn send_until_buffer_is_full_works<Sut: ZeroCopyConnection>() {
        let id = ChannelId::new(0);
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();
        const BUFFER_SIZE: usize = 89;

        let sut_sender = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender()
            .unwrap();

        for i in 0..BUFFER_SIZE {
            let sample_offset = SAMPLE_SIZE * i;
            assert_that!(
                sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE, id),
                is_ok
            );
        }

        let result = sut_sender.try_send(PointerOffset::new(9), SAMPLE_SIZE, id);
        assert_that!(result, is_err);
        assert_that!(result.err().unwrap(), eq ZeroCopySendError::ReceiveBufferFull);
    }

    #[test]
    fn send_until_overflow_works<Sut: ZeroCopyConnection>() {
        let id = ChannelId::new(0);
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();
        const BUFFER_SIZE: usize = 56;

        let sut_sender = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender()
            .unwrap();

        for i in 0..BUFFER_SIZE {
            let sample_offset = SAMPLE_SIZE * i;
            assert_that!(
                sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE, id),
                is_ok
            );
        }

        for i in 0..BUFFER_SIZE {
            let overflow_sample_offset = SAMPLE_SIZE * i;
            let sample_offset = SAMPLE_SIZE * (BUFFER_SIZE + i);
            let result = sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE, id);
            assert_that!(result, is_ok);
            assert_that!(result.ok().unwrap().unwrap().offset(), eq overflow_sample_offset);
        }
    }

    #[test]
    fn receive_can_acquire_data_with_late_connection<Sut: ZeroCopyConnection>() {
        let id = ChannelId::new(0);
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();
        const BUFFER_SIZE: usize = 34;

        let sut_sender = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender()
            .unwrap();

        for i in 0..BUFFER_SIZE {
            let sample_offset = SAMPLE_SIZE * i;
            assert_that!(
                sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE, id),
                is_ok
            );
        }

        let receiver = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver()
            .unwrap();
        for i in 0..BUFFER_SIZE {
            let sample = receiver.receive(id);
            let sample_offset = SAMPLE_SIZE * i;
            assert_that!(sample, is_ok);
            assert_that!(sample.ok().unwrap().unwrap().offset(), eq sample_offset);
        }
    }

    #[test]
    fn new_connection_has_empty_receive_buffer<Sut: ZeroCopyConnection>() {
        let id = ChannelId::new(0);
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver()
            .unwrap();

        let sample = receiver.receive(id).unwrap();
        assert_that!(sample, is_none);
    }

    #[test]
    fn receiver_cannot_borrow_more_samples_than_set_up<Sut: ZeroCopyConnection>() {
        let id = ChannelId::new(0);
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();
        const BUFFER_SIZE: usize = 56;
        const MAX_BORROW: usize = 2;

        let sut_sender = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples_per_channel(MAX_BORROW)
            .enable_safe_overflow(true)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender()
            .unwrap();
        let sut_receiver = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples_per_channel(MAX_BORROW)
            .enable_safe_overflow(true)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver()
            .unwrap();

        let mut sample_offset = SAMPLE_SIZE;
        for _ in 0..2 {
            for _ in 0..BUFFER_SIZE {
                sample_offset += SAMPLE_SIZE;
                assert_that!(
                    sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE, id),
                    is_ok
                );
            }

            let mut samples = vec![];
            for _ in 0..MAX_BORROW {
                let sample = sut_receiver.receive(id).unwrap();
                assert_that!(sample, is_some);
                samples.push(sample.unwrap());
            }

            let result = sut_receiver.receive(id);
            assert_that!(result, is_err);
            assert_that!(
                result.err().unwrap(), eq
                ZeroCopyReceiveError::ReceiveWouldExceedMaxBorrowValue
            );

            for s in samples {
                assert_that!(sut_receiver.release(s, id), is_ok);
                assert_that!(sut_sender.reclaim(id).unwrap(), is_some);
            }
        }
    }

    #[test]
    fn blocking_send_blocks<Sut: ZeroCopyConnection>() {
        let id = ChannelId::new(0);
        let _watchdog = Watchdog::new();
        let name = generate_name();
        let config = Mutex::new(generate_isolated_config::<Sut>());

        let sut_sender = Sut::Builder::new(&name)
            .buffer_size(1)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config.lock().unwrap())
            .create_sender()
            .unwrap();

        let handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new(2).create(&handle).unwrap();

        let sample_offset_1 = SAMPLE_SIZE * 12;
        let sample_offset_2 = SAMPLE_SIZE * 234;

        std::thread::scope(|s| {
            s.spawn(|| {
                let sut_receiver = Sut::Builder::new(&name)
                    .buffer_size(1)
                    .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
                    .config(&config.lock().unwrap())
                    .create_receiver()
                    .unwrap();

                let receive_sample = || loop {
                    if let Some(sample) = sut_receiver.receive(id).unwrap() {
                        return sample;
                    }
                };

                barrier.wait();
                std::thread::sleep(TIMEOUT);
                let sample_1 = receive_sample();
                std::thread::sleep(TIMEOUT);
                let sample_2 = receive_sample();

                assert_that!(sample_1.offset(), eq sample_offset_1);
                assert_that!(sample_2.offset(), eq sample_offset_2);
            });

            barrier.wait();
            let now = Instant::now();

            assert_that!(
                sut_sender.blocking_send(PointerOffset::new(sample_offset_1), SAMPLE_SIZE, id),
                is_ok
            );
            assert_that!(
                sut_sender.blocking_send(PointerOffset::new(sample_offset_2), SAMPLE_SIZE, id),
                is_ok
            );
            assert_that!(now.elapsed(), time_at_least TIMEOUT);
        });
    }

    #[test]
    fn sent_samples_can_be_acquired<Sut: ZeroCopyConnection>() {
        const NUMBER_OF_CHANNELS: usize = 6;
        const BUFFER_SIZE: usize = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .buffer_size(BUFFER_SIZE)
            .config(&config)
            .create_sender()
            .unwrap();
        let _sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .buffer_size(BUFFER_SIZE)
            .config(&config)
            .create_receiver()
            .unwrap();

        let mut offsets = HashSet::new();

        let mut counter = 1;
        for _ in 0..BUFFER_SIZE {
            for id in 0..NUMBER_OF_CHANNELS {
                let sample_offset = SAMPLE_SIZE * counter;
                offsets.insert(sample_offset);
                assert_that!(
                    sut_sender.try_send(
                        PointerOffset::new(sample_offset),
                        SAMPLE_SIZE,
                        ChannelId::new(id)
                    ),
                    is_ok
                );
                counter += 1;
            }
        }

        unsafe {
            sut_sender.acquire_used_offsets(|offset| {
                assert_that!(offsets.remove(&offset.offset()), eq true);
            })
        };
    }

    #[test]
    fn send_samples_can_be_acquired_with_overflow<Sut: ZeroCopyConnection>() {
        let id = ChannelId::new(0);
        const BUFFER_SIZE: usize = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender()
            .unwrap();
        let _sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver()
            .unwrap();

        for i in 0..BUFFER_SIZE {
            let sample_offset = SAMPLE_SIZE * i;
            assert_that!(
                sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE, id),
                is_ok
            );
        }

        let mut offsets = HashSet::new();
        for i in 0..BUFFER_SIZE {
            let sample_offset = SAMPLE_SIZE * (i + BUFFER_SIZE);
            offsets.insert(sample_offset);
            assert_that!(
                sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE, id),
                is_ok
            );
        }

        for _ in 0..BUFFER_SIZE {
            unsafe {
                sut_sender.acquire_used_offsets(|offset| {
                    assert_that!(offsets.remove(&offset.offset()), eq true);
                })
            };
        }
    }

    #[test]
    fn send_and_reclaimed_samples_cannot_be_acquired<Sut: ZeroCopyConnection>() {
        let id = ChannelId::new(0);
        const BUFFER_SIZE: usize = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender()
            .unwrap();
        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver()
            .unwrap();

        for i in 0..BUFFER_SIZE {
            let sample_offset = SAMPLE_SIZE * i;
            assert_that!(
                sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE, id),
                is_ok
            );
        }

        for _ in 0..BUFFER_SIZE {
            let offset = sut_receiver.receive(id).unwrap().unwrap();
            sut_receiver.release(offset, id).unwrap();
        }

        for _ in 0..BUFFER_SIZE {
            assert_that!(sut_sender.reclaim(id).unwrap(), is_some);
        }

        let mut sample_acquired = false;
        unsafe { sut_sender.acquire_used_offsets(|_| sample_acquired = true) };
        assert_that!(sample_acquired, eq false);
    }

    #[test]
    fn send_samples_can_be_acquired_when_receiver_is_dropped<Sut: ZeroCopyConnection>() {
        let id = ChannelId::new(0);
        const BUFFER_SIZE: usize = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender()
            .unwrap();
        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver()
            .unwrap();

        let mut offsets = HashSet::new();
        for i in 0..BUFFER_SIZE {
            let sample_offset = SAMPLE_SIZE * (i + BUFFER_SIZE);
            offsets.insert(sample_offset);
            assert_that!(
                sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE, id),
                is_ok
            );
        }

        for _ in 0..BUFFER_SIZE {
            assert_that!(sut_receiver.receive(id).unwrap(), is_some);
        }

        drop(sut_receiver);

        for _ in 0..BUFFER_SIZE {
            unsafe {
                sut_sender.acquire_used_offsets(|offset| {
                    assert_that!(offsets.remove(&offset.offset()), eq true);
                })
            };
        }
    }

    #[test]
    fn list_connections_works<Sut: ZeroCopyConnection>() {
        let mut sut_names = vec![];
        const LIMIT: usize = 8;
        let config = generate_isolated_config::<Sut>();

        for i in 0..LIMIT {
            assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config).unwrap(), len i);
            sut_names.push(generate_name());
            assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_names[i], &config), eq Ok(false));
            core::mem::forget(
                Sut::Builder::new(&sut_names[i])
                    .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
                    .config(&config)
                    .create_sender()
                    .unwrap(),
            );
            assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_names[i], &config), eq Ok(true));

            let list = <Sut as NamedConceptMgmt>::list_cfg(&config).unwrap();
            assert_that!(list, len i + 1);
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

        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config).unwrap(), len LIMIT);

        for i in 0..LIMIT {
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove_cfg(&sut_names[i], &config)}, eq Ok(true));
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove_cfg(&sut_names[i], &config)}, eq Ok(false));
        }

        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config).unwrap(), len 0);
    }

    #[test]
    fn custom_suffix_keeps_connections_separated<Sut: ZeroCopyConnection>() {
        let config = generate_isolated_config::<Sut>();
        let config_1 = config
            .clone()
            .suffix(unsafe { &FileName::new_unchecked(b".s1") });
        let config_2 = config.suffix(unsafe { &FileName::new_unchecked(b".s2") });

        let sut_name = generate_name();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 0);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let sut_1 = Sut::Builder::new(&sut_name)
            .config(&config_1)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .create_sender()
            .unwrap();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let sut_2 = Sut::Builder::new(&sut_name)
            .config(&config_2)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
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

        core::mem::forget(sut_1);
        core::mem::forget(sut_2);

        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_1)}, eq Ok(true));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_1)}, eq Ok(false));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_2)}, eq Ok(true));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_2)}, eq Ok(false));
    }

    #[test]
    fn defaults_for_configuration_are_set_correctly<Sut: ZeroCopyConnection>() {
        let config = <Sut as NamedConceptMgmt>::Configuration::default();
        assert_that!(*config.get_suffix(), eq Sut::default_suffix());
        assert_that!(*config.get_path_hint(), eq Sut::default_path_hint());
        assert_that!(*config.get_prefix(), eq Sut::default_prefix());
    }

    #[test]
    fn sender_and_receiver_must_have_same_segment_id_requirements<Sut: ZeroCopyConnection>() {
        const BUFFER_SIZE: usize = 10;
        const NUMBER_OF_SEGMENTS: u8 = 123;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let _sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender()
            .unwrap();

        let create_receiver = |number_of_segments: u8| {
            Sut::Builder::new(&name)
                .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
                .buffer_size(BUFFER_SIZE)
                .max_supported_shared_memory_segments(number_of_segments)
                .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
                .enable_safe_overflow(true)
                .config(&config)
                .create_receiver()
        };

        let sut_receiver = create_receiver(NUMBER_OF_SEGMENTS - 1);
        assert_that!(sut_receiver.err(), eq Some(ZeroCopyCreationError::IncompatibleNumberOfSegments));

        let sut_receiver = create_receiver(NUMBER_OF_SEGMENTS + 1);
        assert_that!(sut_receiver.err(), eq Some(ZeroCopyCreationError::IncompatibleNumberOfSegments));

        let sut_receiver = create_receiver(NUMBER_OF_SEGMENTS);
        assert_that!(sut_receiver, is_ok);
    }

    #[cfg(debug_assertions)]
    #[should_panic]
    #[test]
    fn send_pointer_offset_with_out_of_bounds_segment_id_fails<Sut: ZeroCopyConnection>() {
        let id = ChannelId::new(0);
        const BUFFER_SIZE: usize = 10;
        const NUMBER_OF_SEGMENTS: u8 = 123;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender()
            .unwrap();

        // shall panic
        sut_sender
            .try_send(
                PointerOffset::from_offset_and_segment_id(
                    0,
                    SegmentId::new(NUMBER_OF_SEGMENTS + 1),
                ),
                SAMPLE_SIZE,
                id,
            )
            .unwrap();
    }

    #[cfg(debug_assertions)]
    #[should_panic]
    #[test]
    fn release_pointer_offset_with_out_of_bounds_segment_id_fails<Sut: ZeroCopyConnection>() {
        let id = ChannelId::new(0);
        const BUFFER_SIZE: usize = 10;
        const NUMBER_OF_SEGMENTS: u8 = 123;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver()
            .unwrap();

        // shall panic
        sut_receiver
            .release(
                PointerOffset::from_offset_and_segment_id(
                    0,
                    SegmentId::new(NUMBER_OF_SEGMENTS + 1),
                ),
                id,
            )
            .unwrap();
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn receive_pointer_offset_with_out_of_bounds_segment_id_fails<Sut: ZeroCopyConnection>() {
        const BUFFER_SIZE: usize = 10;
        const NUMBER_OF_SEGMENTS: u8 = 123;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender()
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver()
            .unwrap();

        sut_receiver
            .release(
                PointerOffset::from_offset_and_segment_id(
                    0,
                    SegmentId::new(NUMBER_OF_SEGMENTS + 1),
                ),
                ChannelId::new(0),
            )
            .unwrap();

        assert_that!(sut_sender.reclaim(ChannelId::new(0)).err(), eq Some(ZeroCopyReclaimError::ReceiverReturnedCorruptedPointerOffset));
    }

    #[test]
    fn setting_number_of_supported_shared_memory_segments_to_zero_sets_it_to_one<
        Sut: ZeroCopyConnection,
    >() {
        const BUFFER_SIZE: usize = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .max_supported_shared_memory_segments(0)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender()
            .unwrap();

        assert_that!(sut_sender.max_supported_shared_memory_segments(), eq 1);
    }

    #[test]
    fn receiver_cannot_borrow_more_samples_then_set_up_for_multiple_segments<
        Sut: ZeroCopyConnection,
    >() {
        let id = ChannelId::new(0);
        const BUFFER_SIZE: usize = 10;
        const NUMBER_OF_SEGMENTS: u8 = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender()
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver()
            .unwrap();

        for offset in 0..2 {
            for n in 0..BUFFER_SIZE {
                sut_sender
                    .try_send(
                        PointerOffset::from_offset_and_segment_id(
                            offset * SAMPLE_SIZE,
                            SegmentId::new(n as u8),
                        ),
                        SAMPLE_SIZE,
                        id,
                    )
                    .unwrap();
            }
        }

        let mut offsets = vec![];
        for _ in 0..BUFFER_SIZE {
            offsets.push(sut_receiver.receive(id).unwrap().unwrap());
        }

        assert_that!(sut_receiver.receive(id).err(), eq Some(ZeroCopyReceiveError::ReceiveWouldExceedMaxBorrowValue));

        for offset in offsets {
            sut_receiver.release(offset, id).unwrap();
            assert_that!(sut_receiver.receive(id).unwrap(), is_some);
        }

        assert_that!(sut_receiver.receive(id).err(), eq Some(ZeroCopyReceiveError::ReceiveWouldExceedMaxBorrowValue));
    }

    #[test]
    fn receive_with_multiple_segments_and_channels_works<Sut: ZeroCopyConnection>() {
        const BUFFER_SIZE: usize = 10;
        const NUMBER_OF_SEGMENTS: u8 = 10;
        const NUMBER_OF_CHANNELS: usize = 4;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .config(&config)
            .create_sender()
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .config(&config)
            .create_receiver()
            .unwrap();

        for k in 0..2 {
            for n in 0..BUFFER_SIZE {
                for id in 0..NUMBER_OF_CHANNELS {
                    sut_sender
                        .try_send(
                            PointerOffset::from_offset_and_segment_id(
                                k * SAMPLE_SIZE,
                                SegmentId::new(n as u8),
                            ),
                            SAMPLE_SIZE,
                            ChannelId::new(id),
                        )
                        .unwrap();
                }
            }
        }

        for k in 0..2 {
            for n in 0..BUFFER_SIZE {
                for id in 0..NUMBER_OF_CHANNELS {
                    let offset = sut_receiver.receive(ChannelId::new(id)).unwrap().unwrap();
                    assert_that!(offset, eq PointerOffset::from_offset_and_segment_id(
                        k * SAMPLE_SIZE,
                        SegmentId::new(n as u8),
                    ));
                    sut_receiver.release(offset, ChannelId::new(id)).unwrap();
                }
            }
        }
    }

    #[test]
    fn reclaim_works_with_multiple_segments<Sut: ZeroCopyConnection>() {
        let id = ChannelId::new(0);
        const BUFFER_SIZE: usize = 10;
        const NUMBER_OF_SEGMENTS: u8 = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender()
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver()
            .unwrap();

        for k in 0..2 {
            for n in 0..BUFFER_SIZE {
                sut_sender
                    .try_send(
                        PointerOffset::from_offset_and_segment_id(
                            k * SAMPLE_SIZE,
                            SegmentId::new(n as u8),
                        ),
                        SAMPLE_SIZE,
                        id,
                    )
                    .unwrap();
            }
        }

        for _ in 0..2 {
            for _ in 0..BUFFER_SIZE {
                let offset = sut_receiver.receive(id).unwrap().unwrap();
                sut_receiver.release(offset, id).unwrap();
            }
        }

        for k in 0..2 {
            for n in 0..BUFFER_SIZE {
                let offset = sut_sender.reclaim(id).unwrap().unwrap();
                assert_that!(offset, eq PointerOffset::from_offset_and_segment_id(
                    k * SAMPLE_SIZE,
                    SegmentId::new(n as u8),
                ));
            }
        }
    }

    #[test]
    fn acquire_used_offsets_works_with_multiple_segments<Sut: ZeroCopyConnection>() {
        let id = ChannelId::new(0);
        const BUFFER_SIZE: usize = 10;
        const NUMBER_OF_SEGMENTS: u8 = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender()
            .unwrap();

        let _sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver()
            .unwrap();

        let mut offsets = vec![];
        for k in 0..2 {
            for n in 0..BUFFER_SIZE {
                let offset = PointerOffset::from_offset_and_segment_id(
                    k * SAMPLE_SIZE,
                    SegmentId::new(n as u8),
                );
                sut_sender.try_send(offset, SAMPLE_SIZE, id).unwrap();
                offsets.push(offset);
            }
        }

        unsafe {
            sut_sender.acquire_used_offsets(|offset| {
                assert_that!(offsets, contains offset);
            })
        };
    }

    #[cfg(debug_assertions)]
    #[should_panic]
    #[test]
    fn panic_when_same_offset_is_sent_twice_over_same_channel<Sut: ZeroCopyConnection>() {
        let id = ChannelId::new(0);
        const BUFFER_SIZE: usize = 10;
        const NUMBER_OF_SEGMENTS: u8 = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender()
            .unwrap();

        let _sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples_per_channel(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver()
            .unwrap();

        let offset = PointerOffset::from_offset_and_segment_id(SAMPLE_SIZE, SegmentId::new(1_u8));

        assert_that!(sut_sender.try_send(offset, SAMPLE_SIZE, id), is_ok);

        // panics here
        sut_sender.try_send(offset, SAMPLE_SIZE, id).unwrap();
    }

    #[test]
    fn overflow_works_with_multiple_segments<Sut: ZeroCopyConnection>() {
        let id = ChannelId::new(0);
        const NUMBER_OF_SEGMENTS: u8 = 98;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(1)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples_per_channel(1)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender()
            .unwrap();

        let _sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(1)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples_per_channel(1)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver()
            .unwrap();

        let overflow_sample =
            PointerOffset::from_offset_and_segment_id(11 * SAMPLE_SIZE, SegmentId::new(73_u8));
        sut_sender
            .try_send(overflow_sample, SAMPLE_SIZE, id)
            .unwrap();

        let returned_sample = sut_sender
            .try_send(
                PointerOffset::from_offset_and_segment_id(SAMPLE_SIZE, SegmentId::new(1_u8)),
                SAMPLE_SIZE,
                id,
            )
            .unwrap();

        assert_that!(returned_sample, eq Some(overflow_sample));
    }

    #[test]
    fn explicitly_releasing_first_sender_then_receiver_removes_connection<
        Sut: ZeroCopyConnection,
    >() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver()
            .unwrap();
        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender()
            .unwrap();

        core::mem::forget(sut_receiver);
        core::mem::forget(sut_sender);

        assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(true));
        assert_that!(unsafe { Sut::remove_sender(&name, &config) }, is_ok);
        assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(true));
        assert_that!(unsafe { Sut::remove_receiver(&name, &config) }, is_ok);
        assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(false));
    }

    #[test]
    fn explicitly_releasing_first_receiver_then_sender_removes_connection<
        Sut: ZeroCopyConnection,
    >() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver()
            .unwrap();
        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender()
            .unwrap();

        core::mem::forget(sut_receiver);
        core::mem::forget(sut_sender);

        assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(true));
        assert_that!(unsafe { Sut::remove_receiver(&name, &config) }, is_ok);
        assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(true));
        assert_that!(unsafe { Sut::remove_sender(&name, &config) }, is_ok);
        assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(false));
    }

    #[cfg(debug_assertions)]
    #[should_panic]
    #[test]
    fn panics_on_out_of_bounds_channel_id_in_try_send<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .number_of_channels(1)
            .config(&config)
            .create_sender()
            .unwrap();

        // panics here
        let _ = sut_sender.try_send(PointerOffset::new(0), SAMPLE_SIZE, ChannelId::new(1));
    }

    #[cfg(debug_assertions)]
    #[should_panic]
    #[test]
    fn panics_on_out_of_bounds_channel_id_in_blocking_send<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .number_of_channels(1)
            .config(&config)
            .create_sender()
            .unwrap();

        // panics here
        let _ = sut_sender.blocking_send(PointerOffset::new(0), SAMPLE_SIZE, ChannelId::new(1));
    }

    #[cfg(debug_assertions)]
    #[should_panic]
    #[test]
    fn panics_on_out_of_bounds_channel_id_in_reclaim<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .number_of_channels(1)
            .config(&config)
            .create_sender()
            .unwrap();

        // panics here
        let _ = sut_sender.reclaim(ChannelId::new(1));
    }

    #[cfg(debug_assertions)]
    #[should_panic]
    #[test]
    fn panics_on_out_of_bounds_channel_id_in_receive<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .number_of_channels(1)
            .config(&config)
            .create_receiver()
            .unwrap();

        // panics here
        let _ = sut_receiver.receive(ChannelId::new(1));
    }

    #[cfg(debug_assertions)]
    #[should_panic]
    #[test]
    fn panics_on_out_of_bounds_channel_id_in_release<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .number_of_channels(1)
            .config(&config)
            .create_receiver()
            .unwrap();

        // panics here
        let _ = sut_receiver.release(PointerOffset::new(0), ChannelId::new(1));
    }

    #[cfg(debug_assertions)]
    #[should_panic]
    #[test]
    fn panics_on_out_of_bounds_channel_id_in_has_data<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .number_of_channels(1)
            .config(&config)
            .create_receiver()
            .unwrap();

        // panics here
        let _ = sut_receiver.has_data(ChannelId::new(1));
    }

    #[test]
    fn removing_port_from_non_existing_connection_leads_to_error<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        assert_that!(unsafe { Sut::remove_receiver(&name, &config) }, eq Err(ZeroCopyPortRemoveError::DoesNotExist));
        assert_that!(unsafe { Sut::remove_sender(&name, &config) }, eq Err(ZeroCopyPortRemoveError::DoesNotExist));
    }

    #[ignore] // TODO: iox2-671 enable this test when the concurrency issue is fixed.
    #[test]
    fn concurrent_creation_and_destruction_works<Sut: ZeroCopyConnection>() {
        const ITERATIONS: usize = 1000;
        let barrier_1 = Arc::new(Barrier::new(2));
        let barrier_2 = barrier_1.clone();
        let name_1 = generate_name();
        let name_2 = generate_name();
        let config_1 = generate_isolated_config::<Sut>();
        let config_2 = config_1.clone();

        let verify = |error: ZeroCopyCreationError| {
            assert_that!(error == ZeroCopyCreationError::IsBeingCleanedUp || error == ZeroCopyCreationError::InitializationNotYetFinalized, eq true);
        };

        std::thread::scope(|s| {
            let tname_1 = name_1.clone();
            let tname_2 = name_2.clone();
            s.spawn(move || {
                barrier_1.wait();
                for _ in 0..ITERATIONS {
                    let sut_sender = Sut::Builder::new(&tname_1)
                        .config(&config_1)
                        .create_sender();
                    let sut_receiver = Sut::Builder::new(&tname_2)
                        .config(&config_1)
                        .create_receiver();

                    if let Some(e) = sut_sender.err() {
                        verify(e);
                    }

                    if let Some(e) = sut_receiver.err() {
                        verify(e);
                    }
                }
            });

            s.spawn(move || {
                barrier_2.wait();
                for _ in 0..ITERATIONS {
                    let sut_receiver = Sut::Builder::new(&name_1)
                        .config(&config_2)
                        .create_receiver();
                    let sut_sender = Sut::Builder::new(&name_2).config(&config_2).create_sender();

                    if let Some(e) = sut_sender.err() {
                        verify(e);
                    }

                    if let Some(e) = sut_receiver.err() {
                        verify(e);
                    }
                }
            });
        });
    }

    #[test]
    fn channel_state_is_set_to_default_value_on_creation<Sut: ZeroCopyConnection>() {
        const NUMBER_OF_CHANNELS: usize = 12;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Builder::new(&name)
            .config(&config)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .create_receiver()
            .unwrap();

        for id in 0..NUMBER_OF_CHANNELS {
            assert_that!(sut_receiver.channel_state(ChannelId::new(id)).load(Ordering::Relaxed), eq INITIAL_CHANNEL_STATE);
        }
        drop(sut_receiver);

        let sut_sender = Sut::Builder::new(&name)
            .config(&config)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .create_sender()
            .unwrap();

        for id in 0..NUMBER_OF_CHANNELS {
            assert_that!(sut_sender.channel_state(ChannelId::new(id)).load(Ordering::Relaxed), eq INITIAL_CHANNEL_STATE);
        }
        drop(sut_sender);
    }

    #[test]
    fn initial_channel_state_can_be_defined_for_all_channels<Sut: ZeroCopyConnection>() {
        const NUMBER_OF_CHANNELS: usize = 11;
        const CUSTOM_INITIAL_CHANNEL_STATE: u64 = 981273;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Builder::new(&name)
            .config(&config)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .initial_channel_state(CUSTOM_INITIAL_CHANNEL_STATE)
            .create_receiver()
            .unwrap();

        for id in 0..NUMBER_OF_CHANNELS {
            assert_that!(sut_receiver.channel_state(ChannelId::new(id)).load(Ordering::Relaxed), eq CUSTOM_INITIAL_CHANNEL_STATE);
        }
        drop(sut_receiver);

        let sut_sender = Sut::Builder::new(&name)
            .config(&config)
            .number_of_channels(NUMBER_OF_CHANNELS)
            .initial_channel_state(CUSTOM_INITIAL_CHANNEL_STATE)
            .create_sender()
            .unwrap();

        for id in 0..NUMBER_OF_CHANNELS {
            assert_that!(sut_sender.channel_state(ChannelId::new(id)).load(Ordering::Relaxed), eq CUSTOM_INITIAL_CHANNEL_STATE);
        }
        drop(sut_sender);
    }

    #[test]
    fn changing_channel_state_works<Sut: ZeroCopyConnection>() {
        const CHANNEL_ID: ChannelId = ChannelId::new(0);
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Builder::new(&name)
            .config(&config)
            .create_receiver()
            .unwrap();
        sut_receiver
            .channel_state(CHANNEL_ID)
            .store(456, Ordering::Relaxed);

        let sut_sender = Sut::Builder::new(&name)
            .config(&config)
            .create_sender()
            .unwrap();
        assert_that!(sut_sender.channel_state(CHANNEL_ID).load(Ordering::Relaxed), eq 456);
        sut_sender
            .channel_state(CHANNEL_ID)
            .store(789, Ordering::Relaxed);

        assert_that!(sut_receiver.channel_state(CHANNEL_ID).load(Ordering::Relaxed), eq 789);
    }

    #[instantiate_tests(<zero_copy_connection::posix_shared_memory::Connection>)]
    mod posix_shared_memory {}

    #[instantiate_tests(<zero_copy_connection::process_local::Connection>)]
    mod process_local {}
}
