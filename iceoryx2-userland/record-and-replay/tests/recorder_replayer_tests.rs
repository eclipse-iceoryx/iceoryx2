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

use core::sync::atomic::Ordering;

use iceoryx2::service::static_config::message_type_details::{
    TypeDetail, TypeNameString, TypeVariant,
};
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

fn generate_data(len: usize) -> Vec<u8> {
    static COUNTER: IoxAtomicU8 = IoxAtomicU8::new(0);
    let mut data = vec![];

    for _ in 0..len {
        data.push(COUNTER.fetch_add(0, Ordering::Relaxed));
    }

    data
}

#[cfg(test)]
mod recorder_replayer {
    use super::*;
    use core::time::Duration;

    use iceoryx2::{
        prelude::MessagingPattern,
        service::{
            header::publish_subscribe::Header, static_config::message_type_details::TypeVariant,
        },
    };
    use iceoryx2_bb_log::{set_log_level, LogLevel};
    use iceoryx2_bb_posix::file::File;
    use iceoryx2_pal_testing::assert_that;
    use iceoryx2_userland_record_and_replay::{
        record::DataRepresentation, recorder::RecorderBuilder, replayer::ReplayerOpener,
    };

    #[test]
    fn simple_record_and_replay_works() {
        set_log_level(LogLevel::Trace);
        let file_name = generate_file_name();
        let payload_type = generate_type_detail(TypeVariant::FixedSize, 8, 4);
        let user_header_type = generate_type_detail(TypeVariant::FixedSize, 4, 1);
        let system_header_type = generate_type_detail(TypeVariant::FixedSize, 16, 8);

        let mut recorder =
            RecorderBuilder::new(&payload_type, &user_header_type, &system_header_type)
                .data_representation(DataRepresentation::Iox2Dump)
                .messaging_pattern(MessagingPattern::PublishSubscribe)
                .create(&file_name)
                .unwrap();

        let system_header = generate_data(system_header_type.size);
        let user_header = generate_data(user_header_type.size);
        let payload = generate_data(payload_type.size);

        assert_that!(
            recorder.write_payload(&system_header, &user_header, &payload, Duration::ZERO),
            is_ok
        );

        let (buffer, record_header) = ReplayerOpener::new(&file_name)
            .data_representation(DataRepresentation::Iox2Dump)
            .read_into_buffer()
            .unwrap();

        File::remove(&file_name).unwrap();
    }
}
