// Copyright (c) 2024 Contributors to the Eclipse Foundation
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
mod ipc_capable {
    use iceoryx2_bb_posix::barrier::*;
    use iceoryx2_bb_posix::ipc_capable::{Handle, IpcCapable};
    use iceoryx2_bb_posix::mutex::{Mutex, MutexBuilder, MutexHandle};
    use iceoryx2_bb_posix::read_write_mutex::{
        ReadWriteMutex, ReadWriteMutexBuilder, ReadWriteMutexHandle,
    };
    use iceoryx2_bb_posix::semaphore::{
        UnnamedSemaphore, UnnamedSemaphoreBuilder, UnnamedSemaphoreHandle,
    };
    use iceoryx2_bb_testing::assert_that;

    trait TestSut {
        type Handle: Handle;
        type Sut<'a>: IpcCapable<'a, Self::Handle>;

        fn init_process_local_handle(handle: &Self::Handle);
        fn init_inter_process_handle(handle: &Self::Handle);
    }

    #[test]
    fn new_handle_is_not_initialized<Sut: TestSut>() {
        let sut_handle = Sut::Handle::new();
        assert_that!(sut_handle.is_initialized(), eq false);
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn creating_ipc_construct_from_uninitialized_handle_panics<Sut: TestSut>() {
        let sut_handle = Sut::Handle::new();

        unsafe { Sut::Sut::from_ipc_handle(&sut_handle) };
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn creating_ipc_construct_from_process_local_handle_panics<Sut: TestSut>() {
        let sut_handle = Sut::Handle::new();
        Sut::init_process_local_handle(&sut_handle);

        unsafe { Sut::Sut::from_ipc_handle(&sut_handle) };
    }

    #[test]
    fn creating_ipc_construct_from_ipc_handle_works<Sut: TestSut>() {
        let sut_handle = Sut::Handle::new();
        Sut::init_inter_process_handle(&sut_handle);

        // no panic here
        unsafe { Sut::Sut::from_ipc_handle(&sut_handle) };
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn init_handle_twice_panics<Sut: TestSut>() {
        let sut_handle = Sut::Handle::new();
        Sut::init_process_local_handle(&sut_handle);

        Sut::init_inter_process_handle(&sut_handle);
    }

    #[test]
    fn initialized_handle_is_initialized<Sut: TestSut>() {
        let sut_handle_1 = Sut::Handle::new();
        let sut_handle_2 = Sut::Handle::new();

        assert_that!(sut_handle_1.is_initialized(), eq false);
        assert_that!(sut_handle_2.is_initialized(), eq false);

        Sut::init_process_local_handle(&sut_handle_1);
        Sut::init_inter_process_handle(&sut_handle_2);

        assert_that!(sut_handle_1.is_initialized(), eq true);
        assert_that!(sut_handle_2.is_initialized(), eq true);
    }

    #[test]
    fn inter_process_capability_is_set_correctly<Sut: TestSut>() {
        let sut_handle_1 = Sut::Handle::new();
        let sut_handle_2 = Sut::Handle::new();

        Sut::init_process_local_handle(&sut_handle_1);
        Sut::init_inter_process_handle(&sut_handle_2);

        assert_that!(sut_handle_1.is_inter_process_capable(), eq false);
        assert_that!(sut_handle_2.is_inter_process_capable(), eq true);
    }

    struct BarrierTest {}

    impl TestSut for BarrierTest {
        type Handle = BarrierHandle;
        type Sut<'a> = Barrier<'a>;

        fn init_process_local_handle(handle: &Self::Handle) {
            BarrierBuilder::new(1)
                .is_interprocess_capable(false)
                .create(handle)
                .unwrap();
        }

        fn init_inter_process_handle(handle: &Self::Handle) {
            BarrierBuilder::new(1)
                .is_interprocess_capable(true)
                .create(handle)
                .unwrap();
        }
    }

    #[instantiate_tests(<BarrierTest>)]
    mod barrier {}

    struct UnnamedSemaphoreTest {}

    impl TestSut for UnnamedSemaphoreTest {
        type Handle = UnnamedSemaphoreHandle;
        type Sut<'a> = UnnamedSemaphore<'a>;

        fn init_process_local_handle(handle: &Self::Handle) {
            UnnamedSemaphoreBuilder::new()
                .is_interprocess_capable(false)
                .create(handle)
                .unwrap();
        }

        fn init_inter_process_handle(handle: &Self::Handle) {
            UnnamedSemaphoreBuilder::new()
                .is_interprocess_capable(true)
                .create(handle)
                .unwrap();
        }
    }

    #[instantiate_tests(<UnnamedSemaphoreTest>)]
    mod unnamed_semaphore {}

    struct MutexTest {}

    impl TestSut for MutexTest {
        type Handle = MutexHandle<u64>;
        type Sut<'a> = Mutex<'a, 'a, u64>;

        fn init_process_local_handle(handle: &Self::Handle) {
            MutexBuilder::new()
                .is_interprocess_capable(false)
                .create(0, handle)
                .unwrap();
        }

        fn init_inter_process_handle(handle: &Self::Handle) {
            MutexBuilder::new()
                .is_interprocess_capable(true)
                .create(0, handle)
                .unwrap();
        }
    }

    #[instantiate_tests(<MutexTest>)]
    mod mutex {}

    struct ReadWriteMutexTest {}

    impl TestSut for ReadWriteMutexTest {
        type Handle = ReadWriteMutexHandle<u64>;
        type Sut<'a> = ReadWriteMutex<'a, 'a, u64>;

        fn init_process_local_handle(handle: &Self::Handle) {
            ReadWriteMutexBuilder::new()
                .is_interprocess_capable(false)
                .create(0, handle)
                .unwrap();
        }

        fn init_inter_process_handle(handle: &Self::Handle) {
            ReadWriteMutexBuilder::new()
                .is_interprocess_capable(true)
                .create(0, handle)
                .unwrap();
        }
    }

    #[instantiate_tests(<ReadWriteMutexTest>)]
    mod read_write_mutex {}
}
