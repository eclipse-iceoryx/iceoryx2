// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

#![no_std]

pub use inventory;

pub struct TestCase {
    pub name: &'static str,
    pub test_fn: fn(),
}

inventory::collect!(TestCase);

pub mod internal {
    #[cfg(any(target_os = "linux", target_os = "nto"))]
    pub use iceoryx2_pal_posix::posix::abort;
    pub use iceoryx2_pal_print::cout;
    pub use iceoryx2_pal_print::coutln;
}

#[macro_export]
macro_rules! bootstrap {
    () => {
        #[cfg(feature = "std")]
        #[no_mangle]
        pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
            0
        }

        #[cfg(not(feature = "std"))]
        #[no_mangle]
        pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
            run_tests()
        }

        #[cfg(all(any(target_os = "linux", target_os = "nto"), not(feature = "std")))]
        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo) -> ! {
            $crate::internal::coutln!("");
            $crate::internal::coutln!("");
            $crate::internal::cout!("Failed: {}", info);
            $crate::internal::coutln!("");
            $crate::internal::coutln!("");

            unsafe {
                $crate::internal::abort();
                core::hint::unreachable_unchecked()
            }
        }

        #[cfg(all(not(any(target_os = "linux", target_os = "nto")), not(feature = "std")))]
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            loop {}
        }

        #[cfg(not(feature = "std"))]
        #[no_mangle]
        extern "C" fn rust_eh_personality() {}

        #[cfg(not(feature = "std"))]
        fn run_tests() -> isize {
            let tests = $crate::inventory::iter::<$crate::TestCase>();

            let mut passed = 0;
            let mut failed = 0;

            $crate::internal::coutln!("");
            $crate::internal::cout!("running tests:");
            $crate::internal::coutln!("");
            $crate::internal::coutln!("");

            for test in tests {
                $crate::internal::cout!("test ");
                $crate::internal::cout!("{}", test.name);
                $crate::internal::cout!(" ... ");

                (test.test_fn)();

                $crate::internal::coutln!("ok");
                passed += 1;
            }

            $crate::internal::coutln!("");
            $crate::internal::cout!("test result: ");
            if failed == 0 {
                $crate::internal::cout!("ok. ");
            } else {
                $crate::internal::cout!("FAILED. ");
            }

            $crate::internal::cout!("{}", passed);
            $crate::internal::cout!(" passed; ");
            $crate::internal::cout!("{}", failed);
            $crate::internal::cout!(" failed");
            $crate::internal::coutln!("");

            if failed > 0 {
                1
            } else {
                0
            }
        }
    };
}
