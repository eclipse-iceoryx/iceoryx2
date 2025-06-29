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

use core::sync::atomic::AtomicU32;

static COUNTER: AtomicU32 = AtomicU32::new(0);

#[generic_tests::define]
mod ice_atomic {
    use super::*;

    use iceoryx2_pal_concurrency_sync::iox_atomic::{internal::AtomicInteger, IoxAtomic};
    use iceoryx2_pal_testing::assert_that;

    use core::{
        fmt::Debug,
        ops::{AddAssign, BitAnd, BitOr},
        sync::atomic::Ordering,
    };

    trait Req: AtomicInteger + Debug + BitOr + BitAnd {
        fn generate_value() -> Self;
        fn generate_compatibility_value() -> Self;
        fn to_u32(&self) -> u32;
    }

    impl Req for u64 {
        fn to_u32(&self) -> u32 {
            *self as u32
        }

        fn generate_value() -> Self {
            0x0000f0f0f0f0 + COUNTER.fetch_add(1, Ordering::Relaxed) as u64
        }

        fn generate_compatibility_value() -> Self {
            0x00000000f0f0 + COUNTER.fetch_add(1, Ordering::Relaxed) as u64
        }
    }

    impl Req for u128 {
        fn to_u32(&self) -> u32 {
            *self as u32
        }

        fn generate_value() -> Self {
            0x00000f0f0f0f0f0f0f0f0f0f + COUNTER.fetch_add(1, Ordering::Relaxed) as u128
        }

        fn generate_compatibility_value() -> Self {
            0x000000000000000000000f0f + COUNTER.fetch_add(1, Ordering::Relaxed) as u128
        }
    }

    impl Req for i64 {
        fn to_u32(&self) -> u32 {
            *self as u32
        }

        fn generate_value() -> Self {
            (0x0000abcdabcdabcd + COUNTER.fetch_add(1, Ordering::Relaxed) as i64)
                * (-1_i64).pow(COUNTER.load(Ordering::Relaxed))
        }

        fn generate_compatibility_value() -> Self {
            0x000000000000abcd + COUNTER.fetch_add(1, Ordering::Relaxed) as i64
        }
    }

    impl Req for i128 {
        fn to_u32(&self) -> u32 {
            *self as u32
        }

        fn generate_value() -> Self {
            (0x0000abcdabcdabcddeadbeef + COUNTER.fetch_add(1, Ordering::Relaxed) as i128)
                * (-1_i128).pow(COUNTER.load(Ordering::Relaxed))
        }

        fn generate_compatibility_value() -> Self {
            0x00000000000000000000beef + COUNTER.fetch_add(1, Ordering::Relaxed) as i128
        }
    }

    #[test]
    fn new_works<T: Req>() {
        let n = T::generate_value();
        let sut = IoxAtomic::<T>::new(n);

        assert_that!(sut.load(Ordering::Relaxed), eq n);
    }

    #[test]
    fn as_ptr_works<T: Req>() {
        let n1 = T::generate_value();
        let n2 = T::generate_value();

        let sut = IoxAtomic::<T>::new(n1);
        let old_value = unsafe { *sut.as_ptr() };
        unsafe { *sut.as_ptr() = n2 };

        assert_that!(old_value, eq n1);
        assert_that!(unsafe{*sut.as_ptr()}, eq n2);
        assert_that!(sut.load(Ordering::Relaxed), eq n2);
    }

    #[test]
    fn compare_exchange_success_works<T: Req>() {
        let n_old = T::generate_value();
        let n_new = T::generate_value();
        let sut = IoxAtomic::<T>::new(n_old);

        let result = sut.compare_exchange(n_old, n_new, Ordering::Relaxed, Ordering::Relaxed);

        assert_that!(result, is_ok);
        assert_that!(result.unwrap(), eq n_old);
    }

    #[test]
    fn compare_exchange_weak_success_works<T: Req>() {
        let n_old = T::generate_value();
        let n_new = T::generate_value();
        let sut = IoxAtomic::<T>::new(n_old);

        let result = sut.compare_exchange_weak(n_old, n_new, Ordering::Relaxed, Ordering::Relaxed);

        assert_that!(result, is_ok);
        assert_that!(result.unwrap(), eq n_old);
    }

    #[test]
    fn compare_exchange_failure_works<T: Req>() {
        let n_outdated = T::generate_value();
        let n_old = T::generate_value();
        let n_new = T::generate_value();
        let sut = IoxAtomic::<T>::new(n_old);

        let result = sut.compare_exchange(n_outdated, n_new, Ordering::Relaxed, Ordering::Relaxed);

        assert_that!(result, is_err);
        assert_that!(result.err().unwrap(), eq n_old);
    }

    #[test]
    fn compare_exchange_weak_failure_works<T: Req>() {
        let n_outdated = T::generate_value();
        let n_old = T::generate_value();
        let n_new = T::generate_value();
        let sut = IoxAtomic::<T>::new(n_old);

        let result =
            sut.compare_exchange_weak(n_outdated, n_new, Ordering::Relaxed, Ordering::Relaxed);

        assert_that!(result, is_err);
        assert_that!(result.err().unwrap(), eq n_old);
    }

    #[test]
    fn fetch_add_works<T: Req>() {
        let n = T::generate_value();
        let sut = IoxAtomic::<T>::new(n);

        let result = sut.fetch_add(n, Ordering::Relaxed);

        assert_that!(result, eq n);
        assert_that!(sut.load(Ordering::Relaxed), eq n.overflowing_add(n).0);
    }

    #[test]
    fn fetch_and_works<T: Req>() {
        let n1 = T::generate_value();
        let n2 = T::generate_value();

        let sut = IoxAtomic::<T>::new(n1);

        let result = sut.fetch_and(n2, Ordering::Relaxed);

        assert_that!(result, eq n1);
        let mut bit_and = n1;
        bit_and &= n2;
        assert_that!(sut.load(Ordering::Relaxed), eq bit_and);
    }

    #[test]
    fn fetch_max_works<T: Req>() {
        let n1 = T::generate_value();
        let n2 = T::generate_value();

        let sut_1 = IoxAtomic::<T>::new(n1);
        let sut_2 = IoxAtomic::<T>::new(n2);

        let result_1 = sut_1.fetch_max(n2, Ordering::Relaxed);
        let result_2 = sut_2.fetch_max(n1, Ordering::Relaxed);

        assert_that!(result_1, eq n1);
        assert_that!(result_2, eq n2);
        assert_that!(sut_1.load(Ordering::Relaxed), eq n1.max(n2));
        assert_that!(sut_2.load(Ordering::Relaxed), eq n1.max(n2));
    }

    #[test]
    fn fetch_min_works<T: Req>() {
        let n1 = T::generate_value();
        let n2 = T::generate_value();

        let sut_1 = IoxAtomic::<T>::new(n1);
        let sut_2 = IoxAtomic::<T>::new(n2);

        let result_1 = sut_1.fetch_min(n2, Ordering::Relaxed);
        let result_2 = sut_2.fetch_min(n1, Ordering::Relaxed);

        assert_that!(result_1, eq n1);
        assert_that!(result_2, eq n2);
        assert_that!(sut_1.load(Ordering::Relaxed), eq n1.min(n2));
        assert_that!(sut_2.load(Ordering::Relaxed), eq n1.min(n2));
    }

    #[test]
    fn fetch_nand_works<T: Req>() {
        let n1 = T::generate_value();
        let n2 = T::generate_value();

        let sut = IoxAtomic::<T>::new(n1);

        let result = sut.fetch_nand(n2, Ordering::Relaxed);

        assert_that!(result, eq n1);
        let bit_nand = !(n1 & n2);
        assert_that!(sut.load(Ordering::Relaxed), eq bit_nand);
    }

    #[test]
    fn fetch_or_works<T: Req>() {
        let n1 = T::generate_value();
        let n2 = T::generate_value();

        let sut = IoxAtomic::<T>::new(n1);

        let result = sut.fetch_or(n2, Ordering::Relaxed);

        assert_that!(result, eq n1);
        let mut bit_or = n1;
        bit_or |= n2;
        assert_that!(sut.load(Ordering::Relaxed), eq bit_or);
    }

    #[test]
    fn fetch_sub_works<T: Req>() {
        let n1 = T::generate_value();
        let n2 = T::generate_value();

        let sut = IoxAtomic::<T>::new(n1);

        let result = sut.fetch_sub(n2, Ordering::Relaxed);

        assert_that!(result, eq n1);
        assert_that!(sut.load(Ordering::Relaxed), eq n1.overflowing_sub(n2).0);
    }

    fn ok_fetch_update<T: AddAssign + Copy>(value: T) -> Option<T> {
        let mut temp = value;
        temp += value;
        Some(temp)
    }

    fn err_fetch_update<T: AddAssign + Copy>(_value: T) -> Option<T> {
        None
    }

    #[test]
    fn fetch_update_success_works<T: Req>() {
        let n1 = T::generate_value();

        let sut = IoxAtomic::<T>::new(n1);

        let result = sut.fetch_update(Ordering::Relaxed, Ordering::Relaxed, ok_fetch_update::<T>);

        assert_that!(result, is_ok);
        assert_that!(result.unwrap(), eq n1);
        let mut n = n1;
        n += n1;
        assert_that!(sut.load(Ordering::Relaxed), eq n);
    }

    #[test]
    fn fetch_update_failure_works<T: Req>() {
        let n1 = T::generate_value();

        let sut = IoxAtomic::<T>::new(n1);

        let result = sut.fetch_update(Ordering::Relaxed, Ordering::Relaxed, err_fetch_update::<T>);

        assert_that!(result, is_err);
        assert_that!(result.err().unwrap(), eq n1);
        assert_that!(sut.load(Ordering::Relaxed), eq n1);
    }

    #[test]
    fn fetch_xor_works<T: Req>() {
        let n1 = T::generate_value();
        let n2 = T::generate_value();

        let sut = IoxAtomic::<T>::new(n1);

        let result = sut.fetch_xor(n2, Ordering::Relaxed);

        assert_that!(result, eq n1);
        let mut bit_xor = n1;
        bit_xor ^= n2;
        assert_that!(sut.load(Ordering::Relaxed), eq bit_xor);
    }

    #[test]
    fn into_inner_works<T: Req>() {
        let n = T::generate_value();
        let sut = IoxAtomic::<T>::new(n);

        assert_that!(IoxAtomic::<T>::into_inner(sut), eq n);
    }

    #[test]
    fn load_store_works<T: Req>() {
        let n1 = T::generate_value();
        let n2 = T::generate_value();
        let sut = IoxAtomic::<T>::new(n1);

        sut.store(n2, Ordering::Relaxed);

        assert_that!(sut.load(Ordering::Relaxed), eq n2);
    }

    #[test]
    fn swap_works<T: Req>() {
        let n1 = T::generate_value();
        let n2 = T::generate_value();
        let sut = IoxAtomic::<T>::new(n1);

        let result = sut.swap(n2, Ordering::Relaxed);

        assert_that!(result, eq n1);
        assert_that!(sut.load(Ordering::Relaxed), eq n2);
    }

    #[test]
    fn compatibility_new_works<T: Req>() {
        let n = T::generate_compatibility_value();
        let sut = IoxAtomic::<T>::new(n);
        let compat = AtomicU32::new(n.to_u32());

        assert_that!(compat.load(Ordering::Relaxed), eq sut.load(Ordering::Relaxed).to_u32());
    }

    #[test]
    fn compatibility_as_ptr_works<T: Req>() {
        let n1 = T::generate_compatibility_value();
        let n2 = T::generate_compatibility_value();

        let sut = IoxAtomic::<T>::new(n1);
        let compat = AtomicU32::new(n1.to_u32());

        assert_that!(unsafe {*compat.as_ptr()}, eq unsafe{*sut.as_ptr()}.to_u32() );

        unsafe { *sut.as_ptr() = n2 };
        unsafe { *compat.as_ptr() = n2.to_u32() };

        assert_that!(unsafe {*compat.as_ptr()}, eq unsafe{*sut.as_ptr()}.to_u32() );
        assert_that!(unsafe {*compat.as_ptr()}, eq n2.to_u32() );
    }

    #[test]
    fn compatibility_compare_exchange_success_works<T: Req>() {
        let n1 = T::generate_compatibility_value();
        let n2 = T::generate_compatibility_value();

        let sut = IoxAtomic::<T>::new(n1);
        let compat = AtomicU32::new(n1.to_u32());

        let result_sut = sut.compare_exchange(n1, n2, Ordering::Relaxed, Ordering::Relaxed);
        let result_compat = compat.compare_exchange(
            n1.to_u32(),
            n2.to_u32(),
            Ordering::Relaxed,
            Ordering::Relaxed,
        );

        assert_that!(result_sut, is_ok);
        assert_that!(result_compat, is_ok);

        assert_that!(result_sut.unwrap(), eq n1);
        assert_that!(result_compat.unwrap(), eq n1.to_u32());
    }

    #[test]
    fn compatibility_compare_exchange_weak_success_works<T: Req>() {
        let n1 = T::generate_compatibility_value();
        let n2 = T::generate_compatibility_value();

        let sut = IoxAtomic::<T>::new(n1);
        let compat = AtomicU32::new(n1.to_u32());

        let result_sut = sut.compare_exchange_weak(n1, n2, Ordering::Relaxed, Ordering::Relaxed);
        let result_compat = compat.compare_exchange_weak(
            n1.to_u32(),
            n2.to_u32(),
            Ordering::Relaxed,
            Ordering::Relaxed,
        );

        assert_that!(result_sut, is_ok);
        assert_that!(result_compat, is_ok);

        assert_that!(result_sut.unwrap(), eq n1);
        assert_that!(result_compat.unwrap(), eq n1.to_u32());
    }

    #[test]
    fn compatibility_compare_exchange_failure_works<T: Req>() {
        let n1 = T::generate_compatibility_value();
        let n2 = T::generate_compatibility_value();

        let sut = IoxAtomic::<T>::new(n1);
        let compat = AtomicU32::new(n1.to_u32());

        let result_sut = sut.compare_exchange(n2, n1, Ordering::Relaxed, Ordering::Relaxed);
        let result_compat = compat.compare_exchange(
            n2.to_u32(),
            n1.to_u32(),
            Ordering::Relaxed,
            Ordering::Relaxed,
        );

        assert_that!(result_sut, is_err);
        assert_that!(result_compat, is_err);

        assert_that!(result_sut.err().unwrap(), eq n1);
        assert_that!(result_compat.err().unwrap(), eq n1.to_u32());
    }

    #[test]
    fn compatibility_compare_exchange_weak_failure_works<T: Req>() {
        let n1 = T::generate_compatibility_value();
        let n2 = T::generate_compatibility_value();

        let sut = IoxAtomic::<T>::new(n1);
        let compat = AtomicU32::new(n1.to_u32());

        let result_sut = sut.compare_exchange_weak(n2, n1, Ordering::Relaxed, Ordering::Relaxed);
        let result_compat = compat.compare_exchange_weak(
            n2.to_u32(),
            n1.to_u32(),
            Ordering::Relaxed,
            Ordering::Relaxed,
        );

        assert_that!(result_sut, is_err);
        assert_that!(result_compat, is_err);

        assert_that!(result_sut.err().unwrap(), eq n1);
        assert_that!(result_compat.err().unwrap(), eq n1.to_u32());
    }

    #[test]
    fn compatibility_fetch_add_works<T: Req>() {
        let n1 = T::generate_compatibility_value();
        let n2 = T::generate_compatibility_value();

        let sut = IoxAtomic::<T>::new(n1);
        let compat = AtomicU32::new(n1.to_u32());

        assert_that!(sut.fetch_add(n2, Ordering::Relaxed).to_u32(), eq compat.fetch_add(n2.to_u32(), Ordering::Relaxed));
        assert_that!(sut.load(Ordering::Relaxed).to_u32(), eq compat.load(Ordering::Relaxed));
    }

    #[test]
    fn compatibility_fetch_and_works<T: Req>() {
        let n1 = T::generate_compatibility_value();
        let n2 = T::generate_compatibility_value();

        let sut = IoxAtomic::<T>::new(n1);
        let compat = AtomicU32::new(n1.to_u32());

        assert_that!(sut.fetch_and(n2, Ordering::Relaxed).to_u32(), eq compat.fetch_and(n2.to_u32(), Ordering::Relaxed));
        assert_that!(sut.load(Ordering::Relaxed).to_u32(), eq compat.load(Ordering::Relaxed));
    }

    #[test]
    fn compatibility_fetch_max_works<T: Req>() {
        let n1 = T::generate_compatibility_value();
        let n2 = T::generate_compatibility_value();

        let sut = IoxAtomic::<T>::new(n1);
        let compat = AtomicU32::new(n1.to_u32());

        assert_that!(sut.fetch_max(n2, Ordering::Relaxed).to_u32(), eq compat.fetch_max(n2.to_u32(), Ordering::Relaxed));
        assert_that!(sut.load(Ordering::Relaxed).to_u32(), eq compat.load(Ordering::Relaxed));
    }

    #[test]
    fn compatibility_fetch_min_works<T: Req>() {
        let n1 = T::generate_compatibility_value();
        let n2 = T::generate_compatibility_value();

        let sut = IoxAtomic::<T>::new(n1);
        let compat = AtomicU32::new(n1.to_u32());

        assert_that!(sut.fetch_min(n2, Ordering::Relaxed).to_u32(), eq compat.fetch_min(n2.to_u32(), Ordering::Relaxed));
        assert_that!(sut.load(Ordering::Relaxed).to_u32(), eq compat.load(Ordering::Relaxed));
    }

    #[test]
    fn compatibility_fetch_nand_works<T: Req>() {
        let n1 = T::generate_compatibility_value();
        let n2 = T::generate_compatibility_value();

        let sut = IoxAtomic::<T>::new(n1);
        let compat = AtomicU32::new(n1.to_u32());

        assert_that!(sut.fetch_nand(n2, Ordering::Relaxed).to_u32(), eq compat.fetch_nand(n2.to_u32(), Ordering::Relaxed));
        assert_that!(sut.load(Ordering::Relaxed).to_u32(), eq compat.load(Ordering::Relaxed));
    }

    #[test]
    fn compatibility_fetch_or_works<T: Req>() {
        let n1 = T::generate_compatibility_value();
        let n2 = T::generate_compatibility_value();

        let sut = IoxAtomic::<T>::new(n1);
        let compat = AtomicU32::new(n1.to_u32());

        assert_that!(sut.fetch_or(n2, Ordering::Relaxed).to_u32(), eq compat.fetch_or(n2.to_u32(), Ordering::Relaxed));
        assert_that!(sut.load(Ordering::Relaxed).to_u32(), eq compat.load(Ordering::Relaxed));
    }

    #[test]
    fn compatibility_fetch_sub_works<T: Req>() {
        let n1 = T::generate_compatibility_value();
        let n2 = T::generate_compatibility_value();

        let sut = IoxAtomic::<T>::new(n1);
        let compat = AtomicU32::new(n1.to_u32());

        assert_that!(sut.fetch_sub(n2, Ordering::Relaxed).to_u32(), eq compat.fetch_sub(n2.to_u32(), Ordering::Relaxed));
        assert_that!(sut.load(Ordering::Relaxed).to_u32(), eq compat.load(Ordering::Relaxed));
    }

    #[test]
    fn compatibility_fetch_update_success_works<T: Req>() {
        let n1 = T::generate_compatibility_value();

        let sut = IoxAtomic::<T>::new(n1);
        let compat = AtomicU32::new(n1.to_u32());

        let result_sut =
            sut.fetch_update(Ordering::Relaxed, Ordering::Relaxed, ok_fetch_update::<T>);
        let result_compat =
            compat.fetch_update(Ordering::Relaxed, Ordering::Relaxed, ok_fetch_update::<u32>);

        assert_that!(result_sut, is_ok);
        assert_that!(result_compat, is_ok);

        assert_that!(result_sut.unwrap().to_u32(), eq result_compat.unwrap());
        assert_that!(sut.load(Ordering::Relaxed).to_u32(), eq compat.load(Ordering::Relaxed));
    }

    #[test]
    fn compatibility_fetch_update_failure_works<T: Req>() {
        let n1 = T::generate_compatibility_value();

        let sut = IoxAtomic::<T>::new(n1);
        let compat = AtomicU32::new(n1.to_u32());

        let result_sut =
            sut.fetch_update(Ordering::Relaxed, Ordering::Relaxed, err_fetch_update::<T>);
        let result_compat = compat.fetch_update(
            Ordering::Relaxed,
            Ordering::Relaxed,
            err_fetch_update::<u32>,
        );

        assert_that!(result_sut, is_err);
        assert_that!(result_compat, is_err);

        assert_that!(result_sut.err().unwrap().to_u32(), eq result_compat.err().unwrap());
        assert_that!(sut.load(Ordering::Relaxed).to_u32(), eq compat.load(Ordering::Relaxed));
    }

    #[test]
    fn compatibility_fetch_xor_works<T: Req>() {
        let n1 = T::generate_compatibility_value();
        let n2 = T::generate_compatibility_value();

        let sut = IoxAtomic::<T>::new(n1);
        let compat = AtomicU32::new(n1.to_u32());

        assert_that!(sut.fetch_xor(n2, Ordering::Relaxed).to_u32(), eq compat.fetch_xor(n2.to_u32(), Ordering::Relaxed));
        assert_that!(sut.load(Ordering::Relaxed).to_u32(), eq compat.load(Ordering::Relaxed));
    }

    #[test]
    fn compatibility_swap_works<T: Req>() {
        let n1 = T::generate_compatibility_value();
        let n2 = T::generate_compatibility_value();

        let sut = IoxAtomic::<T>::new(n1);
        let compat = AtomicU32::new(n1.to_u32());

        assert_that!(sut.swap(n2, Ordering::Relaxed).to_u32(), eq compat.swap(n2.to_u32(), Ordering::Relaxed));
        assert_that!(sut.load(Ordering::Relaxed).to_u32(), eq compat.load(Ordering::Relaxed));
    }

    #[instantiate_tests(<u64>)]
    mod u64 {}

    #[instantiate_tests(<u128>)]
    mod u128 {}

    #[instantiate_tests(<i64>)]
    mod i64 {}

    #[instantiate_tests(<i128>)]
    mod i128 {}
}
