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
use iceoryx2_bb_system_types::base64url::*;
use iceoryx2_bb_system_types::file_name::*;
use iceoryx2_bb_system_types::file_path::*;
use iceoryx2_bb_system_types::group_name::*;
use iceoryx2_bb_system_types::path::*;
use iceoryx2_bb_system_types::user_name::*;
use iceoryx2_bb_testing::assert_that;

#[test]
fn display_error_enum_works() {
    assert_that!(format!("{}", SemanticStringError::InvalidContent), eq "SemanticStringError::InvalidContent");
    assert_that!(format!("{}", SemanticStringError::ExceedsMaximumLength), eq "SemanticStringError::ExceedsMaximumLength");
}

#[generic_tests::define]
mod semantic_string {

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
    fn try_from_legal_str_succeeds<
        const CAPACITY: usize,
        Sut: SemanticString<CAPACITY> + TryFrom<&'static str>,
    >() {
        let sut = Sut::try_from("woohoo-md");
        assert_that!(sut, is_ok);
        let sut = unsafe { sut.unwrap_unchecked() };

        assert_that!(sut, len 9);
        assert_that!(sut.as_bytes(), eq b"woohoo-md");
    }

    #[test]
    fn try_from_illegal_str_fails<
        const CAPACITY: usize,
        Sut: SemanticString<CAPACITY> + TryFrom<&'static str>,
    >() {
        let sut = Sut::try_from("oh zero \0.txt");
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

        let result = sut.strip_prefix(b"a45");
        if result.is_ok() {
            assert_that!(sut.as_bytes(), eq b"67");
        } else {
            assert_that!(result.err().unwrap(), eq SemanticStringError::InvalidContent);
        }
    }

    #[test]
    fn strip_suffix_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        let mut sut = Sut::new(b"a0123a4567").unwrap();
        assert_that!(sut.strip_suffix(b"a4567"), eq Ok(true));
        assert_that!(sut.as_bytes(), eq b"a0123");

        assert_that!(sut.strip_suffix(b"a4567"), eq Ok(false));
        assert_that!(sut.as_bytes(), eq b"a0123");

        let result = sut.strip_suffix(b"a0123");
        if result.is_ok() {
            assert_that!(sut.as_bytes(), eq b"");
        } else {
            assert_that!(result.err().unwrap(), eq SemanticStringError::InvalidContent);
        }
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

        let result = sut.truncate(0);
        if result.is_ok() {
            assert_that!(sut, is_empty);
        } else {
            assert_that!(result.err().unwrap(), eq SemanticStringError::InvalidContent);
        }
    }

    #[test]
    fn invalid_utf8_characters_fail<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        let sut = Sut::new(&[b'a', b'b', 0xdf, 0xff]);
        assert_that!(sut, is_err);

        let sut = Sut::new(&[b'f', b'u', b'u', 0xff, 0xff]);
        assert_that!(sut, is_err);
    }

    #[test]
    fn is_full_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        let sut = Sut::new(b"a01234567").unwrap();
        assert_that!(sut.is_full(), eq false);
    }

    #[test]
    fn capacity_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        let sut = Sut::new(b"a01234567").unwrap();
        assert_that!(sut.capacity(), eq CAPACITY);
    }

    #[test]
    fn insert_too_much_bytes_fails<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        let mut sut = Sut::new(b"a01234567").unwrap();
        let mut bytes = vec![];
        for _ in 0..8192 {
            bytes.push(b'a')
        }

        let result = sut.insert_bytes(0, &bytes);
        assert_that!(result, is_err);
        assert_that!(
            result.err().unwrap(), eq
            SemanticStringError::ExceedsMaximumLength
        );
    }

    #[test]
    fn pop_until_empty_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        let mut sut = Sut::new(b"aaa").unwrap();

        let mut do_pop = || {
            let result = sut.pop();
            if result.is_ok() {
                assert_that!(result.unwrap().unwrap(), eq b'a');
            } else {
                assert_that!(result.err().unwrap(), eq SemanticStringError::InvalidContent);
            }
        };

        do_pop();
        do_pop();
        do_pop();

        if sut.is_empty() {
            assert_that!(sut.pop().unwrap(), eq None);
        }
    }

    #[instantiate_tests(<{FileName::max_len()}, FileName>)]
    mod file_name {}

    #[instantiate_tests(<64, RestrictedFileName::<64>>)]
    mod restricted_file_name {}

    #[instantiate_tests(<{Path::max_len()}, Path>)]
    mod path {}

    #[instantiate_tests(<{FilePath::max_len()}, FilePath>)]
    mod file_path {}

    #[instantiate_tests(<{UserName::max_len()}, UserName>)]
    mod user_name {}

    #[instantiate_tests(<{GroupName::max_len()}, GroupName>)]
    mod group_name {}

    #[instantiate_tests(<{Base64Url::max_len()}, Base64Url>)]
    mod base64url {}
}
