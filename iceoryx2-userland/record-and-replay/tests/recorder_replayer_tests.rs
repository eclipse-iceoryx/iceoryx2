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
        recorder::{RecorderBuilder, ServiceTypes},
        replayer::ReplayerOpener,
    };

    use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeNameString};
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
        TypeDetail {
            variant,
            type_name: TypeNameString::from_str_truncated(
                &UniqueSystemId::new().unwrap().value().to_string(),
            ),
            size,
            alignment,
        }
    }

    fn unit_type() -> TypeDetail {
        TypeDetail {
            variant: TypeVariant::FixedSize,
            type_name: TypeNameString::from_str_truncated("()"),
            size: 0,
            alignment: 1,
        }
    }

    struct Data {
        payload: Vec<u8>,
        user_header: Vec<u8>,
        system_header: Vec<u8>,
        timestamp: Duration,
    }

    fn generate_data(types: &ServiceTypes, timestamp: Duration) -> Data {
        static COUNTER: IoxAtomicU8 = IoxAtomicU8::new(0);

        let generate = |len| {
            let mut data = vec![];

            for _ in 0..len {
                data.push(COUNTER.fetch_add(0, Ordering::Relaxed));
            }
            data
        };

        Data {
            payload: generate(types.payload.size),
            user_header: generate(types.user_header.size),
            system_header: generate(types.system_header.size),
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
            dataset.push(generate_data(&types, Duration::from_millis(n as _)))
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
}
