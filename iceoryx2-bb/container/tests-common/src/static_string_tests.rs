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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_container::string::*;
use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
use iceoryx2_bb_testing::{assert_that, memory::RawMemory};
use serde_test::{assert_tokens, Token};
use std::str::FromStr;

const SMALL_SUT_CAPACITY: usize = 4;
const SUT_CAPACITY: usize = 129;
type Sut = StaticString<SUT_CAPACITY>;
type SmallSut = StaticString<SMALL_SUT_CAPACITY>;

#[test]
fn default_is_empty() {
    let sut = Sut::default();

    assert_that!(sut.is_empty(), eq true);
}

#[test]
fn placement_default_works() {
    let mut sut = RawMemory::<Sut>::new_filled(0xff);
    unsafe { Sut::placement_default(sut.as_mut_ptr()) };
    assert_that!(unsafe {sut.assume_init()}, len 0);

    assert_that!(unsafe { sut.assume_init_mut() }.push_bytes(b"hello"), is_ok);
    assert_that!(unsafe {sut.assume_init()}.as_bytes(), eq b"hello");
}

#[test]
fn from_bytes_unchecked_works() {
    let mut sut = unsafe { Sut::from_bytes_unchecked(b"let me be your toad") };

    assert_that!(sut, is_not_empty);
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, len 19);
    assert_that!(sut, eq b"let me be your toad");
    assert_that!(sut, ne b"let me be your toad fuu");
    assert_that!(sut.as_bytes(), eq b"let me be your toad");
    assert_that!(sut.as_mut_bytes(), eq b"let me be your toad");
    assert_that!(sut.as_bytes_with_nul(), eq b"let me be your toad\0");
    assert_that!(sut.pop(), eq Some(b'd'));
}

#[test]
fn from_bytes_unchecked_with_empty_slice_works() {
    let mut sut = unsafe { Sut::from_bytes_unchecked(b"") };

    assert_that!(sut, is_empty);
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, len 0);
    assert_that!(sut, eq b"");
    assert_that!(sut.as_bytes(), eq b"");
    assert_that!(sut.as_mut_bytes(), eq b"");
    assert_that!(sut.as_bytes_with_nul(), eq b"\0");
    assert_that!(sut.pop(), is_none);
}

#[test]
fn from_bytes_with_len_smaller_capacity_works() {
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
fn from_bytes_with_empty_slice_works() {
    let sut = Sut::from_bytes(b"");
    assert_that!(sut, is_ok);
    let mut sut = sut.unwrap();

    assert_that!(sut, is_empty);
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, len 0);
    assert_that!(sut, eq b"");
    assert_that!(sut.as_bytes(), eq b"");
    assert_that!(sut.as_mut_bytes(), eq b"");
    assert_that!(sut.as_bytes_with_nul(), eq b"\0");
    assert_that!(sut.pop(), is_none);
}

#[test]
fn from_bytes_fails_when_len_exceeds_capacity() {
    let sut = SmallSut::from_bytes(b"oooh nooo I am toooo looong");
    assert_that!(sut, eq Err(StringModificationError::InsertWouldExceedCapacity));
}

#[test]
fn from_bytes_truncated_works_with_empty_bytes() {
    let mut sut = Sut::from_bytes_truncated(b"").unwrap();

    assert_that!(sut, is_empty);
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, len 0);
    assert_that!(sut, eq b"");
    assert_that!(sut, ne b"woo");
    assert_that!(sut.as_bytes(), eq b"");
    assert_that!(sut.as_mut_bytes(), eq b"");
    assert_that!(sut.as_bytes_with_nul(), eq b"\0");
    assert_that!(sut.pop(), is_none);
}

#[test]
fn from_bytes_truncated_works_with_len_smaller_than_capacity() {
    let sut = Sut::from_bytes_truncated(b"bonjour world");
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
fn from_bytes_truncated_works_with_len_greater_than_capacity() {
    let mut sut = SmallSut::from_bytes_truncated(b"peek a boo").unwrap();

    assert_that!(sut, is_not_empty);
    assert_that!(sut.is_full(), eq true);
    assert_that!(sut, len SMALL_SUT_CAPACITY);
    assert_that!(sut, eq b"peek");
    assert_that!(sut, ne b"peek woo");
    assert_that!(sut.as_bytes(), eq b"peek");
    assert_that!(sut.as_mut_bytes(), eq b"peek");
    assert_that!(sut.as_bytes_with_nul(), eq b"peek\0");
    assert_that!(sut.pop(), eq Some(b'k'));
}

#[test]
fn from_bytes_truncated_fails_with_invalid_characters() {
    let sut = SmallSut::from_bytes_truncated(&[12, 0, 43]);
    assert_that!(sut, eq Err(StringModificationError::InvalidCharacter));
}

#[test]
fn from_str_with_len_smaller_capacity_works() {
    let mut sut = Sut::from_str("a frog sits on nalas head").unwrap();

    assert_that!(sut, is_not_empty);
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, len 25);
    assert_that!(sut, eq b"a frog sits on nalas head");
    assert_that!(sut, ne b"a frog sits on nalas foot");
    assert_that!(sut.as_bytes(), eq b"a frog sits on nalas head");
    assert_that!(sut.as_mut_bytes(), eq b"a frog sits on nalas head");
    assert_that!(sut.as_bytes_with_nul(), eq b"a frog sits on nalas head\0");
    assert_that!(sut.pop(), eq Some(b'd'));
}

#[test]
fn from_str_with_len_zero_works() {
    let mut sut = Sut::from_str("").unwrap();

    assert_that!(sut, is_empty);
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, len 0);
    assert_that!(sut, eq b"");
    assert_that!(sut, ne b"oot");
    assert_that!(sut.as_bytes(), eq b"");
    assert_that!(sut.as_mut_bytes(), eq b"");
    assert_that!(sut.as_bytes_with_nul(), eq b"\0");
    assert_that!(sut.pop(), is_none);
}

#[test]
fn from_str_with_len_greater_than_capacity_fails() {
    let sut = SmallSut::from_str("the frog jumped into oblivion");
    assert_that!(sut, eq Err(StringModificationError::InsertWouldExceedCapacity));
}

#[test]
fn from_str_truncated_with_len_smaller_capacity_works() {
    let mut sut = Sut::from_str_truncated("a butterfly sits on nalas nose").unwrap();

    assert_that!(sut, is_not_empty);
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, len 30);
    assert_that!(sut, eq b"a butterfly sits on nalas nose");
    assert_that!(sut, ne b"a butterfly sits on nalas foot");
    assert_that!(sut.as_bytes(), eq b"a butterfly sits on nalas nose");
    assert_that!(sut.as_mut_bytes(), eq b"a butterfly sits on nalas nose");
    assert_that!(sut.as_bytes_with_nul(), eq b"a butterfly sits on nalas nose\0");
    assert_that!(sut.pop(), eq Some(b'e'));
}

#[test]
fn from_str_truncated_with_len_greater_than_capacity_truncates() {
    let mut sut = SmallSut::from_str_truncated("the butterfly has a plan").unwrap();

    assert_that!(sut, is_not_empty);
    assert_that!(sut.is_full(), eq true);
    assert_that!(sut, len SMALL_SUT_CAPACITY);
    assert_that!(sut, eq b"the ");
    assert_that!(sut, ne b"foo ");
    assert_that!(sut.as_bytes(), eq b"the ");
    assert_that!(sut.as_mut_bytes(), eq b"the ");
    assert_that!(sut.as_bytes_with_nul(), eq b"the \0");
    assert_that!(sut.pop(), eq Some(b' '));
}

#[test]
fn from_str_truncated_fails_with_invalid_characters() {
    let sut = SmallSut::from_str_truncated("💩 ");
    assert_that!(sut, eq Err(StringModificationError::InvalidCharacter));
}

#[test]
fn from_c_str_works_for_empty_string() {
    let value = Sut::try_from(b"").unwrap();
    let sut = unsafe { Sut::from_c_str(value.as_ptr() as *mut core::ffi::c_char).unwrap() };

    assert_that!(sut, len 0);
    assert_that!(sut, eq b"");
}

#[test]
fn from_c_str_works_when_len_is_smaller_than_capacity() {
    let value = Sut::try_from(b"foo baha").unwrap();
    let sut = unsafe { Sut::from_c_str(value.as_ptr() as *mut core::ffi::c_char).unwrap() };

    assert_that!(sut, len 8);
    assert_that!(sut, eq b"foo baha");
}

#[test]
fn from_c_str_fails_when_len_is_greater_than_capacity() {
    let value = Sut::try_from(b"I am toooo looong again").unwrap();
    let sut = unsafe { SmallSut::from_c_str(value.as_ptr() as *mut core::ffi::c_char) };
    assert_that!(sut, eq Err(StringModificationError::InsertWouldExceedCapacity));
}

#[test]
fn eq_with_slice_works() {
    let sut = Sut::try_from(b"roky").unwrap();
    assert_that!(sut == b"roky", eq true);
    assert_that!(sut == b"rokyf", eq false);
}

#[test]
fn serialization_works() {
    let sut = SmallSut::try_from(b"bee").unwrap();

    assert_tokens(&sut, &[Token::Str("bee")]);
}

#[test]
fn try_from_str_fails_when_too_long() {
    assert_that!(SmallSut::try_from("a very loooong string"), eq Err(StringModificationError::InsertWouldExceedCapacity));
}

#[test]
fn try_from_str_fails_when_it_contains_invalid_characters() {
    assert_that!(Sut::try_from("i am illegal - 😅"), eq Err(StringModificationError::InvalidCharacter));
}

#[test]
fn try_from_str_with_valid_content_works() {
    let sut = Sut::try_from("i am a bee").unwrap();
    assert_that!(sut.as_bytes(), eq b"i am a bee");
}

#[test]
fn try_from_u8_array_fails_when_too_long() {
    assert_that!(SmallSut::try_from(b"a very loooong string"), eq Err(StringModificationError::InsertWouldExceedCapacity));
}

#[test]
fn try_from_u8_array_fails_when_it_contains_invalid_characters() {
    assert_that!(Sut::try_from(&[33,44,0,200]), eq Err(StringModificationError::InvalidCharacter));
}

#[test]
fn try_from_u8_array_with_valid_content_works() {
    let sut = Sut::try_from(b"i am a bee").unwrap();
    assert_that!(sut.as_bytes(), eq b"i am a bee");
}
