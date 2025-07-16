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
use iceoryx2_bb_elementary::math::ToB64;
use iceoryx2_bb_posix::config::*;
use iceoryx2_bb_posix::file::*;
use iceoryx2_bb_posix::file_descriptor::*;
use iceoryx2_bb_posix::ownership::*;
use iceoryx2_bb_posix::shared_memory::*;
use iceoryx2_bb_posix::testing::create_test_directory;
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_bb_posix::user::*;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::test_requires;
use iceoryx2_pal_posix::posix::{POSIX_SUPPORT_PERMISSIONS, POSIX_SUPPORT_USERS_AND_GROUPS};

#[test]
fn file_descriptor_smaller_zero_is_invalid() {
    assert_that!(FileDescriptor::new(-12), eq None);
}

#[test]
fn file_descriptor_with_arbitrary_number_greater_equal_zero_is_invalid() {
    assert_that!(FileDescriptor::new(431), is_none);
    assert_that!(FileDescriptor::new(981), is_none);
}

fn generate_name() -> FileName {
    let mut dir = FileName::new(b"test_").unwrap();
    dir.push_bytes(UniqueSystemId::new().unwrap().value().to_b64().as_bytes())
        .unwrap();
    dir
}

trait GenericTestBuilder {
    fn sut() -> Self;
}

impl GenericTestBuilder for File {
    fn sut() -> Self {
        let name = FilePath::from_path_and_file(&test_directory(), &generate_name()).unwrap();

        let file_content = [170u8; 2048];

        create_test_directory();
        let mut file = FileBuilder::new(&name)
            .creation_mode(CreationMode::PurgeAndCreate)
            .create()
            .unwrap();
        file.write(&file_content).unwrap();
        file
    }
}

impl GenericTestBuilder for SharedMemory {
    fn sut() -> Self {
        let name = FileName::new(generate_name().as_bytes()).unwrap();
        SharedMemoryBuilder::new(&name)
            .creation_mode(CreationMode::PurgeAndCreate)
            .size(2048)
            .create()
            .unwrap()
    }
}

#[cfg(test)]
#[::generic_tests::define]
mod file_descriptor_management {
    use super::*;

    #[test]
    fn owner_handling_works<Sut: GenericTestBuilder + FileDescriptorManagement>() {
        test_requires!(POSIX_SUPPORT_USERS_AND_GROUPS);

        create_test_directory();

        let mut sut = Sut::sut();

        let me = User::from_self().unwrap();
        let uid = me.uid();
        let gid = me.details().unwrap().gid();

        assert_that!(
            sut.set_ownership(OwnershipBuilder::new().uid(uid).gid(gid).create()),
            is_ok
        );

        let ownership = sut.ownership().unwrap();
        assert_that!(ownership.uid(), eq uid);
        assert_that!(ownership.gid(), eq gid);
    }

    #[test]
    fn permission_handling_works<Sut: GenericTestBuilder + FileDescriptorManagement>() {
        test_requires!(POSIX_SUPPORT_PERMISSIONS);

        create_test_directory();

        let mut sut = Sut::sut();

        let rw_all = Permission::OWNER_READ
            | Permission::OWNER_WRITE
            | Permission::GROUP_READ
            | Permission::GROUP_WRITE
            | Permission::OTHERS_READ
            | Permission::OTHERS_WRITE;

        assert_that!(sut.set_permission(rw_all), is_ok);
        let permission = sut.permission().unwrap();
        assert_that!(permission, eq rw_all);
    }

    #[test]
    fn metadata_handling_works<Sut: GenericTestBuilder + FileDescriptorManagement>() {
        test_requires!(POSIX_SUPPORT_PERMISSIONS);

        create_test_directory();

        let mut sut = Sut::sut();

        let mut test = |perms| {
            sut.set_permission(perms).unwrap();
            let metadata = sut.metadata().unwrap();

            assert_that!(metadata.size(), eq 2048);
            assert_that!(metadata.permission(), eq perms);
        };

        test(Permission::OWNER_READ | Permission::OWNER_WRITE);
        test(Permission::OWNER_READ);
        test(Permission::OWNER_WRITE);

        test(Permission::GROUP_READ | Permission::GROUP_WRITE);
        test(Permission::GROUP_READ);
        test(Permission::GROUP_WRITE);

        test(Permission::OTHERS_READ | Permission::OTHERS_WRITE);
        test(Permission::OTHERS_READ);
        test(Permission::OTHERS_WRITE);

        test(
            Permission::OWNER_READ
                | Permission::OWNER_WRITE
                | Permission::GROUP_READ
                | Permission::GROUP_WRITE
                | Permission::OTHERS_READ
                | Permission::OTHERS_WRITE,
        );
    }

    #[instantiate_tests(<File>)]
    mod file {}

    #[instantiate_tests(<SharedMemory>)]
    mod shared_memory {}
}
