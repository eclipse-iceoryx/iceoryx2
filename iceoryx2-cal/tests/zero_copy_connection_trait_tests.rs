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
    use core::time::Duration;
    use std::collections::HashSet;
    use std::sync::Mutex;
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
    use iceoryx2_cal::zero_copy_connection;
    use iceoryx2_cal::zero_copy_connection::*;

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
                .create_receiver(ChannelId::new(0)),
            is_ok
        );
        assert_that!(
            Sut::Builder::new(&name)
                .config(&config)
                .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
                .create_sender(ChannelId::new(0)),
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
            .create_sender(ChannelId::new(0))
            .unwrap();
        assert_that!(!sut_sender.is_connected(), eq true);

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver(ChannelId::new(0))
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
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();
        assert_that!(sut_sender.buffer_size(), eq DEFAULT_BUFFER_SIZE);
        assert_that!(
            sut_sender.max_borrowed_samples(), eq
            DEFAULT_MAX_BORROWED_SAMPLES
        );
        assert_that!(
            sut_sender.has_enabled_safe_overflow(), eq
            DEFAULT_ENABLE_SAFE_OVERFLOW
        );

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver(ChannelId::new(0))
            .unwrap();
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
        let config = generate_isolated_config::<Sut>();

        let _sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();
        let _sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver(ChannelId::new(0))
            .unwrap();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender(ChannelId::new(0));
        assert_that!(sut_sender, is_err);
        assert_that!(
            sut_sender.err().unwrap(), eq
            ZeroCopyCreationError::AnotherInstanceIsAlreadyConnected
        );

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver(ChannelId::new(0));
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
            .create_sender(ChannelId::new(0))
            .unwrap();
        let _sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver(ChannelId::new(0))
            .unwrap();

        drop(_sut_sender);
        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender(ChannelId::new(0));
        assert_that!(sut_sender, is_ok);
    }

    #[test]
    fn when_receiver_goes_out_of_scope_another_receiver_can_connect<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let _sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();
        let _sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver(ChannelId::new(0))
            .unwrap();

        drop(_sut_receiver);
        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver(ChannelId::new(0));
        assert_that!(sut_receiver, is_ok);
    }

    #[test]
    fn connection_is_cleaned_up_when_unused<Sut: ZeroCopyConnection + NamedConceptMgmt>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();
        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver(ChannelId::new(0))
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
            .create_sender(ChannelId::new(0))
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .buffer_size(16)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver(ChannelId::new(0));

        assert_that!(sut_receiver, is_err);
    }

    #[test]
    fn connecting_with_incompatible_borrow_max_fails<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let _sut_sender = Sut::Builder::new(&name)
            .receiver_max_borrowed_samples(2)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .receiver_max_borrowed_samples(4)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver(ChannelId::new(0));

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
            .create_sender(ChannelId::new(0))
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver(ChannelId::new(0));

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
            .create_sender(ChannelId::new(0))
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES * 2)
            .config(&config)
            .create_receiver(ChannelId::new(0));

        assert_that!(sut_receiver, is_err);
    }

    #[test]
    fn send_receive_and_retrieval_works<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();
        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver(ChannelId::new(0))
            .unwrap();

        let sample_offset = SAMPLE_SIZE * 2;
        assert_that!(
            sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE),
            is_ok
        );
        let sample = sut_receiver.receive().unwrap();
        assert_that!(sample, is_some);
        assert_that!(sample.as_ref().unwrap().offset(), eq sample_offset);

        assert_that!(sut_receiver.release(sample.unwrap()), is_ok);
        let retrieval = sut_sender.reclaim().unwrap();
        assert_that!(retrieval, is_some);
        assert_that!(retrieval.as_ref().unwrap().offset(), eq sample_offset);

        let retrieval = sut_sender.reclaim().unwrap();
        assert_that!(retrieval, is_none);
    }

    #[test]
    fn when_data_was_sent_receiver_has_data<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();
        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver(ChannelId::new(0))
            .unwrap();

        let sample_offset = SAMPLE_SIZE * 2;
        assert_that!(sut_receiver.has_data(), eq false);
        assert_that!(
            sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE),
            is_ok
        );

        assert_that!(sut_receiver.has_data(), eq true);
    }

    #[test]
    fn send_until_buffer_is_full_works<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();
        const BUFFER_SIZE: usize = 89;

        let sut_sender = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();

        for i in 0..BUFFER_SIZE {
            let sample_offset = SAMPLE_SIZE * i;
            assert_that!(
                sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE),
                is_ok
            );
        }

        let result = sut_sender.try_send(PointerOffset::new(9), SAMPLE_SIZE);
        assert_that!(result, is_err);
        assert_that!(result.err().unwrap(), eq ZeroCopySendError::ReceiveBufferFull);
    }

    #[test]
    fn send_until_overflow_works<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();
        const BUFFER_SIZE: usize = 56;

        let sut_sender = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();

        for i in 0..BUFFER_SIZE {
            let sample_offset = SAMPLE_SIZE * i;
            assert_that!(
                sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE),
                is_ok
            );
        }

        for i in 0..BUFFER_SIZE {
            let overflow_sample_offset = SAMPLE_SIZE * i;
            let sample_offset = SAMPLE_SIZE * (BUFFER_SIZE + i);
            let result = sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE);
            assert_that!(result, is_ok);
            assert_that!(result.ok().unwrap().unwrap().offset(), eq overflow_sample_offset);
        }
    }

    #[test]
    fn receive_can_acquire_data_with_late_connection<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();
        const BUFFER_SIZE: usize = 34;

        let sut_sender = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();

        for i in 0..BUFFER_SIZE {
            let sample_offset = SAMPLE_SIZE * i;
            assert_that!(
                sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE),
                is_ok
            );
        }

        let receiver = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver(ChannelId::new(0))
            .unwrap();
        for i in 0..BUFFER_SIZE {
            let sample = receiver.receive();
            let sample_offset = SAMPLE_SIZE * i;
            assert_that!(sample, is_ok);
            assert_that!(sample.ok().unwrap().unwrap().offset(), eq sample_offset);
        }
    }

    #[test]
    fn new_connection_has_empty_receive_buffer<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver(ChannelId::new(0))
            .unwrap();

        let sample = receiver.receive().unwrap();
        assert_that!(sample, is_none);
    }

    #[test]
    fn receiver_cannot_borrow_more_samples_than_set_up<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();
        const BUFFER_SIZE: usize = 56;
        const MAX_BORROW: usize = 2;

        let sut_sender = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples(MAX_BORROW)
            .enable_safe_overflow(true)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();
        let sut_receiver = Sut::Builder::new(&name)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples(MAX_BORROW)
            .enable_safe_overflow(true)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_receiver(ChannelId::new(0))
            .unwrap();

        let mut sample_offset = SAMPLE_SIZE;
        for _ in 0..2 {
            for _ in 0..BUFFER_SIZE {
                sample_offset += SAMPLE_SIZE;
                assert_that!(
                    sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE),
                    is_ok
                );
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
        let _watchdog = Watchdog::new();
        let name = generate_name();
        let config = Mutex::new(generate_isolated_config::<Sut>());

        let sut_sender = Sut::Builder::new(&name)
            .buffer_size(1)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config.lock().unwrap())
            .create_sender(ChannelId::new(0))
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
                    .create_receiver(ChannelId::new(0))
                    .unwrap();

                let receive_sample = || loop {
                    if let Some(sample) = sut_receiver.receive().unwrap() {
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
                sut_sender.blocking_send(PointerOffset::new(sample_offset_1), SAMPLE_SIZE),
                is_ok
            );
            assert_that!(
                sut_sender.blocking_send(PointerOffset::new(sample_offset_2), SAMPLE_SIZE),
                is_ok
            );
            assert_that!(now.elapsed(), time_at_least TIMEOUT);
        });
    }

    #[test]
    fn send_samples_can_be_acquired<Sut: ZeroCopyConnection>() {
        const BUFFER_SIZE: usize = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();
        let _sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .config(&config)
            .create_receiver(ChannelId::new(0))
            .unwrap();

        let mut offsets = HashSet::new();

        for i in 0..BUFFER_SIZE {
            let sample_offset = SAMPLE_SIZE * i;
            offsets.insert(sample_offset);
            assert_that!(
                sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE),
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
    fn send_samples_can_be_acquired_with_overflow<Sut: ZeroCopyConnection>() {
        const BUFFER_SIZE: usize = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();
        let _sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver(ChannelId::new(0))
            .unwrap();

        for i in 0..BUFFER_SIZE {
            let sample_offset = SAMPLE_SIZE * i;
            assert_that!(
                sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE),
                is_ok
            );
        }

        let mut offsets = HashSet::new();
        for i in 0..BUFFER_SIZE {
            let sample_offset = SAMPLE_SIZE * (i + BUFFER_SIZE);
            offsets.insert(sample_offset);
            assert_that!(
                sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE),
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
        const BUFFER_SIZE: usize = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();
        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver(ChannelId::new(0))
            .unwrap();

        for i in 0..BUFFER_SIZE {
            let sample_offset = SAMPLE_SIZE * i;
            assert_that!(
                sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE),
                is_ok
            );
        }

        for _ in 0..BUFFER_SIZE {
            let offset = sut_receiver.receive().unwrap().unwrap();
            sut_receiver.release(offset).unwrap();
        }

        for _ in 0..BUFFER_SIZE {
            assert_that!(sut_sender.reclaim().unwrap(), is_some);
        }

        let mut sample_acquired = false;
        unsafe { sut_sender.acquire_used_offsets(|_| sample_acquired = true) };
        assert_that!(sample_acquired, eq false);
    }

    #[test]
    fn send_samples_can_be_acquired_when_receiver_is_dropped<Sut: ZeroCopyConnection>() {
        const BUFFER_SIZE: usize = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();
        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver(ChannelId::new(0))
            .unwrap();

        let mut offsets = HashSet::new();
        for i in 0..BUFFER_SIZE {
            let sample_offset = SAMPLE_SIZE * (i + BUFFER_SIZE);
            offsets.insert(sample_offset);
            assert_that!(
                sut_sender.try_send(PointerOffset::new(sample_offset), SAMPLE_SIZE),
                is_ok
            );
        }

        for _ in 0..BUFFER_SIZE {
            assert_that!(sut_receiver.receive().unwrap(), is_some);
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
                    .create_sender(ChannelId::new(0))
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
            .create_sender(ChannelId::new(0))
            .unwrap();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let sut_2 = Sut::Builder::new(&sut_name)
            .config(&config_2)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .create_sender(ChannelId::new(0))
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
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();

        let create_receiver = |number_of_segments: u8| {
            Sut::Builder::new(&name)
                .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
                .buffer_size(BUFFER_SIZE)
                .max_supported_shared_memory_segments(number_of_segments)
                .receiver_max_borrowed_samples(BUFFER_SIZE)
                .enable_safe_overflow(true)
                .config(&config)
                .create_receiver(ChannelId::new(0))
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
        const BUFFER_SIZE: usize = 10;
        const NUMBER_OF_SEGMENTS: u8 = 123;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();

        // shall panic
        sut_sender
            .try_send(
                PointerOffset::from_offset_and_segment_id(
                    0,
                    SegmentId::new(NUMBER_OF_SEGMENTS + 1),
                ),
                SAMPLE_SIZE,
            )
            .unwrap();
    }

    #[cfg(debug_assertions)]
    #[should_panic]
    #[test]
    fn release_pointer_offset_with_out_of_bounds_segment_id_fails<Sut: ZeroCopyConnection>() {
        const BUFFER_SIZE: usize = 10;
        const NUMBER_OF_SEGMENTS: u8 = 123;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver(ChannelId::new(0))
            .unwrap();

        // shall panic
        sut_receiver
            .release(PointerOffset::from_offset_and_segment_id(
                0,
                SegmentId::new(NUMBER_OF_SEGMENTS + 1),
            ))
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
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender()
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver()
            .unwrap();

        sut_receiver
            .release(PointerOffset::from_offset_and_segment_id(
                0,
                SegmentId::new(NUMBER_OF_SEGMENTS + 1),
            ))
            .unwrap();

        assert_that!(sut_sender.reclaim().err(), eq Some(ZeroCopyReclaimError::ReceiverReturnedCorruptedPointerOffset));
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
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();

        assert_that!(sut_sender.max_supported_shared_memory_segments(), eq 1);
    }

    #[test]
    fn receiver_cannot_borrow_more_samples_then_set_up_for_multiple_segments<
        Sut: ZeroCopyConnection,
    >() {
        const BUFFER_SIZE: usize = 10;
        const NUMBER_OF_SEGMENTS: u8 = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver(ChannelId::new(0))
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
                    )
                    .unwrap();
            }
        }

        let mut offsets = vec![];
        for _ in 0..BUFFER_SIZE {
            offsets.push(sut_receiver.receive().unwrap().unwrap());
        }

        assert_that!(sut_receiver.receive().err(), eq Some(ZeroCopyReceiveError::ReceiveWouldExceedMaxBorrowValue));

        for offset in offsets {
            sut_receiver.release(offset).unwrap();
            assert_that!(sut_receiver.receive().unwrap(), is_some);
        }

        assert_that!(sut_receiver.receive().err(), eq Some(ZeroCopyReceiveError::ReceiveWouldExceedMaxBorrowValue));
    }

    #[test]
    fn receive_with_multiple_segments_works<Sut: ZeroCopyConnection>() {
        const BUFFER_SIZE: usize = 10;
        const NUMBER_OF_SEGMENTS: u8 = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver(ChannelId::new(0))
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
                    )
                    .unwrap();
            }
        }

        for k in 0..2 {
            for n in 0..BUFFER_SIZE {
                let offset = sut_receiver.receive().unwrap().unwrap();
                assert_that!(offset, eq PointerOffset::from_offset_and_segment_id(
                    k * SAMPLE_SIZE,
                    SegmentId::new(n as u8),
                ));
                sut_receiver.release(offset).unwrap();
            }
        }
    }

    #[test]
    fn reclaim_works_with_multiple_segments<Sut: ZeroCopyConnection>() {
        const BUFFER_SIZE: usize = 10;
        const NUMBER_OF_SEGMENTS: u8 = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();

        let sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver(ChannelId::new(0))
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
                    )
                    .unwrap();
            }
        }

        for _ in 0..2 {
            for _ in 0..BUFFER_SIZE {
                let offset = sut_receiver.receive().unwrap().unwrap();
                sut_receiver.release(offset).unwrap();
            }
        }

        for k in 0..2 {
            for n in 0..BUFFER_SIZE {
                let offset = sut_sender.reclaim().unwrap().unwrap();
                assert_that!(offset, eq PointerOffset::from_offset_and_segment_id(
                    k * SAMPLE_SIZE,
                    SegmentId::new(n as u8),
                ));
            }
        }
    }

    #[test]
    fn acquire_used_offsets_works_with_multiple_segments<Sut: ZeroCopyConnection>() {
        const BUFFER_SIZE: usize = 10;
        const NUMBER_OF_SEGMENTS: u8 = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();

        let _sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver(ChannelId::new(0))
            .unwrap();

        let mut offsets = vec![];
        for k in 0..2 {
            for n in 0..BUFFER_SIZE {
                let offset = PointerOffset::from_offset_and_segment_id(
                    k * SAMPLE_SIZE,
                    SegmentId::new(n as u8),
                );
                sut_sender.try_send(offset, SAMPLE_SIZE).unwrap();
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
    fn panic_when_same_offset_is_sent_twice<Sut: ZeroCopyConnection>() {
        const BUFFER_SIZE: usize = 10;
        const NUMBER_OF_SEGMENTS: u8 = 10;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();

        let _sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(2 * BUFFER_SIZE)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples(BUFFER_SIZE)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver(ChannelId::new(0))
            .unwrap();

        let offset =
            PointerOffset::from_offset_and_segment_id(SAMPLE_SIZE, SegmentId::new(1 as u8));

        assert_that!(sut_sender.try_send(offset, SAMPLE_SIZE), is_ok);

        // panics here
        sut_sender.try_send(offset, SAMPLE_SIZE).unwrap();
    }

    #[test]
    fn overflow_works_with_multiple_segments<Sut: ZeroCopyConnection>() {
        const NUMBER_OF_SEGMENTS: u8 = 98;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(1)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples(1)
            .enable_safe_overflow(true)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();

        let _sut_receiver = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .buffer_size(1)
            .max_supported_shared_memory_segments(NUMBER_OF_SEGMENTS)
            .receiver_max_borrowed_samples(1)
            .enable_safe_overflow(true)
            .config(&config)
            .create_receiver(ChannelId::new(0))
            .unwrap();

        let overflow_sample =
            PointerOffset::from_offset_and_segment_id(11 * SAMPLE_SIZE, SegmentId::new(73 as u8));
        sut_sender.try_send(overflow_sample, SAMPLE_SIZE).unwrap();

        let returned_sample = sut_sender
            .try_send(
                PointerOffset::from_offset_and_segment_id(SAMPLE_SIZE, SegmentId::new(1 as u8)),
                SAMPLE_SIZE,
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
            .create_receiver(ChannelId::new(0))
            .unwrap();
        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();

        core::mem::forget(sut_receiver);
        core::mem::forget(sut_sender);

        assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(true));
        assert_that!(
            unsafe { Sut::remove_sender(&name, &config, ChannelId::new(0)) },
            is_ok
        );
        assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(true));
        assert_that!(
            unsafe { Sut::remove_receiver(&name, &config, ChannelId::new(0)) },
            is_ok
        );
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
            .create_receiver(ChannelId::new(0))
            .unwrap();
        let sut_sender = Sut::Builder::new(&name)
            .number_of_samples_per_segment(NUMBER_OF_SAMPLES)
            .config(&config)
            .create_sender(ChannelId::new(0))
            .unwrap();

        core::mem::forget(sut_receiver);
        core::mem::forget(sut_sender);

        assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(true));
        assert_that!(
            unsafe { Sut::remove_receiver(&name, &config, ChannelId::new(0)) },
            is_ok
        );
        assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(true));
        assert_that!(
            unsafe { Sut::remove_sender(&name, &config, ChannelId::new(0)) },
            is_ok
        );
        assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(false));
    }

    #[test]
    fn removing_port_from_non_existing_connection_leads_to_error<Sut: ZeroCopyConnection>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        assert_that!(unsafe { Sut::remove_receiver(&name, &config, ChannelId::new(0)) }, eq Err(ZeroCopyPortRemoveError::DoesNotExist));
        assert_that!(unsafe { Sut::remove_sender(&name, &config, ChannelId::new(0)) }, eq Err(ZeroCopyPortRemoveError::DoesNotExist));
    }

    #[instantiate_tests(<zero_copy_connection::posix_shared_memory::Connection>)]
    mod posix_shared_memory {}

    #[instantiate_tests(<zero_copy_connection::process_local::Connection>)]
    mod process_local {}
}
