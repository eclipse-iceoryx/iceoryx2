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
        replayer::ReplayerOpener,
    };

    use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeNameString};
    use iceoryx2::testing;
    use iceoryx2_bb_container::semantic_string::SemanticString;
    use iceoryx2_bb_posix::config::test_directory;
    use iceoryx2_bb_posix::testing::create_test_directory;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_system_types::file_name::FileName;
    use iceoryx2_bb_system_types::file_path::FilePath;
    use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU8;

    fn generate_file_name() -> FilePath {
        create_test_directory();
        let mut file = FileName::new(b"record_replay_tests_").unwrap();
        file.push_bytes(
            UniqueSystemId::new()
                .unwrap()
                .value()
                .to_string()
                .as_bytes(),
        )
        .unwrap();

        FilePath::from_path_and_file(&test_directory(), &file).unwrap()
    }

    fn generate_type_detail(variant: TypeVariant, size: usize, alignment: usize) -> TypeDetail {
        testing::create_custom_type_detail(
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
    ) {
        const NUMBER_OF_DATA: usize = 129;
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::FixedSize, 8, 4),
            user_header: generate_type_detail(TypeVariant::FixedSize, 4, 1),
            system_header: generate_type_detail(TypeVariant::FixedSize, 16, 8),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .messaging_pattern(messaging_pattern)
            .create(&file_name)
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

        let (buffer, record_header) = ReplayerOpener::new(&file_name)
            .data_representation(data_representation)
            .read_into_buffer()
            .unwrap();

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
        );
    }

    #[test]
    fn record_and_replay_works_for_human_readable() {
        record_and_replay_works(
            DataRepresentation::HumanReadable,
            MessagingPattern::RequestResponse,
        );
    }

    fn record_and_replay_with_dynamic_payload_works(
        data_representation: DataRepresentation,
        messaging_pattern: MessagingPattern,
    ) {
        const NUMBER_OF_DATA: usize = 15;
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::Dynamic, 256, 8),
            user_header: generate_type_detail(TypeVariant::FixedSize, 8, 2),
            system_header: generate_type_detail(TypeVariant::FixedSize, 32, 4),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .messaging_pattern(messaging_pattern)
            .create(&file_name)
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

        let (buffer, record_header) = ReplayerOpener::new(&file_name)
            .data_representation(data_representation)
            .read_into_buffer()
            .unwrap();

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

    fn writing_invalid_fixed_size_payload_fails(
        data_representation: DataRepresentation,
        messaging_pattern: MessagingPattern,
    ) {
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::FixedSize, 32, 4),
            user_header: generate_type_detail(TypeVariant::FixedSize, 64, 8),
            system_header: generate_type_detail(TypeVariant::FixedSize, 128, 64),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .messaging_pattern(messaging_pattern)
            .create(&file_name)
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

        let (buffer, record_header) = ReplayerOpener::new(&file_name)
            .data_representation(data_representation)
            .read_into_buffer()
            .unwrap();

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
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::Dynamic, 32, 4),
            user_header: generate_type_detail(TypeVariant::FixedSize, 64, 8),
            system_header: generate_type_detail(TypeVariant::FixedSize, 128, 64),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .messaging_pattern(messaging_pattern)
            .create(&file_name)
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

        let (buffer, record_header) = ReplayerOpener::new(&file_name)
            .data_representation(data_representation)
            .read_into_buffer()
            .unwrap();

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
    ) {
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::Dynamic, 32, 4),
            user_header: generate_type_detail(TypeVariant::FixedSize, 4, 2),
            system_header: generate_type_detail(TypeVariant::FixedSize, 128, 64),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .messaging_pattern(messaging_pattern)
            .create(&file_name)
            .unwrap();

        let mut data = generate_service_data(&types, Duration::ZERO);
        data.user_header = generate_data(types.user_header.size() - 1);

        assert_that!(
            recorder.write(RawRecord {
                timestamp: data.timestamp,
                system_header: &data.system_header,
                user_header: &data.user_header,
                payload: &data.payload
            }),
            eq Err(RecorderWriteError::CorruptedUserHeaderRecord)
        );

        let (buffer, record_header) = ReplayerOpener::new(&file_name)
            .data_representation(data_representation)
            .read_into_buffer()
            .unwrap();

        assert_that!(record_header, eq * recorder.header());
        assert_that!(buffer, len 0);

        File::remove(&file_name).unwrap();
    }

    #[test]
    fn writing_invalid_user_header_fails_for_iox2dump() {
        writing_invalid_user_header_fails(
            DataRepresentation::Iox2Dump,
            MessagingPattern::PublishSubscribe,
        );
    }

    #[test]
    fn writing_invalid_user_header_fails_for_human_readable() {
        writing_invalid_user_header_fails(
            DataRepresentation::HumanReadable,
            MessagingPattern::RequestResponse,
        );
    }

    fn writing_invalid_user_header_for_unit_type_fails(
        data_representation: DataRepresentation,
        messaging_pattern: MessagingPattern,
    ) {
        let file_name = generate_file_name();
        let types = ServiceTypes {
            payload: generate_type_detail(TypeVariant::Dynamic, 32, 4),
            user_header: TypeDetail::new::<()>(TypeVariant::FixedSize),
            system_header: generate_type_detail(TypeVariant::FixedSize, 128, 64),
        };

        let mut recorder = RecorderBuilder::new(&types)
            .data_representation(data_representation)
            .messaging_pattern(messaging_pattern)
            .create(&file_name)
            .unwrap();

        let mut data = generate_service_data(&types, Duration::ZERO);
        data.user_header = generate_data(1);

        assert_that!(
            recorder.write(RawRecord {
                timestamp: data.timestamp,
                system_header: &data.system_header,
                user_header: &data.user_header,
                payload: &data.payload
            }),
            eq Err(RecorderWriteError::CorruptedUserHeaderRecord)
        );

        let (buffer, record_header) = ReplayerOpener::new(&file_name)
            .data_representation(data_representation)
            .read_into_buffer()
            .unwrap();

        assert_that!(record_header, eq * recorder.header());
        assert_that!(buffer, len 0);

        File::remove(&file_name).unwrap();
    }

    #[test]
    fn writing_invalid_user_header_for_unit_type_fails_for_iox2dump() {
        writing_invalid_user_header_for_unit_type_fails(
            DataRepresentation::Iox2Dump,
            MessagingPattern::PublishSubscribe,
        );
    }

    #[test]
    fn writing_invalid_user_header_for_unit_type_fails_for_human_readable() {
        writing_invalid_user_header_for_unit_type_fails(
            DataRepresentation::HumanReadable,
            MessagingPattern::RequestResponse,
        );
    }
}
