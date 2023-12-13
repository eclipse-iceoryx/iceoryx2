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

use iceoryx2_bb_container::byte_string::*;
use iceoryx2_bb_testing::assert_that;

const SUT_CAPACITY: usize = 129;
type Sut = FixedSizeByteString<SUT_CAPACITY>;

#[test]
fn fixed_size_byte_string_new_string_is_empty() {
    let mut sut = Sut::new();

    assert_that!(sut, is_empty);
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, len 0);
    assert_that!(sut.pop(), is_none);
    assert_that!(sut, eq b"");
    assert_that!(sut.as_bytes(), eq b"");
    assert_that!(sut.as_mut_bytes(), eq b"");
    assert_that!(sut.as_bytes_with_nul(), eq b"\0");
}

#[test]
fn fixed_size_byte_string_from_bytes_works() {
    let sut = Sut::from_bytes(b"bonjour world");
    assert_that!(sut, is_ok);
    let mut sut = sut.unwrap();

    assert_that!(sut, is_not_empty);
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, len 13);
    assert_that!(sut, eq b"bonjour world");
    assert_that!(sut, ne b"bonjour world! woo");
    assert_that!(sut.as_bytes(), eq b"bonjour world");
    assert_that!(sut.as_mut_bytes(), eq b"bonjour world");
    assert_that!(sut.as_bytes_with_nul(), eq b"bonjour world\0");
    assert_that!(sut.pop(), eq Some(b'd'));
}

#[test]
fn fixed_size_byte_string_from_byte_slice_works() {
    let mut sut = Sut::from(b"hello world!");

    assert_that!(sut, is_not_empty);
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, len 12);
    assert_that!(sut, eq b"hello world!");
    assert_that!(sut, ne b"hello world! woo");
    assert_that!(sut.as_bytes(), eq b"hello world!");
    assert_that!(sut.as_mut_bytes(), eq b"hello world!");
    assert_that!(sut.as_bytes_with_nul(), eq b"hello world!\0");
    assert_that!(sut.pop(), eq Some(b'!'));
}

#[test]
fn fixed_size_byte_string_capacity_is_correct() {
    let sut = Sut::new();

    assert_that!(sut.capacity(), eq SUT_CAPACITY);
}

#[test]
fn fixed_size_byte_string_clear_works() {
    let mut sut = Sut::from(b"Fuu Fuu");
    sut.clear();

    assert_that!(sut, len 0);
}

#[test]
fn fixed_size_byte_string_is_full_works() {
    let sut = FixedSizeByteString::<5>::from(b"hello");
    assert_that!(sut.is_full(), eq true);
}

#[test]
#[should_panic]
fn fixed_size_byte_string_insert_works() {
    let mut sut = Sut::new();

    assert_that!(sut.insert(0, b'a'), is_ok);
    assert_that!(sut.insert(0, b'a'), is_ok);
    assert_that!(sut.insert(1, b'b'), is_ok);
    assert_that!(sut.insert(2, b'b'), is_ok);
    assert_that!(sut.insert(3, b'.'), is_ok);
    assert_that!(sut.insert(4, b','), is_ok);
    assert_that!(sut.insert(5, b'-'), is_ok);
    assert_that!(sut.insert(5, b'X'), is_ok);
    assert_that!(sut.insert(2, b'!'), is_ok);

    assert_that!(sut, len 9);
    assert_that!(sut, eq b"ab!ba.,X-");
    assert_that!(sut.as_bytes_with_nul(), eq b"ab!ba.,X-\0");

    assert_that!(sut.insert(10, b'.'), is_err);
}

#[test]
#[should_panic]
fn fixed_size_byte_string_insert_bytes_works() {
    let mut sut = Sut::from(b"hello ");
    assert_that!(sut.insert_bytes(0, b"world "), is_ok);
    assert_that!(sut.insert_bytes(7, b"hehe"), is_ok);
    assert_that!(sut.insert_bytes(5, b"!!!"), is_ok);

    assert_that!(sut, len 20);
    assert_that!(sut, eq b"world!!! hehehello");
    assert_that!(sut.as_bytes_with_nul(), eq b"world!!! hehehello\0");

    assert_that!(sut.insert_bytes(21, b"!!!"), is_err);
}

#[test]
fn fixed_size_byte_string_pop_works() {
    let mut sut = Sut::from(b"hello");

    assert_that!(sut.pop(), eq Some(b'o'));
    assert_that!(sut, len 4);
    assert_that!(sut.as_bytes_with_nul(), eq b"hell\0");
    sut.clear();

    assert_that!(sut, len 0);
    assert_that!(sut.pop(), is_none);
    assert_that!(sut.as_bytes_with_nul(), eq b"\0");
}

#[test]
fn fixed_size_byte_string_push_works() {
    let mut sut = Sut::new();
    assert_that!(sut.push(b'b'), is_ok);
    assert_that!(sut.push(b'u'), is_ok);
    assert_that!(sut.push(b'h'), is_ok);

    assert_that!(sut, len 3);
    assert_that!(sut, eq b"buh");
    assert_that!(sut.as_bytes_with_nul(), eq b"buh\0");
}

#[test]
fn fixed_size_byte_string_push_bytes_works() {
    let mut sut = Sut::new();
    assert_that!(sut.push_bytes(b"all glory "), is_ok);
    assert_that!(sut.push_bytes(b"to the hypnotoad"), is_ok);

    assert_that!(sut, len 26);
    assert_that!(sut, eq b"all glory to the hypnotoad");
    assert_that!(sut.as_bytes_with_nul(), eq b"all glory to the hypnotoad\0");
}

#[test]
fn fixed_size_byte_string_remove_works() {
    let mut sut = Sut::from(b"hassel the hoff");

    assert_that!(sut.remove(0), eq  b'h');
    assert_that!(sut.remove(7), eq b'h');
    assert_that!(sut.remove(12), eq b'f');

    assert_that!(sut, len 12);
    assert_that!(sut, eq b"assel te hof");
    assert_that!(sut.as_bytes_with_nul(), eq b"assel te hof\0");
}

#[test]
fn fixed_size_byte_string_retain_works() {
    let mut sut = Sut::from(b"live long and nibble");

    sut.retain(|c| c == b' ');

    assert_that!(sut, len 17);
    assert_that!(sut, eq b"livelongandnibble");
    assert_that!(sut.as_bytes_with_nul(), eq b"livelongandnibble\0");
}

#[test]
fn fixed_size_byte_string_remove_range_works() {
    let mut sut = Sut::from(b"bibbe di babbe di buu");

    sut.remove_range(14, 3);
    sut.remove_range(5, 3);

    assert_that!(sut, len 15);
    assert_that!(sut, eq b"bibbe babbe buu");
    assert_that!(sut.as_bytes_with_nul(), eq b"bibbe babbe buu\0");
}

#[test]
fn fixed_size_byte_string_from_c_str_works() {
    let value = Sut::from(b"");
    let sut = unsafe { Sut::from_c_str(value.as_ptr() as *mut std::ffi::c_char).unwrap() };

    assert_that!(sut, len 0);
    assert_that!(sut, eq b"");

    let value = Sut::from(b"foo baha");
    let sut = unsafe { Sut::from_c_str(value.as_ptr() as *mut std::ffi::c_char).unwrap() };

    assert_that!(sut, len 8);
    assert_that!(sut, eq b"foo baha");
}

#[test]
fn fixed_size_byte_string_new_unchecked_works() {
    let sut = unsafe { Sut::new_unchecked(b"fuu me duu") };

    assert_that!(sut, len 10);
    assert_that!(sut, eq b"fuu me duu");
    assert_that!(sut.as_bytes_with_nul(), eq b"fuu me duu\0");

    let sut = unsafe { Sut::new_unchecked(b"") };

    assert_that!(sut, len 0);
    assert_that!(sut, eq b"");
    assert_that!(sut.as_bytes_with_nul(), eq b"\0");
}

#[test]
fn fixed_size_byte_string_truncate_works() {
    let mut sut = unsafe { Sut::new_unchecked(b"droubadix") };
    sut.truncate(4);

    assert_that!(sut, len 4);
    assert_that!(sut, eq b"drou");
    assert_that!(sut.as_bytes_with_nul(), eq b"drou\0");

    sut.truncate(6);
    assert_that!(sut, len 4);
}

#[test]
fn fixed_size_byte_string_find_works() {
    let sut = unsafe { Sut::new_unchecked(b"blubb_di:bubbx") };

    assert_that!(sut.find(b"bkasjdkasjdkasjdksjd"), is_none);

    assert_that!(sut.find(b"b"), eq Some(0));
    assert_that!(sut.find(b"blubb"), eq Some(0));
    assert_that!(sut.find(b"bb"), eq Some(3));
    assert_that!(sut.find(b"di"), eq Some(6));
    assert_that!(sut.find(b"bubbx"), eq Some(9));
    assert_that!(sut.find(b"x"), eq Some(13));

    assert_that!(sut.find(b"."), eq None);
    assert_that!(sut.find(b","), eq None);
    assert_that!(sut.find(b"-"), eq None);
}

#[test]
fn fixed_size_byte_string_rfind_works() {
    let sut = unsafe { Sut::new_unchecked(b"alubb_di:bubbx") };

    assert_that!(sut.rfind(b"bkasjdkasjdkasjdksjd"), is_none);

    assert_that!(sut.rfind(b"b"), eq Some(12));
    assert_that!(sut.rfind(b"alubb"), eq Some(0));
    assert_that!(sut.rfind(b"bb"), eq Some(11));
    assert_that!(sut.rfind(b"di"), eq Some(6));
    assert_that!(sut.rfind(b"bubbx"), eq Some(9));
    assert_that!(sut.rfind(b"x"), eq Some(13));
    assert_that!(sut.rfind(b"a"), eq Some(0));

    assert_that!(sut.rfind(b"."), eq None);
    assert_that!(sut.rfind(b","), eq None);
    assert_that!(sut.rfind(b"-"), eq None);
}

#[test]
fn fixed_size_byte_string_strip_prefix_works() {
    let sut = unsafe { Sut::new_unchecked(b"msla:0123_lerata.fuu") };

    let mut sut_clone = sut;
    assert_that!(sut_clone.strip_prefix(b"bkasjdkas120ie19jdkasjdksjd"), eq false);
    assert_that!(sut_clone, eq sut);

    let mut sut_clone = sut;
    assert_that!(sut_clone.strip_prefix(b"msla:"), eq true);
    assert_that!(sut_clone, eq b"0123_lerata.fuu");

    let mut sut_clone = sut;
    assert_that!(sut_clone.strip_prefix(b"m"), eq true);
    assert_that!(sut_clone, eq b"sla:0123_lerata.fuu");

    let mut sut_clone = sut;
    assert_that!(sut_clone.strip_prefix(b"sla"), eq false);
    assert_that!(sut_clone, eq sut);

    let mut sut_clone = sut;
    assert_that!(sut_clone.strip_prefix(b"fuu"), eq false);
    assert_that!(sut_clone, eq sut);
}

#[test]
fn fixed_size_byte_string_strip_suffix_works() {
    let sut = unsafe { Sut::new_unchecked(b"msla:0123_lerata.fuu") };

    let mut sut_clone = sut;
    assert_that!(sut_clone.strip_suffix(b"bkaslqwsd0jdkasjdkasjdksjd"), eq false);
    assert_that!(sut_clone, eq sut);

    let mut sut_clone = sut;
    assert_that!(sut_clone.strip_suffix(b".fuu"), eq true);
    assert_that!(sut_clone, eq b"msla:0123_lerata");

    let mut sut_clone = sut;
    assert_that!(sut_clone.strip_suffix(b"u"), eq true);
    assert_that!(sut_clone, eq b"msla:0123_lerata.fu");

    let mut sut_clone = sut;
    assert_that!(sut_clone.strip_suffix(b"fu"), eq false);
    assert_that!(sut_clone, eq sut);

    let mut sut_clone = sut;
    assert_that!(sut_clone.strip_suffix(b"msla"), eq false);
    assert_that!(sut_clone, eq sut);
}
