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

#[cfg(feature = "std")]
#[macro_export]
macro_rules! test_harness {
    () => {
        pub fn main() {
            fn expect_panic(
                test_fn: fn(),
                expected_message: Option<&'static str>,
            ) -> Result<(), $crate::libtest_mimic::Failed> {
                // Prevent stack trace from being output on expected panics
                let prev_hook = std::panic::take_hook();
                std::panic::set_hook(std::boxed::Box::new(|_| {}));
                let result = std::panic::catch_unwind(test_fn);
                std::panic::set_hook(prev_hook);

                match result {
                    Ok(_) => Err($crate::libtest_mimic::Failed::from(
                        "expected panic but test succeeded",
                    )),
                    Err(e) => {
                        if let Some(expected) = expected_message {
                            let matches = e
                                .downcast_ref::<&str>()
                                .map(|s| s.contains(expected))
                                .or_else(|| {
                                    e.downcast_ref::<String>().map(|s| s.contains(expected))
                                })
                                .unwrap_or(false);
                            if !matches {
                                return Err($crate::libtest_mimic::Failed::from(format!(
                                    "panic did not contain expected message \"{}\"",
                                    expected
                                )));
                            }
                        }
                        Ok(())
                    }
                }
            }

            let args = $crate::libtest_mimic::Arguments::from_args();
            let tests = $crate::inventory::iter::<$crate::TestCase>()
                .map(|test_case| {
                    let test_fn = test_case.test_fn;
                    let should_panic = test_case.should_panic;
                    let should_panic_message = test_case.should_panic_message;
                    $crate::libtest_mimic::Trial::test(test_case.name, move || {
                        if should_panic {
                            expect_panic(test_fn, should_panic_message)
                        } else {
                            test_fn();
                            Ok(())
                        }
                    })
                })
                .collect::<std::vec::Vec<_>>();
            $crate::libtest_mimic::run(&args, tests).exit();
        }
    };
}

#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! test_harness {
    () => {
        #[no_mangle]
        pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
            run_tests()
        }

        #[cfg(any(target_os = "linux", target_os = "nto"))]
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

        #[cfg(not(any(target_os = "linux", target_os = "nto")))]
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            loop {}
        }

        #[no_mangle]
        extern "C" fn rust_eh_personality() {}

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
