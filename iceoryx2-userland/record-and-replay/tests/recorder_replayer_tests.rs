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

#[cfg(test)]
mod recorder_replayer {
    use core::time::Duration;

    use core::sync::atomic::Ordering;
    use iceoryx2::{
        prelude::MessagingPattern, service::static_config::message_type_details::TypeVariant,
    };
    use iceoryx2_bb_posix::file::File;
    use iceoryx2_pal_testing::assert_that;
    use iceoryx2_userland_record_and_replay::{
        record::{DataRepresentation, RawRecord},
        recorder::{RecorderBuilder, RecorderWriteError, ServiceTypes},
        replayer::{ReplayerOpenError, ReplayerOpener},
        testing,
    };

    use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeNameString};
    use iceoryx2_bb_posix::testing::generate_file_name;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU8;

    fn generate_type_detail(variant: TypeVariant, size: usize, alignment: usize) -> TypeDetail {
        iceoryx2::testing::create_custom_type_detail(
            variant,
            TypeNameString::from_str_truncated(&UniqueSystemId::new().unwrap().value().to_string()),
            size,
            alignment,
        )
    }

    struct Data {
        payload: Vec<u8>,
        user_header: Vec<u8>,
        system_header: Vec<u8>,
        timestamp: Duration,
    }

    fn generate_data(len: usize) -> Vec<u8> {
        static COUNTER: IoxAtomicU8 = IoxAtomicU8::new(0);

        let mut data = vec![];

        for _ in 0..len {
            data.push(COUNTER.fetch_add(0, Ordering::Relaxed));
        }
        data
    }

    fn generate_service_data(types: &ServiceTypes, timestamp: Duration) -> Data {
        Data {
            payload: generate_data(types.payload.size()),
            user_header: generate_data(types.user_header.size()),
            system_header: generate_data(types.system_header.size()),
            timestamp,
        }
    }

    fn record_and_replay_works(
        data_representation: DataRepresentation,
        messaging_pattern: MessagingPattern,
        user_header_type: TypeDetail,
        number_of_data: usize,
    ) {
        let service_name = iceoryx2::testing::generate_service_name();
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::FixedSize, 8, 4),
            user_header: user_header_type,
            system_header: generate_type_detail(TypeVariant::FixedSize, 16, 8),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .messaging_pattern(messaging_pattern)
            .create(&file_name, &service_name)
            .unwrap();

        let mut dataset = vec![];
        for n in 0..number_of_data {
            dataset.push(generate_service_data(&types, Duration::from_millis(n as _)))
        }

        for data in &dataset {
            assert_that!(
                recorder.write(RawRecord {
                    timestamp: data.timestamp,
                    system_header: &data.system_header,
                    user_header: &data.user_header,
                    payload: &data.payload
                }),
                is_ok
            );
        }

        let replay = ReplayerOpener::new(&file_name)
            .data_representation(data_representation)
            .open()
            .unwrap();
        let record_header = replay.header().clone();
        let buffer = replay.read_into_buffer().unwrap();

        assert_that!(record_header, eq * recorder.header());
        assert_that!(buffer, len dataset.len());

        for n in 0..dataset.len() {
            assert_that!(buffer[n].payload, eq dataset[n].payload);
            assert_that!(buffer[n].user_header, eq dataset[n].user_header);
            assert_that!(buffer[n].system_header, eq dataset[n].system_header);
            assert_that!(buffer[n].timestamp, eq dataset[n].timestamp);
        }

        File::remove(&file_name).unwrap();
    }

    #[test]
    fn record_and_replay_works_for_iox2dump() {
        record_and_replay_works(
            DataRepresentation::Iox2Dump,
            MessagingPattern::PublishSubscribe,
            generate_type_detail(TypeVariant::FixedSize, 16, 8),
            129,
        );
    }

    #[test]
    fn record_and_replay_works_for_human_readable() {
        record_and_replay_works(
            DataRepresentation::HumanReadable,
            MessagingPattern::RequestResponse,
            generate_type_detail(TypeVariant::FixedSize, 32, 8),
            89,
        );
    }

    #[test]
    fn record_and_replay_works_for_iox2dump_with_unit_user_header() {
        record_and_replay_works(
            DataRepresentation::Iox2Dump,
            MessagingPattern::PublishSubscribe,
            TypeDetail::new::<()>(TypeVariant::FixedSize),
            145,
        );
    }

    #[test]
    fn record_and_replay_works_for_human_readable_with_unit_user_header() {
        record_and_replay_works(
            DataRepresentation::HumanReadable,
            MessagingPattern::RequestResponse,
            TypeDetail::new::<()>(TypeVariant::FixedSize),
            99,
        );
    }

    #[test]
    fn record_and_replay_works_with_empty_record_for_iox2dump() {
        record_and_replay_works(
            DataRepresentation::Iox2Dump,
            MessagingPattern::PublishSubscribe,
            generate_type_detail(TypeVariant::FixedSize, 16, 8),
            0,
        );
    }

    #[test]
    fn record_and_replay_works_with_empty_record_for_human_readable() {
        record_and_replay_works(
            DataRepresentation::HumanReadable,
            MessagingPattern::RequestResponse,
            generate_type_detail(TypeVariant::FixedSize, 32, 8),
            0,
        );
    }

    fn record_and_replay_with_dynamic_payload_works(
        data_representation: DataRepresentation,
        messaging_pattern: MessagingPattern,
    ) {
        const NUMBER_OF_DATA: usize = 15;
        let service_name = iceoryx2::testing::generate_service_name();
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::Dynamic, 256, 8),
            user_header: generate_type_detail(TypeVariant::FixedSize, 8, 2),
            system_header: generate_type_detail(TypeVariant::FixedSize, 32, 4),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .messaging_pattern(messaging_pattern)
            .create(&file_name, &service_name)
            .unwrap();

        let mut dataset = vec![];
        for n in 0..NUMBER_OF_DATA {
            let mut data = generate_service_data(&types, Duration::from_millis(n as _));
            data.payload = generate_data(types.payload.size() * (n + 1));
            dataset.push(data);
        }

        for data in &dataset {
            assert_that!(
                recorder.write(RawRecord {
                    timestamp: data.timestamp,
                    system_header: &data.system_header,
                    user_header: &data.user_header,
                    payload: &data.payload
                }),
                is_ok
            );
        }

        let replay = ReplayerOpener::new(&file_name)
            .data_representation(data_representation)
            .open()
            .unwrap();
        let record_header = replay.header().clone();
        let buffer = replay.read_into_buffer().unwrap();

        assert_that!(record_header, eq * recorder.header());
        assert_that!(buffer, len dataset.len());

        for n in 0..dataset.len() {
            assert_that!(buffer[n].payload, eq dataset[n].payload);
            assert_that!(buffer[n].user_header, eq dataset[n].user_header);
            assert_that!(buffer[n].system_header, eq dataset[n].system_header);
            assert_that!(buffer[n].timestamp, eq dataset[n].timestamp);
        }

        File::remove(&file_name).unwrap();
    }

    #[test]
    fn record_and_replay_with_dynamic_payload_works_for_iox2dump() {
        record_and_replay_with_dynamic_payload_works(
            DataRepresentation::Iox2Dump,
            MessagingPattern::PublishSubscribe,
        );
    }

    #[test]
    fn record_and_replay_with_dynamic_payload_works_for_human_readable() {
        record_and_replay_with_dynamic_payload_works(
            DataRepresentation::HumanReadable,
            MessagingPattern::RequestResponse,
        );
    }

    fn writing_decreasing_timestamps_fails(data_representation: DataRepresentation) {
        let service_name = iceoryx2::testing::generate_service_name();
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::FixedSize, 8, 4),
            user_header: TypeDetail::new::<()>(TypeVariant::FixedSize),
            system_header: generate_type_detail(TypeVariant::FixedSize, 16, 8),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .create(&file_name, &service_name)
            .unwrap();

        let mut dataset = vec![];
        dataset.push(generate_service_data(&types, Duration::from_millis(5)));
        dataset.push(generate_service_data(&types, Duration::from_millis(2)));

        assert_that!(
            recorder.write(RawRecord {
                timestamp: dataset[0].timestamp,
                system_header: &dataset[0].system_header,
                user_header: &dataset[0].user_header,
                payload: &dataset[0].payload
            }),
            is_ok
        );

        assert_that!(
            recorder.write(RawRecord {
                timestamp: dataset[1].timestamp,
                system_header: &dataset[1].system_header,
                user_header: &dataset[1].user_header,
                payload: &dataset[1].payload
            }).err(),
            eq Some(RecorderWriteError::TimestampOlderThanPreviousRecord)
        );

        File::remove(&file_name).unwrap();
    }

    #[test]
    fn writing_decreasing_timestamps_fails_for_iox2dump() {
        writing_decreasing_timestamps_fails(DataRepresentation::Iox2Dump);
    }

    #[test]
    fn writing_decreasing_timestamps_fails_for_human_readable() {
        writing_decreasing_timestamps_fails(DataRepresentation::HumanReadable);
    }

    fn writing_invalid_fixed_size_payload_fails(
        data_representation: DataRepresentation,
        messaging_pattern: MessagingPattern,
    ) {
        let service_name = iceoryx2::testing::generate_service_name();
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::FixedSize, 32, 4),
            user_header: generate_type_detail(TypeVariant::FixedSize, 64, 8),
            system_header: generate_type_detail(TypeVariant::FixedSize, 128, 64),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .messaging_pattern(messaging_pattern)
            .create(&file_name, &service_name)
            .unwrap();

        let mut data = generate_service_data(&types, Duration::ZERO);
        data.payload = generate_data(types.payload.size() + 1);

        assert_that!(
            recorder.write(RawRecord {
                timestamp: data.timestamp,
                system_header: &data.system_header,
                user_header: &data.user_header,
                payload: &data.payload
            }),
            eq Err(RecorderWriteError::CorruptedPayloadRecord)
        );

        let replay = ReplayerOpener::new(&file_name)
            .data_representation(data_representation)
            .open()
            .unwrap();
        let record_header = replay.header().clone();
        let buffer = replay.read_into_buffer().unwrap();

        assert_that!(record_header, eq * recorder.header());
        assert_that!(buffer, len 0);

        File::remove(&file_name).unwrap();
    }

    #[test]
    fn writing_invalid_fixed_size_payload_fails_for_iox2dump() {
        writing_invalid_fixed_size_payload_fails(
            DataRepresentation::Iox2Dump,
            MessagingPattern::PublishSubscribe,
        );
    }

    #[test]
    fn writing_invalid_fixed_size_payload_fails_for_human_readable() {
        writing_invalid_fixed_size_payload_fails(
            DataRepresentation::HumanReadable,
            MessagingPattern::RequestResponse,
        );
    }

    fn writing_invalid_dynamic_payload_fails(
        data_representation: DataRepresentation,
        messaging_pattern: MessagingPattern,
    ) {
        let service_name = iceoryx2::testing::generate_service_name();
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::Dynamic, 32, 4),
            user_header: generate_type_detail(TypeVariant::FixedSize, 64, 8),
            system_header: generate_type_detail(TypeVariant::FixedSize, 128, 64),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .messaging_pattern(messaging_pattern)
            .create(&file_name, &service_name)
            .unwrap();

        let mut data = generate_service_data(&types, Duration::ZERO);
        data.payload = generate_data(types.payload.size() + 1);

        assert_that!(
            recorder.write(RawRecord {
                timestamp: data.timestamp,
                system_header: &data.system_header,
                user_header: &data.user_header,
                payload: &data.payload
            }),
            eq Err(RecorderWriteError::CorruptedPayloadRecord)
        );

        let replay = ReplayerOpener::new(&file_name)
            .data_representation(data_representation)
            .open()
            .unwrap();
        let record_header = replay.header().clone();
        let buffer = replay.read_into_buffer().unwrap();

        assert_that!(record_header, eq * recorder.header());
        assert_that!(buffer, len 0);

        File::remove(&file_name).unwrap();
    }

    #[test]
    fn writing_invalid_dynamic_payload_fails_for_iox2dump() {
        writing_invalid_dynamic_payload_fails(
            DataRepresentation::Iox2Dump,
            MessagingPattern::PublishSubscribe,
        );
    }

    #[test]
    fn writing_invalid_dynamic_payload_fails_for_human_readable() {
        writing_invalid_dynamic_payload_fails(
            DataRepresentation::HumanReadable,
            MessagingPattern::RequestResponse,
        );
    }

    fn writing_invalid_user_header_fails(
        data_representation: DataRepresentation,
        messaging_pattern: MessagingPattern,
        user_header_type: TypeDetail,
    ) {
        let service_name = iceoryx2::testing::generate_service_name();
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::Dynamic, 32, 4),
            user_header: user_header_type,
            system_header: generate_type_detail(TypeVariant::FixedSize, 128, 64),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .messaging_pattern(messaging_pattern)
            .create(&file_name, &service_name)
            .unwrap();

        let mut data = generate_service_data(&types, Duration::ZERO);
        data.user_header = generate_data(types.user_header.size() + 1);

        assert_that!(
            recorder.write(RawRecord {
                timestamp: data.timestamp,
                system_header: &data.system_header,
                user_header: &data.user_header,
                payload: &data.payload
            }),
            eq Err(RecorderWriteError::CorruptedUserHeaderRecord)
        );

        let replay = ReplayerOpener::new(&file_name)
            .data_representation(data_representation)
            .open()
            .unwrap();
        let record_header = replay.header().clone();
        let buffer = replay.read_into_buffer().unwrap();

        assert_that!(record_header, eq * recorder.header());
        assert_that!(buffer, len 0);

        File::remove(&file_name).unwrap();
    }

    #[test]
    fn writing_invalid_user_header_fails_for_iox2dump() {
        writing_invalid_user_header_fails(
            DataRepresentation::Iox2Dump,
            MessagingPattern::PublishSubscribe,
            generate_type_detail(TypeVariant::FixedSize, 8, 2),
        );
    }

    #[test]
    fn writing_invalid_user_header_fails_for_human_readable() {
        writing_invalid_user_header_fails(
            DataRepresentation::HumanReadable,
            MessagingPattern::RequestResponse,
            generate_type_detail(TypeVariant::FixedSize, 8, 4),
        );
    }

    #[test]
    fn writing_invalid_user_header_for_unit_type_fails_for_iox2dump() {
        writing_invalid_user_header_fails(
            DataRepresentation::Iox2Dump,
            MessagingPattern::PublishSubscribe,
            TypeDetail::new::<()>(TypeVariant::FixedSize),
        );
    }

    #[test]
    fn writing_invalid_user_header_for_unit_type_fails_for_human_readable() {
        writing_invalid_user_header_fails(
            DataRepresentation::HumanReadable,
            MessagingPattern::RequestResponse,
            TypeDetail::new::<()>(TypeVariant::FixedSize),
        );
    }

    fn writing_invalid_system_header_fails(
        data_representation: DataRepresentation,
        messaging_pattern: MessagingPattern,
    ) {
        let service_name = iceoryx2::testing::generate_service_name();
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::Dynamic, 32, 4),
            user_header: generate_type_detail(TypeVariant::Dynamic, 32, 4),
            system_header: generate_type_detail(TypeVariant::FixedSize, 128, 64),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .messaging_pattern(messaging_pattern)
            .create(&file_name, &service_name)
            .unwrap();

        let mut data = generate_service_data(&types, Duration::ZERO);
        data.system_header = generate_data(types.system_header.size() - 1);

        assert_that!(
            recorder.write(RawRecord {
                timestamp: data.timestamp,
                system_header: &data.system_header,
                user_header: &data.user_header,
                payload: &data.payload
            }),
            eq Err(RecorderWriteError::CorruptedSystemHeaderRecord)
        );

        let replay = ReplayerOpener::new(&file_name)
            .data_representation(data_representation)
            .open()
            .unwrap();

        let record_header = replay.header().clone();
        let buffer = replay.read_into_buffer().unwrap();

        assert_that!(record_header, eq * recorder.header());
        assert_that!(buffer, len 0);

        File::remove(&file_name).unwrap();
    }

    #[test]
    fn writing_invalid_system_header_fails_for_iox2dump() {
        writing_invalid_system_header_fails(
            DataRepresentation::Iox2Dump,
            MessagingPattern::PublishSubscribe,
        );
    }

    #[test]
    fn writing_invalid_system_header_fails_for_human_readable() {
        writing_invalid_system_header_fails(
            DataRepresentation::HumanReadable,
            MessagingPattern::PublishSubscribe,
        );
    }

    fn record_and_replay_by_reading_step_by_step_works(
        data_representation: DataRepresentation,
        messaging_pattern: MessagingPattern,
    ) {
        const NUMBER_OF_DATA: usize = 129;
        let service_name = iceoryx2::testing::generate_service_name();
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::FixedSize, 8, 4),
            user_header: TypeDetail::new::<()>(TypeVariant::FixedSize),
            system_header: generate_type_detail(TypeVariant::FixedSize, 16, 8),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .messaging_pattern(messaging_pattern)
            .create(&file_name, &service_name)
            .unwrap();

        let mut dataset = vec![];
        for n in 0..NUMBER_OF_DATA {
            dataset.push(generate_service_data(&types, Duration::from_millis(n as _)))
        }

        for data in &dataset {
            assert_that!(
                recorder.write(RawRecord {
                    timestamp: data.timestamp,
                    system_header: &data.system_header,
                    user_header: &data.user_header,
                    payload: &data.payload
                }),
                is_ok
            );
        }

        let mut replayer = ReplayerOpener::new(&file_name)
            .data_representation(data_representation)
            .open()
            .unwrap();

        assert_that!(replayer.header(), eq recorder.header());

        for n in 0..dataset.len() {
            let record = replayer.next_record().unwrap().unwrap();
            assert_that!(record.payload, eq dataset[n].payload);
            assert_that!(record.user_header, eq dataset[n].user_header);
            assert_that!(record.system_header, eq dataset[n].system_header);
            assert_that!(record.timestamp, eq dataset[n].timestamp);
        }

        File::remove(&file_name).unwrap();
    }

    #[test]
    fn record_and_replay_by_reading_step_by_step_works_for_iox2dump() {
        record_and_replay_by_reading_step_by_step_works(
            DataRepresentation::Iox2Dump,
            MessagingPattern::PublishSubscribe,
        );
    }

    #[test]
    fn record_and_replay_by_reading_step_by_step_works_for_human_readable() {
        record_and_replay_by_reading_step_by_step_works(
            DataRepresentation::HumanReadable,
            MessagingPattern::PublishSubscribe,
        );
    }

    fn reading_corrupted_payload_fails(
        data_representation: DataRepresentation,
        messaging_pattern: MessagingPattern,
    ) {
        let service_name = iceoryx2::testing::generate_service_name();
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::FixedSize, 8, 4),
            user_header: TypeDetail::new::<()>(TypeVariant::FixedSize),
            system_header: generate_type_detail(TypeVariant::FixedSize, 16, 8),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .messaging_pattern(messaging_pattern)
            .create(&file_name, &service_name)
            .unwrap();

        let mut data = generate_service_data(&types, Duration::ZERO);
        data.payload = generate_data(types.payload.size() + 5);

        assert_that!(
            unsafe {
                testing::recorder_write_unchecked(
                    &mut recorder,
                    RawRecord {
                        timestamp: data.timestamp,
                        system_header: &data.system_header,
                        user_header: &data.user_header,
                        payload: &data.payload,
                    },
                )
            },
            is_ok
        );

        let replay = ReplayerOpener::new(&file_name)
            .data_representation(data_representation)
            .open()
            .unwrap();
        let result = replay.read_into_buffer();

        assert_that!(result.err(), eq Some(ReplayerOpenError::CorruptedPayloadRecord));

        File::remove(&file_name).unwrap();
    }

    #[test]
    fn reading_corrupted_payload_fails_for_iox2dump() {
        reading_corrupted_payload_fails(
            DataRepresentation::Iox2Dump,
            MessagingPattern::PublishSubscribe,
        );
    }

    #[test]
    fn reading_corrupted_payload_fails_for_human_readable() {
        reading_corrupted_payload_fails(
            DataRepresentation::HumanReadable,
            MessagingPattern::PublishSubscribe,
        );
    }

    fn reading_corrupted_user_header_fails(
        data_representation: DataRepresentation,
        messaging_pattern: MessagingPattern,
    ) {
        let service_name = iceoryx2::testing::generate_service_name();
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::FixedSize, 8, 4),
            user_header: generate_type_detail(TypeVariant::FixedSize, 32, 4),
            system_header: generate_type_detail(TypeVariant::FixedSize, 16, 8),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .messaging_pattern(messaging_pattern)
            .create(&file_name, &service_name)
            .unwrap();

        let mut data = generate_service_data(&types, Duration::ZERO);
        data.user_header = generate_data(types.user_header.size() + 5);

        assert_that!(
            unsafe {
                testing::recorder_write_unchecked(
                    &mut recorder,
                    RawRecord {
                        timestamp: data.timestamp,
                        system_header: &data.system_header,
                        user_header: &data.user_header,
                        payload: &data.payload,
                    },
                )
            },
            is_ok
        );

        let replayer = ReplayerOpener::new(&file_name)
            .data_representation(data_representation)
            .open()
            .unwrap();
        let result = replayer.read_into_buffer();

        assert_that!(result.err(), eq Some(ReplayerOpenError::CorruptedUserHeaderRecord));

        File::remove(&file_name).unwrap();
    }

    #[test]
    fn reading_corrupted_user_header_fails_for_iox2dump() {
        reading_corrupted_user_header_fails(
            DataRepresentation::Iox2Dump,
            MessagingPattern::PublishSubscribe,
        );
    }

    #[test]
    fn reading_corrupted_user_header_fails_for_human_readable() {
        reading_corrupted_user_header_fails(
            DataRepresentation::HumanReadable,
            MessagingPattern::PublishSubscribe,
        );
    }

    fn reading_corrupted_system_header_fails(
        data_representation: DataRepresentation,
        messaging_pattern: MessagingPattern,
    ) {
        let service_name = iceoryx2::testing::generate_service_name();
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::FixedSize, 8, 4),
            user_header: generate_type_detail(TypeVariant::FixedSize, 32, 4),
            system_header: generate_type_detail(TypeVariant::FixedSize, 16, 8),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .messaging_pattern(messaging_pattern)
            .create(&file_name, &service_name)
            .unwrap();

        let mut data = generate_service_data(&types, Duration::ZERO);
        data.system_header = generate_data(types.system_header.size() + 5);

        assert_that!(
            unsafe {
                testing::recorder_write_unchecked(
                    &mut recorder,
                    RawRecord {
                        timestamp: data.timestamp,
                        system_header: &data.system_header,
                        user_header: &data.user_header,
                        payload: &data.payload,
                    },
                )
            },
            is_ok
        );

        let replay = ReplayerOpener::new(&file_name)
            .data_representation(data_representation)
            .open()
            .unwrap();
        let result = replay.read_into_buffer();

        assert_that!(result.err(), eq Some(ReplayerOpenError::CorruptedSystemHeaderRecord));

        File::remove(&file_name).unwrap();
    }

    #[test]
    fn reading_corrupted_system_header_fails_for_iox2dump() {
        reading_corrupted_system_header_fails(
            DataRepresentation::Iox2Dump,
            MessagingPattern::PublishSubscribe,
        );
    }

    #[test]
    fn reading_corrupted_system_header_fails_for_human_readable() {
        reading_corrupted_system_header_fails(
            DataRepresentation::HumanReadable,
            MessagingPattern::PublishSubscribe,
        );
    }

    fn reading_decreasing_timestamps_fails(data_representation: DataRepresentation) {
        let service_name = iceoryx2::testing::generate_service_name();
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::FixedSize, 8, 4),
            user_header: TypeDetail::new::<()>(TypeVariant::FixedSize),
            system_header: generate_type_detail(TypeVariant::FixedSize, 16, 8),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .create(&file_name, &service_name)
            .unwrap();

        let mut dataset = vec![];
        dataset.push(generate_service_data(&types, Duration::from_millis(5)));
        dataset.push(generate_service_data(&types, Duration::from_millis(2)));

        for data in dataset {
            assert_that!(
                unsafe {
                    testing::recorder_write_unchecked(
                        &mut recorder,
                        RawRecord {
                            timestamp: data.timestamp,
                            system_header: &data.system_header,
                            user_header: &data.user_header,
                            payload: &data.payload,
                        },
                    )
                },
                is_ok
            );
        }

        let result = ReplayerOpener::new(&file_name)
            .data_representation(data_representation)
            .open()
            .unwrap()
            .read_into_buffer();

        assert_that!(result.err(), eq Some(ReplayerOpenError::CorruptedTimeline));

        File::remove(&file_name).unwrap();
    }

    #[test]
    fn reading_decreasing_timestamps_fails_for_iox2dump() {
        reading_decreasing_timestamps_fails(DataRepresentation::Iox2Dump);
    }

    #[test]
    fn reading_decreasing_timestamps_fails_for_human_readable() {
        reading_decreasing_timestamps_fails(DataRepresentation::HumanReadable);
    }
}
