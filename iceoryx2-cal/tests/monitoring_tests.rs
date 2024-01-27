// Copyright (c) 2024 Contributors to the Eclipse Foundation
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
mod monitoring {
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_system_types::file_name::*;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::monitoring::*;
    use iceoryx2_cal::named_concept::*;

    fn generate_name() -> FileName {
        let mut file = FileName::new(b"monitoring_tests_").unwrap();
        file.push_bytes(
            UniqueSystemId::new()
                .unwrap()
                .value()
                .to_string()
                .as_bytes(),
        )
        .unwrap();
        file
    }

    #[test]
    fn create_works<Sut: Monitoring>() {
        let name = generate_name();

        let sut_token = Sut::Builder::new(&name).create();
        assert_that!(sut_token, is_ok);
        assert_that!(*sut_token.as_ref().unwrap().name(), eq name);
    }

    #[test]
    fn create_same_token_twice_fails<Sut: Monitoring>() {
        let name = generate_name();

        let sut_token_1 = Sut::Builder::new(&name).create();
        assert_that!(sut_token_1, is_ok);

        let sut_token_2 = Sut::Builder::new(&name).create();
        assert_that!(sut_token_2, is_err);
        assert_that!(
            sut_token_2.err().unwrap(), eq
            MonitoringCreateTokenError::AlreadyExists
        );
    }

    #[test]
    fn token_removes_resources_when_going_out_of_scope<Sut: Monitoring>() {
        let name = generate_name();

        let sut_token_1 = Sut::Builder::new(&name).create();
        assert_that!(sut_token_1, is_ok);
        drop(sut_token_1);

        let sut_token_2 = Sut::Builder::new(&name).create();
        assert_that!(sut_token_2, is_ok);
    }

    #[test]
    #[cfg(not(any(target_os = "linux", target_os = "freebsd", target_os = "macos")))]
    fn monitor_works<Sut: Monitoring>() {
        let name = generate_name();

        let mut sut_monitor = Sut::Builder::new(&name).monitor().unwrap();
        assert_that!(*sut_monitor.name(), eq name);
        assert_that!(sut_monitor.state().unwrap(), eq State::DoesNotExist);

        let sut_token = Sut::Builder::new(&name).create();
        assert_that!(sut_monitor.state().unwrap(), eq State::Alive);

        drop(sut_token);
        assert_that!(sut_monitor.state().unwrap(), eq State::DoesNotExist);
    }

    #[instantiate_tests(<iceoryx2_cal::monitoring::file_lock::FileLockMonitoring>)]
    mod file_lock {}
}
