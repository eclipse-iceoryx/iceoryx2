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

mod fixed_size_byte_string {
    use core::ops::DerefMut;
    use std::hash::{Hash, Hasher};

    use iceoryx2_bb_container::byte_string::*;
    use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
    use iceoryx2_bb_testing::{assert_that, memory::RawMemory};
    use serde_test::{assert_tokens, Token};
    use std::collections::hash_map::DefaultHasher;

    const SUT_CAPACITY: usize = 129;
    const SUT_CAPACITY_ALT: usize = 65;
    type Sut = FixedSizeByteString<SUT_CAPACITY>;
    type SutAlt = FixedSizeByteString<SUT_CAPACITY_ALT>;

    #[test]
    fn new_string_is_empty() {
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
    fn from_bytes_works() {
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
    fn from_bytes_truncated_works() {
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
    fn from_byte_slice_works() {
        let sut = FixedSizeByteString::<5>::from_bytes_truncated(b"hell");

        assert_that!(sut, is_not_empty);
        assert_that!(sut.is_full(), eq false);
        assert_that!(sut, len 4);
        assert_that!(sut, eq b"hell");
        assert_that!(sut.as_bytes_with_nul(), eq b"hell\0");

        let sut = FixedSizeByteString::<5>::from_bytes_truncated(b"hello world");

        assert_that!(sut, is_not_empty);
        assert_that!(sut.is_full(), eq true);
        assert_that!(sut, len 5);
        assert_that!(sut, eq b"hello");
        assert_that!(sut.as_bytes_with_nul(), eq b"hello\0");
    }

    #[test]
    fn capacity_is_correct() {
        assert_that!(Sut::capacity(), eq SUT_CAPACITY);
    }

    #[test]
    fn clear_works() {
        let mut sut = Sut::from(b"Fuu Fuu");
        sut.clear();

        assert_that!(sut, len 0);
    }

    #[test]
    fn is_full_works() {
        let sut = FixedSizeByteString::<5>::from(b"hello");
        assert_that!(sut.is_full(), eq true);
    }

    #[test]
    #[should_panic]
    fn insert_works() {
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
    fn insert_bytes_works() {
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
    fn pop_works() {
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
    fn push_works() {
        let mut sut = Sut::new();
        assert_that!(sut.push(b'b'), is_ok);
        assert_that!(sut.push(b'u'), is_ok);
        assert_that!(sut.push(b'h'), is_ok);

        assert_that!(sut, len 3);
        assert_that!(sut, eq b"buh");
        assert_that!(sut.as_bytes_with_nul(), eq b"buh\0");
    }

    #[test]
    fn push_bytes_works() {
        let mut sut = Sut::new();
        assert_that!(sut.push_bytes(b"all glory "), is_ok);
        assert_that!(sut.push_bytes(b"to the hypnotoad"), is_ok);

        assert_that!(sut, len 26);
        assert_that!(sut, eq b"all glory to the hypnotoad");
        assert_that!(sut.as_bytes_with_nul(), eq b"all glory to the hypnotoad\0");
    }

    #[test]
    fn remove_works() {
        let mut sut = Sut::from(b"hassel the hoff");

        assert_that!(sut.remove(0), eq  b'h');
        assert_that!(sut.remove(7), eq b'h');
        assert_that!(sut.remove(12), eq b'f');

        assert_that!(sut, len 12);
        assert_that!(sut, eq b"assel te hof");
        assert_that!(sut.as_bytes_with_nul(), eq b"assel te hof\0");
    }

    #[test]
    fn retain_works() {
        let mut sut = Sut::from(b"live long and nibble");

        sut.retain(|c| c == b' ');

        assert_that!(sut, len 17);
        assert_that!(sut, eq b"livelongandnibble");
        assert_that!(sut.as_bytes_with_nul(), eq b"livelongandnibble\0");
    }

    #[test]
    fn remove_range_works() {
        let mut sut = Sut::from(b"bibbe di babbe di buu");

        sut.remove_range(14, 3);
        sut.remove_range(5, 3);

        assert_that!(sut, len 15);
        assert_that!(sut, eq b"bibbe babbe buu");
        assert_that!(sut.as_bytes_with_nul(), eq b"bibbe babbe buu\0");
    }

    #[test]
    fn from_c_str_works() {
        let value = Sut::from(b"");
        let sut = unsafe { Sut::from_c_str(value.as_ptr() as *mut core::ffi::c_char).unwrap() };

        assert_that!(sut, len 0);
        assert_that!(sut, eq b"");

        let value = Sut::from(b"foo baha");
        let sut = unsafe { Sut::from_c_str(value.as_ptr() as *mut core::ffi::c_char).unwrap() };

        assert_that!(sut, len 8);
        assert_that!(sut, eq b"foo baha");
    }

    #[test]
    fn new_unchecked_works() {
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
    fn truncate_works() {
        let mut sut = unsafe { Sut::new_unchecked(b"droubadix") };
        sut.truncate(4);

        assert_that!(sut, len 4);
        assert_that!(sut, eq b"drou");
        assert_that!(sut.as_bytes_with_nul(), eq b"drou\0");

        sut.truncate(6);
        assert_that!(sut, len 4);
    }

    #[test]
    fn find_works() {
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
    fn rfind_works() {
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
    fn strip_prefix_works() {
        let sut = unsafe { Sut::new_unchecked(b"msla:0123_lerata.fuu") };

        let mut sut_clone = sut.clone();
        assert_that!(sut_clone.strip_prefix(b"bkasjdkas120ie19jdkasjdksjd"), eq false);
        assert_that!(sut_clone, eq sut);

        let mut sut_clone = sut.clone();
        assert_that!(sut_clone.strip_prefix(b"msla:"), eq true);
        assert_that!(sut_clone, eq b"0123_lerata.fuu");

        let mut sut_clone = sut.clone();
        assert_that!(sut_clone.strip_prefix(b"m"), eq true);
        assert_that!(sut_clone, eq b"sla:0123_lerata.fuu");

        let mut sut_clone = sut.clone();
        assert_that!(sut_clone.strip_prefix(b"sla"), eq false);
        assert_that!(sut_clone, eq sut);

        let mut sut_clone = sut.clone();
        assert_that!(sut_clone.strip_prefix(b"fuu"), eq false);
        assert_that!(sut_clone, eq sut);
    }

    #[test]
    fn strip_suffix_works() {
        let sut = unsafe { Sut::new_unchecked(b"msla:0123_lerata.fuu") };

        let mut sut_clone = sut.clone();
        assert_that!(sut_clone.strip_suffix(b"bkaslqwsd0jdkasjdkasjdksjd"), eq false);
        assert_that!(sut_clone, eq sut);

        let mut sut_clone = sut.clone();
        assert_that!(sut_clone.strip_suffix(b".fuu"), eq true);
        assert_that!(sut_clone, eq b"msla:0123_lerata");

        let mut sut_clone = sut.clone();
        assert_that!(sut_clone.strip_suffix(b"u"), eq true);
        assert_that!(sut_clone, eq b"msla:0123_lerata.fu");

        let mut sut_clone = sut.clone();
        assert_that!(sut_clone.strip_suffix(b"fu"), eq false);
        assert_that!(sut_clone, eq sut);

        let mut sut_clone = sut.clone();
        assert_that!(sut_clone.strip_suffix(b"msla"), eq false);
        assert_that!(sut_clone, eq sut);
    }

    #[test]
    fn ordering_works() {
        unsafe {
            assert_that!(Sut::new_unchecked(b"fuubla").cmp(&Sut::new_unchecked(b"fuubla")), eq core::cmp::Ordering::Equal );
            assert_that!(Sut::new_unchecked(b"fuubla").cmp(&Sut::new_unchecked(b"fuvbla")), eq core::cmp::Ordering::Less );
            assert_that!(Sut::new_unchecked(b"fuubla").cmp(&Sut::new_unchecked(b"fuubaa")), eq core::cmp::Ordering::Greater );
            assert_that!(Sut::new_unchecked(b"fuubla").cmp(&Sut::new_unchecked(b"fuubla123")), eq core::cmp::Ordering::Less );
            assert_that!(Sut::new_unchecked(b"fuubla").cmp(&Sut::new_unchecked(b"fuu")), eq core::cmp::Ordering::Greater );
        }
    }

    #[test]
    fn partial_ordering_works() {
        unsafe {
            assert_that!(SutAlt::new_unchecked(b"darth_fuubla").partial_cmp(&Sut::new_unchecked(b"darth_fuubla")), eq Some(core::cmp::Ordering::Equal ));
            assert_that!(SutAlt::new_unchecked(b"darth_fuubla").partial_cmp(&Sut::new_unchecked(b"darth_fuvbla")), eq Some(core::cmp::Ordering::Less ));
            assert_that!(SutAlt::new_unchecked(b"darth_fuubla").partial_cmp(&Sut::new_unchecked(b"darth_fuubaa")), eq Some(core::cmp::Ordering::Greater ));
            assert_that!(SutAlt::new_unchecked(b"darth_fuubla").partial_cmp(&Sut::new_unchecked(b"darth_fuubla123")), eq Some(core::cmp::Ordering::Less ));
            assert_that!(SutAlt::new_unchecked(b"darth_fuubla").partial_cmp(&Sut::new_unchecked(b"darth_fuu")), eq Some(core::cmp::Ordering::Greater ));
        }
    }

    #[test]
    fn strnlen_returns_max_without_null_terminator() {
        let max_len = 4;
        let some_string = "hello world";
        assert_that!(unsafe { strnlen(some_string.as_ptr().cast(), max_len) }, eq max_len);
    }

    #[test]
    fn error_display_works() {
        assert_that!(format!("{}", FixedSizeByteStringModificationError::InsertWouldExceedCapacity), eq "FixedSizeByteStringModificationError::InsertWouldExceedCapacity");
    }

    #[test]
    fn hash_works() {
        let sut_1 = Sut::from_bytes_truncated(b"hypnotoad forever");
        let sut_1_1 = Sut::from_bytes_truncated(b"hypnotoad forever");
        let sut_2 = Sut::from_bytes_truncated(b"the hoff rocks");

        let mut hasher_1 = DefaultHasher::new();
        let mut hasher_1_1 = DefaultHasher::new();
        let mut hasher_2 = DefaultHasher::new();

        sut_1.hash(&mut hasher_1);
        let hash_1 = hasher_1.finish();
        sut_1_1.hash(&mut hasher_1_1);
        let hash_1_1 = hasher_1_1.finish();
        sut_2.hash(&mut hasher_2);
        let hash_2 = hasher_2.finish();

        assert_that!(hash_1, eq hash_1_1);
        assert_that!(hash_1, ne hash_2);
    }

    #[test]
    fn deref_mut_works() {
        let mut sut = Sut::from_bytes_truncated(b"hello");
        sut.deref_mut()[0] = b'b';

        assert_that!(sut, eq b"bello");
    }

    #[test]
    fn str_slice_equality_works() {
        let hello = b"funzel";
        let sut = Sut::from_bytes_truncated(b"funzel");

        assert_that!(sut == hello.as_slice(), eq true);
    }

    #[test]
    #[should_panic]
    fn from_panics_when_capacity_is_exceeded() {
        let _ = FixedSizeByteString::<2>::from(b"hello");
    }

    #[test]
    fn default_string_is_empty() {
        assert_that!(Sut::default(), is_empty);
    }

    #[test]
    fn as_escaped_string_works() {
        assert_that!(as_escaped_string(b"\\t"), eq "\\t");
        assert_that!(as_escaped_string(b"\\r"), eq "\\r");
        assert_that!(as_escaped_string(b"\\n"), eq "\\n");
        assert_that!(as_escaped_string(b"\x20"), eq " ");
        assert_that!(as_escaped_string(b"\x7e"), eq "~");
        assert_that!(as_escaped_string(b"\x01"), eq "\\x01");
    }

    #[test]
    #[should_panic]
    fn new_unchecked_panics_when_capacity_is_exceeded() {
        let _ = unsafe { FixedSizeByteString::<3>::new_unchecked(b"12345") };
    }

    #[test]
    fn from_bytes_fails_when_capacity_is_exceeded() {
        let sut = FixedSizeByteString::<3>::from_bytes(b"12345");
        assert_that!(sut, is_err);
        assert_that!(
            sut.err().unwrap(), eq
            FixedSizeByteStringModificationError::InsertWouldExceedCapacity
        );
    }

    #[test]
    fn from_c_str_fails_when_capacity_is_exceeded() {
        let content = b"i like chocolate in my noodlesoup";
        let sut = unsafe { FixedSizeByteString::<5>::from_c_str(content.as_ptr().cast()) };
        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq FixedSizeByteStringModificationError::InsertWouldExceedCapacity);
    }

    #[test]
    #[should_panic]
    fn insert_at_out_of_bounds_index_panics() {
        let mut sut = Sut::from_bytes_truncated(b"the hoff rocks");
        let _ = sut.insert_bytes(123, b"but what about hypnotoad");
    }

    #[test]
    fn insert_value_exceeding_capacity_fails() {
        let mut sut = FixedSizeByteString::<10>::from_bytes_truncated(b"lakirski");
        let result = sut.insert_bytes(8, b" materialski");
        assert_that!(result, is_err);
        assert_that!(
            result.err().unwrap(), eq
            FixedSizeByteStringModificationError::InsertWouldExceedCapacity
        );
    }

    #[test]
    #[should_panic]
    fn remove_out_of_bounds_index_panics() {
        let mut sut = Sut::from_bytes_truncated(b"Hypnotoad loves accounting and book keeping!");
        sut.remove(90);
    }

    #[test]
    #[should_panic]
    fn remove_range_out_of_bounds_index_panics() {
        let mut sut = Sut::from_bytes_truncated(b"Who ate the last unicorn?");
        sut.remove_range(48, 12);
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
    fn serialization_works() {
        let content = "Brother Hypnotoad is starring at you.";
        let sut = Sut::from_bytes_truncated(content.as_bytes());

        assert_tokens(&sut, &[Token::Str(content)]);
    }
}
