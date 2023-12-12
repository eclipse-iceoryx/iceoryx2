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

use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_system_types::file_name::*;
use iceoryx2_bb_system_types::file_path::*;
use iceoryx2_bb_system_types::group_name::*;
use iceoryx2_bb_system_types::path::*;
use iceoryx2_bb_system_types::user_name::*;

#[generic_tests::define]
mod semantic_string {
    use iceoryx2_bb_testing::assert_that;

    use super::*;

    #[test]
    fn new_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        let sut = Sut::new(b"hello-txt");
        assert_that!(sut, is_ok);
        let sut = sut.unwrap();

        assert_that!(sut, len 9);
        assert_that!(sut.as_bytes(), eq b"hello-txt");
    }

    #[test]
    fn new_name_with_illegal_char_is_illegal<
        const CAPACITY: usize,
        Sut: SemanticString<CAPACITY>,
    >() {
        let sut = Sut::new(b"hello \0.txt");
        assert_that!(sut, is_err);
    }

    #[test]
    fn insert_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        let mut sut = Sut::new(b"hello").unwrap();
        assert_that!(sut.insert(1, b't'), is_ok);

        assert_that!(sut, len 6);
        assert_that!(sut.as_bytes(), eq b"htello");
    }

    #[test]
    fn insert_illegal_character_fails<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        let mut sut = Sut::new(b"hello").unwrap();
        assert_that!(sut.insert(1, b'*'), is_err);

        assert_that!(sut, len 5);
        assert_that!(sut.as_bytes(), eq b"hello");
    }

    #[test]
    fn insert_bytes_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        let mut sut = Sut::new(b"wld").unwrap();
        assert_that!(sut.insert_bytes(1, b"or"), is_ok);

        assert_that!(sut, len 5);
        assert_that!(sut.as_bytes(), eq b"world");
    }

    #[test]
    fn insert_bytes_with_illegal_character_fails<
        const CAPACITY: usize,
        Sut: SemanticString<CAPACITY>,
    >() {
        let mut sut = Sut::new(b"wld").unwrap();
        assert_that!(sut.insert_bytes(1, b"o\0r"), is_err);

        assert_that!(sut, len 3);
        assert_that!(sut.as_bytes(), eq b"wld");
    }

    #[test]
    fn pop_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        let mut sut = Sut::new(b"fuu-blaa-fuu").unwrap();
        let result = sut.pop();
        assert_that!(result.unwrap(), eq Some(b'u'));

        assert_that!(sut, len 11);
        assert_that!(sut.as_bytes(), eq b"fuu-blaa-fu");
    }

    #[test]
    fn remove_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        let mut sut = Sut::new(b"a01234").unwrap();
        let result = sut.remove(2);
        assert_that!(result, eq Ok(b'1'));

        assert_that!(sut, len 5);
        assert_that!(sut.as_bytes(), eq b"a0234");
    }

    #[test]
    fn remove_range_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        let mut sut = Sut::new(b"a01234567").unwrap();
        let result = sut.remove_range(3, 3);
        assert_that!(result, is_ok);

        assert_that!(sut, len 6);
        assert_that!(sut.as_bytes(), eq b"a01567");
    }

    #[test]
    fn retain_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        let mut sut = Sut::new(b"a01234567").unwrap();
        let result = sut.retain(|c| c == b'4');
        assert_that!(result, is_ok);

        assert_that!(sut, len 8);
        assert_that!(sut.as_bytes(), eq b"a0123567");
    }

    #[test]
    fn strip_prefix_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        let mut sut = Sut::new(b"a0123a4567").unwrap();
        assert_that!(sut.strip_prefix(b"a0123"), eq Ok(true));
        assert_that!(sut.as_bytes(), eq b"a4567");

        assert_that!(sut.strip_prefix(b"a0123"), eq Ok(false));
        assert_that!(sut.as_bytes(), eq b"a4567");
    }

    #[test]
    fn strip_suffix_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        let mut sut = Sut::new(b"a0123a4567").unwrap();
        assert_that!(sut.strip_suffix(b"a4567"), eq Ok(true));
        assert_that!(sut.as_bytes(), eq b"a0123");

        assert_that!(sut.strip_suffix(b"a4567"), eq Ok(false));
        assert_that!(sut.as_bytes(), eq b"a0123");
    }

    #[test]
    fn truncate_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        let mut sut = Sut::new(b"a01234567").unwrap();
        assert_that!(sut.truncate(4), is_ok);

        assert_that!(sut, len 4);
        assert_that!(sut.as_bytes(), eq b"a012");

        assert_that!(sut.truncate(6), is_ok);
        assert_that!(sut, len 4);
        assert_that!(sut.as_bytes(), eq b"a012");
    }

    #[instantiate_tests(<{FileName::max_len()}, FileName>)]
    mod file_name {}

    #[instantiate_tests(<{Path::max_len()}, Path>)]
    mod path {}

    #[instantiate_tests(<{FilePath::max_len()}, FilePath>)]
    mod file_path {}

    #[instantiate_tests(<{UserName::max_len()}, UserName>)]
    mod user_name {}

    #[instantiate_tests(<{GroupName::max_len()}, GroupName>)]
    mod group_name {}
}
