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

use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_elementary::unique_id::*;
use iceoryx2_bb_posix::config;
use iceoryx2_bb_posix::file::*;
use iceoryx2_bb_posix::file_descriptor::*;
use iceoryx2_bb_posix::group::Gid;
use iceoryx2_bb_posix::process::ProcessId;
use iceoryx2_bb_posix::socket_ancillary::*;
use iceoryx2_bb_posix::testing::create_test_directory;
use iceoryx2_bb_posix::user::Uid;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::test_requires;
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS_ANCILLARY_DATA;

fn generate_file_name() -> FilePath {
    let mut file = FileName::new(b"unix_datagram_socket_file_tests").unwrap();
    file.push_bytes(UniqueId::new().value().to_string().as_bytes())
        .unwrap();

    FilePath::from_path_and_file(&config::test_directory(), &file).unwrap()
}

struct TestFixture {
    files: Vec<FilePath>,
}

impl TestFixture {
    fn new() -> TestFixture {
        TestFixture { files: vec![] }
    }

    fn create_file(&mut self) -> File {
        let file_name = generate_file_name();
        let file = FileBuilder::new(&file_name)
            .creation_mode(CreationMode::PurgeAndCreate)
            .create()
            .unwrap();
        self.files.push(file_name);
        file
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        for file in &self.files {
            File::remove(file).expect("failed to cleanup test file");
        }
    }
}

#[test]
fn socket_ancillary_is_empty_when_created() {
    test_requires!(POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS_ANCILLARY_DATA);

    let sut = SocketAncillary::new();
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, is_empty);
}

#[test]
fn socket_ancillary_credentials_work() {
    test_requires!(POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS_ANCILLARY_DATA);

    let mut sut = SocketAncillary::new();

    let mut credentials = SocketCred::new();
    credentials.set_pid(ProcessId::new(123));
    credentials.set_uid(Uid::new_from_native(456));
    credentials.set_gid(Gid::new_from_native(789));
    sut.set_creds(&credentials);
    assert_that!(sut.get_creds(), eq Some(credentials));
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, is_not_empty);

    credentials.set_pid(ProcessId::new(999));
    credentials.set_uid(Uid::new_from_native(888));
    credentials.set_gid(Gid::new_from_native(777));
    sut.set_creds(&credentials);
    assert_that!(sut.get_creds(), eq Some(credentials));
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, is_not_empty);

    sut.clear();
    assert_that!(sut.get_creds(), eq None);
}

#[test]
fn socket_ancillary_add_file_descriptors_work() {
    test_requires!(POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS_ANCILLARY_DATA);

    let mut test = TestFixture::new();
    let mut sut = SocketAncillary::new();

    create_test_directory();
    for _i in 0..MAX_FILE_DESCRIPTORS_PER_MESSAGE - 1 {
        assert_that!(sut.add_fd(test.create_file().file_descriptor().clone()), eq true);
        assert_that!(sut, is_not_empty);
        assert_that!(sut.is_full(), eq false);
    }

    assert_that!(sut.add_fd(test.create_file().file_descriptor().clone()), eq true);
    assert_that!(sut, is_not_empty);
    assert_that!(sut.is_full(), eq true);

    assert_that!(sut.add_fd(test.create_file().file_descriptor().clone()), eq false);

    sut.clear();
    assert_that!(sut, is_empty);
    assert_that!(sut.is_full(), eq false);

    assert_that!(sut.add_fd(test.create_file().file_descriptor().clone()), eq true);
}
