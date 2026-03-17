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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_posix_tests_common::file_descriptor_tests;

#[test]
fn file_descriptor_smaller_zero_is_invalid() {
    file_descriptor_tests::file_descriptor_smaller_zero_is_invalid();
}

#[test]
fn file_descriptor_with_arbitrary_number_greater_equal_zero_is_invalid() {
    file_descriptor_tests::file_descriptor_with_arbitrary_number_greater_equal_zero_is_invalid();
}

#[cfg(test)]
#[::generic_tests::define]
mod file_descriptor_management {
    use super::*;

    use iceoryx2_bb_posix::file::File;
    use iceoryx2_bb_posix::file_descriptor::FileDescriptorManagement;
    use iceoryx2_bb_posix::shared_memory::SharedMemory;
    use iceoryx2_bb_posix_tests_common::file_descriptor_tests::GenericTestBuilder;

    #[test]
    fn file_descriptor_owner_handling_works<Sut: GenericTestBuilder + FileDescriptorManagement>() {
        file_descriptor_tests::file_descriptor_owner_handling_works::<Sut>();
    }

    #[test]
    fn file_descriptor_permission_handling_works<
        Sut: GenericTestBuilder + FileDescriptorManagement,
    >() {
        file_descriptor_tests::file_descriptor_permission_handling_works::<Sut>();
    }

    #[test]
    fn file_descriptor_metadata_handling_works<
        Sut: GenericTestBuilder + FileDescriptorManagement,
    >() {
        file_descriptor_tests::file_descriptor_metadata_handling_works::<Sut>();
    }

    #[instantiate_tests(<File>)]
    mod file {}

    #[instantiate_tests(<SharedMemory>)]
    mod shared_memory {}
}
