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

use core::ops::DerefMut;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use iceoryx2_bb_concurrency::cell::UnsafeCell;
use iceoryx2_bb_container::string::{RelocatableString, *};
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
use iceoryx2_bb_testing::assert_that;
use std::collections::hash_map::DefaultHasher;

#[generic_tests::define]
mod string {
    use super::*;
    const SUT_CAPACITY: usize = 129;

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

    struct RelocatableStringFactory {
        raw_memory: UnsafeCell<Box<[u8; RelocatableString::const_memory_size(SUT_CAPACITY) * 3]>>,
        allocator: UnsafeCell<Option<Box<BumpAllocator>>>,
    }

    impl RelocatableStringFactory {
        fn allocator<'a>(&'a self) -> &'a BumpAllocator {
            unsafe {
                if (*self.allocator.get()).is_none() {
                    *self.allocator.get() = Some(Box::new(BumpAllocator::new(
                        (*self.raw_memory.get()).as_mut_ptr(),
                    )))
                }
            };

            unsafe { (*self.allocator.get()).as_ref().unwrap() }
        }
    }

    impl StringTestFactory for RelocatableStringFactory {
        type Sut = RelocatableString;

        fn new() -> Self {
            Self {
                raw_memory: UnsafeCell::new(Box::new(
                    [0u8; RelocatableString::const_memory_size(SUT_CAPACITY) * 3],
                )),
                allocator: UnsafeCell::new(None),
            }
        }

        fn create_sut(&self) -> Box<Self::Sut> {
            let mut sut = Box::new(unsafe { Self::Sut::new_uninit(SUT_CAPACITY) });
            unsafe { sut.init(self.allocator()).unwrap() };

            sut
        }
    }

    struct PolymorphicStringFactory {
        raw_memory: UnsafeCell<Box<[u8; core::mem::size_of::<u8>() * ((SUT_CAPACITY + 1) * 3)]>>,
        allocator: UnsafeCell<Option<Box<BumpAllocator>>>,
    }

    impl PolymorphicStringFactory {
        fn allocator(&self) -> &'static BumpAllocator {
            unsafe {
                if (*self.allocator.get()).is_none() {
                    *self.allocator.get() = Some(Box::new(BumpAllocator::new(
                        (*self.raw_memory.get()).as_mut_ptr(),
                    )))
                }
            };

            unsafe { (*self.allocator.get()).as_ref().unwrap() }
        }
    }

    impl StringTestFactory for PolymorphicStringFactory {
        type Sut = PolymorphicString<'static, BumpAllocator>;

        fn new() -> Self {
            Self {
                raw_memory: UnsafeCell::new(Box::new(
                    [0u8; core::mem::size_of::<u8>() * ((SUT_CAPACITY + 1) * 3)],
                )),
                allocator: UnsafeCell::new(None),
            }
        }

        fn create_sut(&self) -> Box<Self::Sut> {
            Box::new(Self::Sut::new(self.allocator(), SUT_CAPACITY).unwrap())
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
    fn find_of_non_existing_char_returns_none<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        const CHAR_TO_FIND: u8 = 37;

        for _ in 0..SUT_CAPACITY - 1 {
            assert_that!(sut.push(44), is_ok);
        }

        assert_that!(sut.find(&[CHAR_TO_FIND]), is_none);
    }

    #[test]
    fn find_returns_first_char_match_from_start<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        const CHAR_TO_FIND: u8 = 37;

        assert_that!(sut.push(44), is_ok);
        assert_that!(sut.push(CHAR_TO_FIND), is_ok);
        for _ in 0..SUT_CAPACITY - 3 {
            assert_that!(sut.push(44), is_ok);
        }
        assert_that!(sut.push(CHAR_TO_FIND), is_ok);

        assert_that!(sut.find(&[CHAR_TO_FIND]), eq Some(1));
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
    fn find_where_range_is_equal_to_sut_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        const RANGE_TO_FIND: [u8; 5] = [37, 38, 49, 40, 44];

        assert_that!(sut.push_bytes(&RANGE_TO_FIND), is_ok);

        assert_that!(sut.find(&RANGE_TO_FIND), eq Some(0));
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
            assert_that!(sut.len(), eq n as usize);
        }
    }

    #[should_panic]
    #[test]
    fn insert_bytes_out_of_bounds_panics<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        sut.insert_bytes(4, &[2]).unwrap();
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

    #[test]
    fn push_bytes_with_invalid_characters_fails<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        assert_that!(sut.push_bytes(&[1, 2, 0, 4]), eq Err(StringModificationError::InvalidCharacter));
        for n in 128u8..u8::MAX {
            assert_that!(sut.push_bytes(&[1, 2, n, 4]), eq Err(StringModificationError::InvalidCharacter));
        }
    }

    #[test]
    fn push_bytes_with_valid_characters_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        let mut temp = vec![];

        for n in 1u8..128u8 {
            temp.push(n);
            assert_that!(sut.push_bytes(&[n]), is_ok);
            assert_that!(sut.len(), eq n as usize);
            assert_that!(sut.as_bytes(), eq temp.as_slice());
        }
    }

    #[test]
    fn push_bytes_fails_when_it_exceeds_the_capacity<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for _ in 0..SUT_CAPACITY - 2 {
            assert_that!(sut.push(87), is_ok);
        }

        assert_that!(sut.push_bytes(&[33,44,55,66]), eq Err(StringModificationError::InsertWouldExceedCapacity));
    }

    #[test]
    fn push_multiple_valid_bytes_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        assert_that!(sut.push_bytes(&[33, 44, 55, 66]), is_ok);
        assert_that!(sut.len(), eq 4);
        assert_that!(sut.as_bytes(), eq & [33, 44, 55, 66]);
    }

    #[test]
    fn remove_first_character_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for n in 0..SUT_CAPACITY {
            let byte = ((n % 80) + 20) as u8;
            assert_that!(sut.push(byte), is_ok);
        }

        for n in 0..SUT_CAPACITY {
            let byte = ((n % 80) + 20) as u8;
            assert_that!(sut.len(), eq SUT_CAPACITY - n);
            assert_that!(sut.remove(0), eq Some(byte));
        }
    }

    #[test]
    fn remove_last_character_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for n in 0..SUT_CAPACITY {
            let byte = ((n % 80) + 20) as u8;
            assert_that!(sut.push(byte), is_ok);
        }

        for n in (0..SUT_CAPACITY).rev() {
            let byte = ((n % 80) + 20) as u8;
            assert_that!(sut.remove(sut.len() - 1), eq Some(byte));
        }
    }

    #[test]
    fn remove_non_existing_entry_returns_none<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        assert_that!(sut.remove(7), is_none);
    }

    #[test]
    fn remove_non_existing_range_returns_false<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        assert_that!(sut.remove_range(2, 4), eq false);
    }

    #[test]
    fn remove_non_existing_range_from_non_empty_string_returns_false<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(sut.push_bytes(b"hui"), is_ok);

        assert_that!(sut.remove_range(1, 5), eq false);
    }

    #[test]
    fn remove_full_range_ends_up_in_empty_string<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for _ in 0..SUT_CAPACITY {
            assert_that!(sut.push(43), is_ok);
        }

        assert_that!(sut.remove_range(0, SUT_CAPACITY), eq true);
        assert_that!(sut.len(), eq 0);
        assert_that!(sut.is_empty(), eq true);
    }

    #[test]
    fn remove_range_from_start_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for n in 0..SUT_CAPACITY {
            let byte = ((n % 80) + 20) as u8;
            assert_that!(sut.push(byte), is_ok);
        }

        let number_of_removed_elements = SUT_CAPACITY / 2;
        assert_that!(sut.remove_range(0, number_of_removed_elements), eq true);
        assert_that!(sut.len(), eq SUT_CAPACITY - number_of_removed_elements);

        for n in 0..SUT_CAPACITY - number_of_removed_elements {
            let byte = (((n + SUT_CAPACITY - number_of_removed_elements - 1) % 80) + 20) as u8;
            assert_that!(sut[n], eq byte);
        }
    }

    #[test]
    fn remove_range_from_center_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for n in 0..SUT_CAPACITY {
            let byte = ((n % 80) + 20) as u8;
            assert_that!(sut.push(byte), is_ok);
        }

        let number_of_removed_elements = 20;
        assert_that!(sut.remove_range(10, number_of_removed_elements), eq true);
        assert_that!(sut.len(), eq SUT_CAPACITY - number_of_removed_elements);

        for n in 0..10 {
            let byte = ((n % 80) + 20) as u8;
            assert_that!(sut[n], eq byte);
        }

        for n in 10..SUT_CAPACITY - number_of_removed_elements {
            let byte = (((n + SUT_CAPACITY - number_of_removed_elements - 9) % 80) + 20) as u8;
            assert_that!(sut[n], eq byte);
        }
    }

    #[test]
    fn retain_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(sut.push_bytes(b"live long and nibble"), is_ok);

        sut.retain(|c| c == b' ');

        assert_that!(sut, len 17);
        assert_that!(sut.as_bytes_with_nul(), eq b"livelongandnibble\0");
    }

    #[test]
    fn rfind_of_character_in_empty_string_returns_none<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let sut = factory.create_sut();

        for n in 0u8..u8::MAX {
            assert_that!(sut.rfind(&[n]), is_none);
        }
    }

    #[test]
    fn rfind_of_range_in_empty_string_returns_none<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let sut = factory.create_sut();

        for n in 0u8..u8::MAX {
            assert_that!(sut.rfind(&[n, n, n]), is_none);
        }
    }

    #[test]
    fn rfind_of_char_located_at_the_beginning_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        const CHAR_TO_FIND: u8 = 37;

        assert_that!(sut.push(CHAR_TO_FIND), is_ok);
        for _ in 0..SUT_CAPACITY - 1 {
            assert_that!(sut.push(44), is_ok);
        }

        assert_that!(sut.rfind(&[CHAR_TO_FIND]), eq Some(0));
    }

    #[test]
    fn rfind_of_non_existing_char_returns_none<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        const CHAR_TO_FIND: u8 = 37;

        for _ in 0..SUT_CAPACITY - 1 {
            assert_that!(sut.push(44), is_ok);
        }

        assert_that!(sut.rfind(&[CHAR_TO_FIND]), is_none);
    }

    #[test]
    fn rfind_returns_first_char_match_from_end<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        const CHAR_TO_FIND: u8 = 37;

        assert_that!(sut.push(CHAR_TO_FIND), is_ok);
        for _ in 0..SUT_CAPACITY - 3 {
            assert_that!(sut.push(44), is_ok);
        }
        assert_that!(sut.push(CHAR_TO_FIND), is_ok);
        assert_that!(sut.push(44), is_ok);

        assert_that!(sut.rfind(&[CHAR_TO_FIND]), eq Some(SUT_CAPACITY - 2));
    }

    #[test]
    fn rfind_of_char_located_in_the_middle_works<Factory: StringTestFactory>() {
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

        assert_that!(sut.rfind(&[CHAR_TO_FIND]), eq Some((SUT_CAPACITY - 2)/2));
    }

    #[test]
    fn rfind_of_char_located_at_the_end_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        const CHAR_TO_FIND: u8 = 37;

        for _ in 0..(SUT_CAPACITY - 2) / 2 {
            assert_that!(sut.push(44), is_ok);
        }
        assert_that!(sut.push(CHAR_TO_FIND), is_ok);

        assert_that!(sut.rfind(&[CHAR_TO_FIND]), eq Some((SUT_CAPACITY - 2)/2));
    }

    #[test]
    fn rfind_of_range_located_at_the_beginning_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        const RANGE_TO_FIND: [u8; 4] = [37, 38, 49, 40];

        assert_that!(sut.push_bytes(&RANGE_TO_FIND), is_ok);
        for _ in 0..(SUT_CAPACITY - 1) / 2 {
            assert_that!(sut.push(44), is_ok);
        }

        assert_that!(sut.rfind(&RANGE_TO_FIND), eq Some(0));
    }

    #[test]
    fn rfind_of_range_located_in_the_middle_works<Factory: StringTestFactory>() {
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

        assert_that!(sut.rfind(&RANGE_TO_FIND), eq Some((SUT_CAPACITY - 4)/2));
    }

    #[test]
    fn rfind_of_range_located_at_the_end_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        const RANGE_TO_FIND: [u8; 4] = [37, 38, 49, 40];

        for _ in 0..(SUT_CAPACITY - 1) / 2 {
            assert_that!(sut.push(44), is_ok);
        }
        assert_that!(sut.push_bytes(&RANGE_TO_FIND), is_ok);

        assert_that!(sut.rfind(&RANGE_TO_FIND), eq Some((SUT_CAPACITY - 1)/2));
    }

    #[test]
    fn rfind_where_range_is_equal_to_sut_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        const RANGE_TO_FIND: [u8; 5] = [99, 55, 66, 40, 44];

        assert_that!(sut.push_bytes(&RANGE_TO_FIND), is_ok);

        assert_that!(sut.rfind(&RANGE_TO_FIND), eq Some(0));
    }

    #[test]
    fn truncate_to_larger_string_does_nothing<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(sut.push_bytes(b"blumbadix"), is_ok);
        sut.truncate(40);

        assert_that!(sut.len(), eq 9);
        assert_that!(sut.as_bytes_with_nul(), eq b"blumbadix\0");
    }

    #[test]
    fn truncate_to_smaller_string_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(sut.push_bytes(b"droubadix"), is_ok);
        sut.truncate(4);

        assert_that!(sut, len 4);
        assert_that!(sut.as_bytes_with_nul(), eq b"drou\0");
    }

    #[test]
    fn truncate_to_string_len_does_nothing<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(sut.push_bytes(b"oh my frog"), is_ok);

        for n in 10..SUT_CAPACITY {
            assert_that!(sut.len(), eq n);
            assert_that!(sut.push(21), is_ok);
            sut.truncate(sut.len());
        }
    }

    #[test]
    fn strip_prefix_from_empty_string_does_nothing<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(sut.strip_prefix(b"fubby"), eq false);

        assert_that!(sut.as_bytes(), eq b"");
    }

    #[test]
    fn strip_non_existing_prefix_does_nothing<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(sut.push_bytes(b"funny little moo"), is_ok);
        assert_that!(sut.strip_prefix(b"fubby"), eq false);

        assert_that!(sut.as_bytes(), eq b"funny little moo");
    }

    #[test]
    fn strip_existing_prefix_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(sut.push_bytes(b"its a meee mario"), is_ok);
        assert_that!(sut.strip_prefix(b"its a"), eq true);

        assert_that!(sut.as_bytes(), eq b" meee mario");
    }

    #[test]
    fn strip_existing_range_that_is_not_a_prefix_does_nothing<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(sut.push_bytes(b"what does a hypnotoad sound like?"), is_ok);
        assert_that!(sut.strip_prefix(b"hypnotoad"), eq false);

        assert_that!(sut.as_bytes(), eq b"what does a hypnotoad sound like?");
    }

    #[test]
    fn strip_non_existing_suffix_does_nothing<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(sut.push_bytes(b"all glory to the hypnotoad"), is_ok);
        assert_that!(sut.strip_suffix(b"frog"), eq false);

        assert_that!(sut.as_bytes(), eq b"all glory to the hypnotoad");
    }

    #[test]
    fn strip_existing_suffix_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(sut.push_bytes(b"all glory to the hasselhoff"), is_ok);
        assert_that!(sut.strip_suffix(b"hasselhoff"), eq true);

        assert_that!(sut.as_bytes(), eq b"all glory to the ");
    }

    #[test]
    fn strip_existing_range_that_is_not_a_suffix_does_nothing<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(sut.push_bytes(b"all glory to mario"), is_ok);
        assert_that!(sut.strip_suffix(b"glory"), eq false);

        assert_that!(sut.as_bytes(), eq b"all glory to mario");
    }

    #[test]
    fn strip_suffix_from_empty_string_does_nothing<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(sut.strip_suffix(b"fubby"), eq false);

        assert_that!(sut.as_bytes(), eq b"");
    }

    #[test]
    fn ordering_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut_small = factory.create_sut();
        let mut sut_greater = factory.create_sut();

        assert_that!(sut_small.push_bytes(b"bone_funny"), is_ok);
        assert_that!(sut_greater.push_bytes(b"fone_bunny"), is_ok);

        assert_that!(sut_small.cmp(&sut_small), eq Ordering::Equal);
        assert_that!(sut_small.cmp(&sut_greater), eq Ordering::Less);
        assert_that!(sut_greater.cmp(&sut_small), eq Ordering::Greater);
    }

    #[test]
    fn partial_ordering_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut_small = factory.create_sut();
        let mut sut_greater = factory.create_sut();

        assert_that!(sut_small.push_bytes(b"greater is smaller"), is_ok);
        assert_that!(sut_greater.push_bytes(b"small is greater"), is_ok);

        assert_that!(sut_small.partial_cmp(&sut_small), eq Some(Ordering::Equal));
        assert_that!(sut_small.partial_cmp(&sut_greater), eq Some(Ordering::Less));
        assert_that!(sut_greater.partial_cmp(&sut_small), eq Some(Ordering::Greater));
    }

    #[test]
    fn hash_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut_1 = factory.create_sut();
        let mut sut_1_1 = factory.create_sut();
        let mut sut_2 = factory.create_sut();

        assert_that!(sut_1.push_bytes(b"hypnotoad forever"), is_ok);
        assert_that!(sut_1_1.push_bytes(b"hypnotoad forever"), is_ok);
        assert_that!(sut_2.push_bytes(b"the hoff rocks"), is_ok);

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
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        assert_that!(sut.push_bytes(b"hello"), is_ok);

        sut.deref_mut()[0] = b'b';

        assert_that!(sut.as_bytes(), eq b"bello");
    }

    #[test]
    fn equality_works<Factory: StringTestFactory>() {
        let factory = Factory::new();
        let mut sut_1 = factory.create_sut();
        let mut sut_2 = factory.create_sut();

        assert_that!(sut_1.push_bytes(b"funzel"), is_ok);
        assert_that!(sut_2.push_bytes(b"rafunzel"), is_ok);

        assert_that!(sut_1 == sut_1, eq true);
        assert_that!(sut_1 == sut_2, eq false);

        assert_that!(*sut_1 == *sut_1, eq true);
        assert_that!(*sut_1 == *sut_2, eq false);
    }

    #[instantiate_tests(<PolymorphicStringFactory>)]
    mod polymorphic_string {}

    #[instantiate_tests(<RelocatableStringFactory>)]
    mod relocatable_string {}

    #[instantiate_tests(<StaticStringFactory>)]
    mod static_string {}
}

#[test]
fn error_display_works() {
    assert_that!(format!("{}", StringModificationError::InsertWouldExceedCapacity), eq "StringModificationError::InsertWouldExceedCapacity");
    assert_that!(format!("{}", StringModificationError::InvalidCharacter), eq "StringModificationError::InvalidCharacter");
}
