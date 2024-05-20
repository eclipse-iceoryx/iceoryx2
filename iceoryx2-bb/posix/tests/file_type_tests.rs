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

use iceoryx2_bb_posix::file_type::*;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_pal_posix::*;

#[test]
fn file_type_mode_t_conversion_works() {
    assert_that!(FileType::File, eq FileType::from_mode_t(posix::S_IFREG));
    assert_that!(FileType::Character, eq FileType::from_mode_t(posix::S_IFCHR));
    assert_that!(FileType::Block, eq FileType::from_mode_t(posix::S_IFBLK));
    assert_that!(FileType::Directory, eq FileType::from_mode_t(posix::S_IFDIR));
    assert_that!(
        FileType::SymbolicLink, eq
        FileType::from_mode_t(posix::S_IFLNK)
    );
    assert_that!(FileType::Socket, eq FileType::from_mode_t(posix::S_IFSOCK));
    assert_that!(FileType::FiFo, eq FileType::from_mode_t(posix::S_IFIFO));
    assert_that!(FileType::Unknown, eq FileType::from_mode_t(337));
}
