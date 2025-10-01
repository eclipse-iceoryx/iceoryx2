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

#[generic_tests::define]
mod string {
    use core::ops::DerefMut;
    use std::hash::{Hash, Hasher};

    use iceoryx2_bb_container::byte_string::*;
    use iceoryx2_bb_container::string::*;
    use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
    use iceoryx2_bb_testing::{assert_that, memory::RawMemory};
    use serde_test::{assert_tokens, Token};
    use std::collections::hash_map::DefaultHasher;

    const SUT_CAPACITY: usize = 129;
    const SUT_CAPACITY_ALT: usize = 65;
    type Sut = FixedSizeByteString<SUT_CAPACITY>;
    type SutAlt = FixedSizeByteString<SUT_CAPACITY_ALT>;

    trait StringTestFactory {
        type Sut: String;

        fn new() -> Self;
        fn create_sut(&self) -> Box<Self::Sut>;
    }

    struct StaticStringFactory {}

    impl StringTestFactory for StaticStringFactory {
        type Sut = StaticString<SUT_CAPACITY>;

        fn new() -> Self {
            Self {}
        }

        fn create_sut(&self) -> Box<Self::Sut> {
            Box::new(Self::Sut::new())
        }
    }

    #[test]
    fn new_string_is_empty<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        assert_that!(sut, is_empty);
        assert_that!(sut.is_full(), eq false);
        assert_that!(sut, len 0);
        assert_that!(sut.pop(), is_none);
        assert_that!(sut.as_bytes(), eq b"");
        assert_that!(sut.as_mut_bytes(), eq b"");
        assert_that!(sut.as_bytes_with_nul(), eq b"\0");
    }

    #[test]
    fn capacity_is_correct<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let sut = factory.create_sut();

        assert_that!(sut.capacity(), eq SUT_CAPACITY);
    }

    #[test]
    fn push_valid_bytes_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        assert_that!(sut.len(), eq 0);
        for byte in 1u8..128u8 {
            assert_that!(sut.push(byte), is_ok);
            assert_that!(sut.len(), eq byte as usize);
        }

        for n in 0..127 {
            assert_that!(sut[n], eq n as u8 + 1 );
        }
    }

    #[test]
    fn push_invalid_byte_fails<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        assert_that!(sut.push(0), eq Err(StringModificationError::InvalidCharacter));
        for byte in 128u8..=255u8 {
            assert_that!(sut.push(byte), eq Err(StringModificationError::InvalidCharacter));
        }
    }

    #[test]
    fn push_into_full_string_fails<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for _ in 0..SUT_CAPACITY {
            assert_that!(sut.push(12), is_ok);
        }

        assert_that!(sut.is_full(), eq true);
        assert_that!(sut.push(12), eq Err(StringModificationError::InsertWouldExceedCapacity));
    }

    #[test]
    fn as_bytes_or_str_returns_push_content<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        let mut temp = vec![];

        for n in 0..SUT_CAPACITY {
            let byte = (n as u8) % 80 + 32;
            assert_that!(sut.as_str().as_bytes(), eq temp.as_slice());
            assert_that!(sut.as_bytes(), eq temp.as_slice());
            assert_that!(sut.as_mut_bytes(), eq temp.as_slice());
            temp.push(0);
            assert_that!(sut.as_bytes_with_nul(), eq temp.as_slice());
            temp.pop();

            let c_str =
                unsafe { core::slice::from_raw_parts(sut.as_c_str().cast::<u8>(), sut.len() + 1) };
            assert_that!(c_str, eq sut.as_bytes_with_nul());

            assert_that!(sut.push(byte), is_ok);
            temp.push(byte);
        }
    }

    #[test]
    fn clear_of_empty_string_does_nothing<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        sut.clear();

        assert_that!(sut, len 0);
    }

    #[test]
    fn clear_removes_all_contents<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for n in 0..SUT_CAPACITY {
            let byte = (n as u8) % 80 + 32;
            assert_that!(sut.push(byte), is_ok);
        }

        sut.clear();

        assert_that!(sut.len(), eq 0);
        assert_that!(sut.is_empty(), eq true);
        assert_that!(sut.as_bytes(), len 0);
    }

    #[test]
    fn find_of_character_in_empty_string_returns_none<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let sut = factory.create_sut();

        for n in 0u8..u8::MAX {
            assert_that!(sut.find(&[n]), is_none);
        }
    }

    #[test]
    fn find_of_range_in_empty_string_returns_none<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let sut = factory.create_sut();

        for n in 0u8..u8::MAX {
            assert_that!(sut.find(&[n, n, n]), is_none);
        }
    }

    #[test]
    fn find_of_char_located_at_the_beginning_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        const CHAR_TO_FIND: u8 = 37;

        assert_that!(sut.push(CHAR_TO_FIND), is_ok);
        for _ in 0..SUT_CAPACITY - 1 {
            assert_that!(sut.push(44), is_ok);
        }

        assert_that!(sut.find(&[CHAR_TO_FIND]), eq Some(0));
    }

    #[test]
    fn find_of_char_located_in_the_middle_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        const CHAR_TO_FIND: u8 = 37;

        for _ in 0..(SUT_CAPACITY - 2) / 2 {
            assert_that!(sut.push(44), is_ok);
        }
        assert_that!(sut.push(CHAR_TO_FIND), is_ok);
        for _ in 0..(SUT_CAPACITY - 2) / 2 {
            assert_that!(sut.push(44), is_ok);
        }

        assert_that!(sut.find(&[CHAR_TO_FIND]), eq Some((SUT_CAPACITY - 2)/2));
    }

    #[test]
    fn find_of_char_located_at_the_end_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        const CHAR_TO_FIND: u8 = 37;

        for _ in 0..(SUT_CAPACITY - 2) / 2 {
            assert_that!(sut.push(44), is_ok);
        }
        assert_that!(sut.push(CHAR_TO_FIND), is_ok);

        assert_that!(sut.find(&[CHAR_TO_FIND]), eq Some((SUT_CAPACITY - 2)/2));
    }

    #[test]
    fn find_of_range_located_at_the_beginning_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        const RANGE_TO_FIND: [u8; 4] = [37, 38, 49, 40];

        assert_that!(sut.push_bytes(&RANGE_TO_FIND), is_ok);
        for _ in 0..(SUT_CAPACITY - 1) / 2 {
            assert_that!(sut.push(44), is_ok);
        }

        assert_that!(sut.find(&RANGE_TO_FIND), eq Some(0));
    }

    #[test]
    fn find_of_range_located_in_the_middle_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        const RANGE_TO_FIND: [u8; 4] = [37, 38, 49, 40];

        for _ in 0..(SUT_CAPACITY - 4) / 2 {
            assert_that!(sut.push(44), is_ok);
        }
        assert_that!(sut.push_bytes(&RANGE_TO_FIND), is_ok);
        for _ in 0..(SUT_CAPACITY - 4) / 2 {
            assert_that!(sut.push(44), is_ok);
        }

        assert_that!(sut.find(&RANGE_TO_FIND), eq Some((SUT_CAPACITY - 4)/2));
    }

    #[test]
    fn find_of_range_located_at_the_end_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        const RANGE_TO_FIND: [u8; 4] = [37, 38, 49, 40];

        for _ in 0..(SUT_CAPACITY - 1) / 2 {
            assert_that!(sut.push(44), is_ok);
        }
        assert_that!(sut.push_bytes(&RANGE_TO_FIND), is_ok);

        assert_that!(sut.find(&RANGE_TO_FIND), eq Some((SUT_CAPACITY - 1)/2));
    }

    #[test]
    fn insert_of_valid_character_at_the_beginning_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        let mut temp = vec![];

        for n in 1u8..128u8 {
            assert_that!(sut.insert(0, n), is_ok);
            assert_that!(sut.len(), eq n as usize);
            temp.insert(0, n);

            assert_that!(sut.as_bytes(), eq temp.as_slice());
        }
    }

    #[test]
    fn insert_of_invalid_character_fails<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        assert_that!(sut.insert(0, 0), eq Err(StringModificationError::InvalidCharacter));
        for n in 128u8..u8::MAX {
            assert_that!(sut.insert(0, n), eq Err(StringModificationError::InvalidCharacter));
        }
    }

    #[test]
    fn insert_into_full_string_fails<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for _ in 0..SUT_CAPACITY {
            assert_that!(sut.push(67), is_ok);
        }

        for idx in 0..SUT_CAPACITY {
            assert_that!(sut.insert(idx, 99), eq Err(StringModificationError::InsertWouldExceedCapacity));
        }
    }

    #[test]
    fn insert_of_valid_character_in_the_middle_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for _ in 0..SUT_CAPACITY - 1 {
            assert_that!(sut.push(67), is_ok);
        }

        assert_that!(sut.insert(SUT_CAPACITY / 2, 99,), is_ok);
        assert_that!(sut.len(), eq SUT_CAPACITY);
        assert_that!(sut.is_full(), eq true);

        for n in 0..SUT_CAPACITY {
            if n == SUT_CAPACITY / 2 {
                assert_that!(sut[n], eq 99);
            } else {
                assert_that!(sut[n], eq 67);
            }
        }
    }

    #[test]
    fn insert_of_valid_character_at_the_end_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        let mut temp = vec![];

        for n in 1u8..128u8 {
            assert_that!(sut.insert(sut.len(), n), is_ok);
            assert_that!(sut.len(), eq n as usize);
            temp.push(n);

            assert_that!(sut.as_bytes(), eq temp.as_slice());
        }
    }

    #[test]
    fn insert_bytes_at_the_start_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        let mut temp = vec![];

        for n in 0..SUT_CAPACITY as u8 / 4 {
            let bytes = vec![n + 1, n + 1, n + 1, n + 1];

            assert_that!(sut.insert_bytes(0, bytes.as_slice()), is_ok);
            for _ in 0..4 {
                temp.insert(0, n + 1);
            }

            assert_that!(sut.len(), eq(n as usize + 1) * 4);
            assert_that!(sut.as_bytes(), eq temp.as_slice());
        }
    }

    #[test]
    fn insert_bytes_in_the_middle_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        let mut temp = vec![];
        const BYTES: [u8; 4] = [33, 44, 55, 66];

        for _ in 0..SUT_CAPACITY - 4 {
            assert_that!(sut.push(34), is_ok);
            temp.push(34u8);
        }

        temp.insert(SUT_CAPACITY / 2, 66);
        temp.insert(SUT_CAPACITY / 2, 55);
        temp.insert(SUT_CAPACITY / 2, 44);
        temp.insert(SUT_CAPACITY / 2, 33);
        assert_that!(sut.insert_bytes(SUT_CAPACITY / 2, &BYTES), is_ok);

        assert_that!(sut.len(), eq SUT_CAPACITY);
        assert_that!(sut.as_bytes(), eq temp.as_slice());
    }

    #[test]
    fn insert_bytes_at_the_end_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        let mut temp = vec![];

        for n in 0..SUT_CAPACITY as u8 / 4 {
            let bytes = vec![n + 1, n + 1, n + 1, n + 1];
            assert_that!(sut.insert_bytes(sut.len(), bytes.as_slice()), is_ok);
            temp.extend_from_slice(bytes.as_slice());

            assert_that!(sut.len(), eq(n as usize + 1) * 4);
            assert_that!(sut.as_bytes(), eq temp.as_slice());
        }
    }

    #[test]
    fn insert_bytes_when_it_would_exceed_capacity_fails<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for _ in 0..SUT_CAPACITY - 2 {
            assert_that!(sut.push(24), is_ok);
        }

        assert_that!(sut.insert_bytes(0, &[1, 2, 3, 4]), eq Err(StringModificationError::InsertWouldExceedCapacity));
    }

    #[test]
    fn insert_bytes_with_invalid_characters_fails<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        assert_that!(sut.insert_bytes(0, &[1, 2, 0, 4]), eq Err(StringModificationError::InvalidCharacter));
        for n in 128u8..u8::MAX {
            assert_that!(sut.insert_bytes(0, &[1, 2, n, 4]), eq Err(StringModificationError::InvalidCharacter));
        }
    }

    #[test]
    fn insert_bytes_with_valid_characters_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for n in 1u8..128u8 {
            assert_that!(sut.insert_bytes(0, &[n]), is_ok);
        }
    }

    #[test]
    fn pop_removes_the_last_element<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for n in 1u8..128u8 {
            assert_that!(sut.push(n), is_ok);
        }

        for n in (1u8..128u8).rev() {
            assert_that!(sut.len(), eq n as usize);
            assert_that!(sut.pop(), eq Some(n));
        }

        assert_that!(sut.pop(), is_none);
    }

    /// cotninue

    #[test]
    fn remove_works<Factory: StringTestFactory>() {
        let mut sut = Sut::from(b"hassel the hoff");

        assert_that!(sut.remove(0), eq  b'h');
        assert_that!(sut.remove(7), eq b'h');
        assert_that!(sut.remove(12), eq b'f');

        assert_that!(sut, len 12);
        assert_that!(sut, eq b"assel te hof");
        assert_that!(sut.as_bytes_with_nul(), eq b"assel te hof\0");
    }

    #[test]
    fn retain_works<Factory: StringTestFactory>() {
        let mut sut = Sut::from(b"live long and nibble");

        sut.retain(|c| c == b' ');

        assert_that!(sut, len 17);
        assert_that!(sut, eq b"livelongandnibble");
        assert_that!(sut.as_bytes_with_nul(), eq b"livelongandnibble\0");
    }

    #[test]
    fn remove_range_works<Factory: StringTestFactory>() {
        let mut sut = Sut::from(b"bibbe di babbe di buu");

        sut.remove_range(14, 3);
        sut.remove_range(5, 3);

        assert_that!(sut, len 15);
        assert_that!(sut, eq b"bibbe babbe buu");
        assert_that!(sut.as_bytes_with_nul(), eq b"bibbe babbe buu\0");
    }

    #[test]
    fn truncate_works<Factory: StringTestFactory>() {
        let mut sut = unsafe { Sut::new_unchecked(b"droubadix") };
        sut.truncate(4);

        assert_that!(sut, len 4);
        assert_that!(sut, eq b"drou");
        assert_that!(sut.as_bytes_with_nul(), eq b"drou\0");

        sut.truncate(6);
        assert_that!(sut, len 4);
    }

    #[test]
    fn rfind_works<Factory: StringTestFactory>() {
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
    fn strip_prefix_works<Factory: StringTestFactory>() {
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
    fn strip_suffix_works<Factory: StringTestFactory>() {
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
    fn ordering_works<Factory: StringTestFactory>() {
        unsafe {
            assert_that!(Sut::new_unchecked(b"fuubla").cmp(&Sut::new_unchecked(b"fuubla")), eq core::cmp::Ordering::Equal );
            assert_that!(Sut::new_unchecked(b"fuubla").cmp(&Sut::new_unchecked(b"fuvbla")), eq core::cmp::Ordering::Less );
            assert_that!(Sut::new_unchecked(b"fuubla").cmp(&Sut::new_unchecked(b"fuubaa")), eq core::cmp::Ordering::Greater );
            assert_that!(Sut::new_unchecked(b"fuubla").cmp(&Sut::new_unchecked(b"fuubla123")), eq core::cmp::Ordering::Less );
            assert_that!(Sut::new_unchecked(b"fuubla").cmp(&Sut::new_unchecked(b"fuu")), eq core::cmp::Ordering::Greater );
        }
    }

    #[test]
    fn partial_ordering_works<Factory: StringTestFactory>() {
        unsafe {
            assert_that!(SutAlt::new_unchecked(b"darth_fuubla").partial_cmp(&Sut::new_unchecked(b"darth_fuubla")), eq Some(core::cmp::Ordering::Equal ));
            assert_that!(SutAlt::new_unchecked(b"darth_fuubla").partial_cmp(&Sut::new_unchecked(b"darth_fuvbla")), eq Some(core::cmp::Ordering::Less ));
            assert_that!(SutAlt::new_unchecked(b"darth_fuubla").partial_cmp(&Sut::new_unchecked(b"darth_fuubaa")), eq Some(core::cmp::Ordering::Greater ));
            assert_that!(SutAlt::new_unchecked(b"darth_fuubla").partial_cmp(&Sut::new_unchecked(b"darth_fuubla123")), eq Some(core::cmp::Ordering::Less ));
            assert_that!(SutAlt::new_unchecked(b"darth_fuubla").partial_cmp(&Sut::new_unchecked(b"darth_fuu")), eq Some(core::cmp::Ordering::Greater ));
        }
    }

    #[test]
    fn error_display_works<Factory: StringTestFactory>() {
        assert_that!(format!("{}", StringModificationError::InsertWouldExceedCapacity), eq "StringModificationError::InsertWouldExceedCapacity");
        assert_that!(format!("{}", StringModificationError::InvalidCharacter), eq "StringModificationError::InvalidCharacter");
    }

    #[test]
    fn hash_works<Factory: StringTestFactory>() {
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
    fn deref_mut_works<Factory: StringTestFactory>() {
        let mut sut = Sut::from_bytes_truncated(b"hello");
        sut.deref_mut()[0] = b'b';

        assert_that!(sut, eq b"bello");
    }

    #[test]
    fn str_slice_equality_works<Factory: StringTestFactory>() {
        let hello = b"funzel";
        let sut = Sut::from_bytes_truncated(b"funzel");

        assert_that!(sut == hello.as_slice(), eq true);
    }

    #[test]
    #[should_panic]
    fn from_panics_when_capacity_is_exceeded<Factory: StringTestFactory>() {
        let _ = FixedSizeByteString::<2>::from(b"hello");
    }

    #[test]
    fn default_string_is_empty<Factory: StringTestFactory>() {
        assert_that!(Sut::default(), is_empty);
    }

    #[test]
    #[should_panic]
    fn new_unchecked_panics_when_capacity_is_exceeded<Factory: StringTestFactory>() {
        let _ = unsafe { FixedSizeByteString::<3>::new_unchecked(b"12345") };
    }

    #[test]
    fn from_bytes_fails_when_capacity_is_exceeded<Factory: StringTestFactory>() {
        let sut = FixedSizeByteString::<3>::from_bytes(b"12345");
        assert_that!(sut, is_err);
        assert_that!(
            sut.err().unwrap(), eq
            FixedSizeByteStringModificationError::InsertWouldExceedCapacity
        );
    }

    #[test]
    fn from_c_str_fails_when_capacity_is_exceeded<Factory: StringTestFactory>() {
        let content = b"i like chocolate in my noodlesoup";
        let sut = unsafe { FixedSizeByteString::<5>::from_c_str(content.as_ptr().cast()) };
        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq FixedSizeByteStringModificationError::InsertWouldExceedCapacity);
    }

    #[test]
    #[should_panic]
    fn insert_at_out_of_bounds_index_panics<Factory: StringTestFactory>() {
        let mut sut = Sut::from_bytes_truncated(b"the hoff rocks");
        let _ = sut.insert_bytes(123, b"but what about hypnotoad");
    }

    #[test]
    fn insert_value_exceeding_capacity_fails<Factory: StringTestFactory>() {
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
    fn remove_out_of_bounds_index_panics<Factory: StringTestFactory>() {
        let mut sut = Sut::from_bytes_truncated(b"Hypnotoad loves accounting and book keeping!");
        sut.remove(90);
    }

    #[test]
    #[should_panic]
    fn remove_range_out_of_bounds_index_panics<Factory: StringTestFactory>() {
        let mut sut = Sut::from_bytes_truncated(b"Who ate the last unicorn?");
        sut.remove_range(48, 12);
    }

    #[test]
    fn placement_default_works<Factory: StringTestFactory>() {
        let mut sut = RawMemory::<Sut>::new_filled(0xff);
        unsafe { Sut::placement_default(sut.as_mut_ptr()) };
        assert_that!(unsafe {sut.assume_init()}, len 0);

        assert_that!(unsafe { sut.assume_init_mut() }.push_bytes(b"hello"), is_ok);
        assert_that!(unsafe {sut.assume_init()}.as_bytes(), eq b"hello");
    }

    #[test]
    fn serialization_works<Factory: StringTestFactory>() {
        let content = "Brother Hypnotoad is starring at you.";
        let sut = Sut::from_bytes_truncated(content.as_bytes());

        assert_tokens(&sut, &[Token::Str(content)]);
    }

    #[instantiate_tests(<StaticStringFactory>)]
    mod static_string {}
}
