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
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::*;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_bb_testing::assert_that;

#[cfg(target_os = "windows")]
mod windows {
    use super::*;

    #[test]
    fn file_path_new_with_illegal_name_fails() {
        let sut = FilePath::new(b"C:\\some\\path\\to\\dir\\");
        assert_that!(sut, is_err);

        let sut = FilePath::new(b"C:\\weird\\relative\\path\\.");
        assert_that!(sut, is_err);

        let sut = FilePath::new(b"C:\\weird\\relative\\path\\..");
        assert_that!(sut, is_err);
    }

    #[test]
    fn file_path_new_with_legal_name_works() {
        let sut = FilePath::new(b"C:\\some\\file\\path");
        assert_that!(sut, is_ok);

        let sut = FilePath::new(b"C:\\some\\file\\p");
        assert_that!(sut, is_ok);

        let sut = FilePath::new(b"C:\\some\\file\\.p");
        assert_that!(sut, is_ok);

        let sut = FilePath::new(b"C:\\some\\file\\p.");
        assert_that!(sut, is_ok);

        let sut = FilePath::new(b"C:\\some\\file\\p..");
        assert_that!(sut, is_ok);
    }

    #[test]
    fn file_path_from_legal_path_and_file_works() {
        unsafe {
            let sut = FilePath::from_path_and_file(
                &Path::new_unchecked(b"C:\\some\\file\\path"),
                &FileName::new_unchecked(b"filename.txt"),
            );
            assert_that!(sut, is_ok);
            assert_that!(sut.unwrap(), eq b"C:\\some\\file\\path\\filename.txt");
        }

        unsafe {
            let sut = FilePath::from_path_and_file(
                &Path::new_unchecked(b"file\\path\\..\\"),
                &FileName::new_unchecked(b"filename.txt"),
            );
            assert_that!(sut, is_ok);
            assert_that!(sut.unwrap(), eq b"file\\path\\..\\filename.txt");
        }
    }

    #[test]
    fn file_path_extract_file_name_works() {
        let sut = FilePath::new(b"C:\\some\\file\\path").unwrap();
        assert_that!(sut.file_name(), eq b"path");

        let sut = FilePath::new(b"another\\path\\to\\fuuu").unwrap();
        assert_that!(sut.file_name(), eq b"fuuu");

        let sut = FilePath::new(b"\\blubbb").unwrap();
        assert_that!(sut.file_name(), eq b"blubbb");

        let sut = FilePath::new(b"another\\path\\to\\a").unwrap();
        assert_that!(sut.file_name(), eq b"a");
    }

    #[test]
    fn file_path_extract_path_works() {
        let sut = FilePath::new(b"C:\\some\\file\\path").unwrap();
        assert_that!(sut.path(), eq b"C:\\some\\file");

        let sut = FilePath::new(b"another\\path\\to\\fuuu").unwrap();
        assert_that!(sut.path(), eq b"another\\path\\to");

        let sut = FilePath::new(b"\\blubbb").unwrap();
        assert_that!(sut.path(), eq b"\\");
    }
}

#[cfg(not(target_os = "windows"))]
mod unix {
    use super::*;

    #[test]
    fn file_path_new_with_illegal_name_fails() {
        let sut = FilePath::new(b"/some/path/to/dir/");
        assert_that!(sut, is_err);

        let sut = FilePath::new(b"/weird/relative/path/.");
        assert_that!(sut, is_err);

        let sut = FilePath::new(b"/weird/relative/path/..");
        assert_that!(sut, is_err);
    }

    #[test]
    fn file_path_new_with_legal_name_works() {
        let sut = FilePath::new(b"/some/file/path");
        assert_that!(sut, is_ok);

        let sut = FilePath::new(b"/some/file/p");
        assert_that!(sut, is_ok);

        let sut = FilePath::new(b"/some/file/.p");
        assert_that!(sut, is_ok);

        let sut = FilePath::new(b"/some/file/p.");
        assert_that!(sut, is_ok);

        let sut = FilePath::new(b"/some/file/p..");
        assert_that!(sut, is_ok);
    }

    #[test]
    fn file_path_from_legal_path_and_file_works() {
        unsafe {
            let sut = FilePath::from_path_and_file(
                &Path::new_unchecked(b"/some/file/path"),
                &FileName::new_unchecked(b"filename.txt"),
            );
            assert_that!(sut, is_ok);
            assert_that!(sut.unwrap(), eq b"/some/file/path/filename.txt");
        }

        unsafe {
            let sut = FilePath::from_path_and_file(
                &Path::new_unchecked(b"file/path/../"),
                &FileName::new_unchecked(b"filename.txt"),
            );
            assert_that!(sut, is_ok);
            assert_that!(sut.unwrap(), eq b"file/path/../filename.txt");
        }
    }

    #[test]
    fn file_path_extract_file_name_works() {
        let sut = FilePath::new(b"/some/file/path").unwrap();
        assert_that!(sut.file_name(), eq b"path");

        let sut = FilePath::new(b"another/path/to/fuuu").unwrap();
        assert_that!(sut.file_name(), eq b"fuuu");

        let sut = FilePath::new(b"/blubbb").unwrap();
        assert_that!(sut.file_name(), eq b"blubbb");

        let sut = FilePath::new(b"another/path/to/a").unwrap();
        assert_that!(sut.file_name(), eq b"a");
    }

    #[test]
    fn file_path_extract_path_works() {
        let sut = FilePath::new(b"/some/file/path").unwrap();
        assert_that!(sut.path(), eq b"/some/file");

        let sut = FilePath::new(b"another/path/to/fuuu").unwrap();
        assert_that!(sut.path(), eq b"another/path/to");

        let sut = FilePath::new(b"/blubbb").unwrap();
        assert_that!(sut.path(), eq b"/");
    }
}

#[test]
fn file_path_new_with_illegal_relative_name_fails() {
    let sut = FilePath::new(b"");
    assert_that!(sut, is_err);

    let sut = FilePath::new(b".");
    assert_that!(sut, is_err);

    let sut = FilePath::new(b"..");
    assert_that!(sut, is_err);
}

#[test]
fn file_path_from_empty_path_and_legal_file_works() {
    unsafe {
        let sut = FilePath::from_path_and_file(
            &Path::new_unchecked(b""),
            &FileName::new_unchecked(b"filename.txt"),
        );
        assert_that!(sut, is_ok);
        assert_that!(sut.unwrap(), eq b"filename.txt");
    }
}

#[test]
fn file_path_extract_file_name_from_path_consisting_only_of_a_file_works() {
    let sut = FilePath::new(b"barbe").unwrap();
    assert_that!(sut.file_name(), eq b"barbe");

    let sut = FilePath::new(b"b").unwrap();
    assert_that!(sut.file_name(), eq b"b");
}

#[test]
fn file_path_extract_path_from_path_consisting_only_of_a_file_works() {
    let sut = FilePath::new(b"barbe").unwrap();
    assert_that!(sut.path(), eq b"");
}

#[test]
fn file_path_with_utf_8_content_works() {
    let mut sut = FilePath::new(b"/arghnark").unwrap();
    assert_that!(sut.insert_bytes(5, "🧐".as_bytes()), is_ok);
    assert_that!(sut.insert_bytes(13, "🧐".as_bytes()), is_ok);
    assert_that!(Into::<String>::into(&sut), eq "/argh🧐nark🧐");

    assert_that!(sut.remove(6), eq Err(SemanticStringError::InvalidContent));
    assert_that!(sut.pop(), eq Err(SemanticStringError::InvalidContent));
}
