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
type PanicHook = Box<dyn Fn(&std::panic::PanicHookInfo) + Send + Sync>;

#[cfg(feature = "std")]
struct PanicHookGuard(Option<PanicHook>);

#[cfg(feature = "std")]
impl PanicHookGuard {
    // Prevent stack trace from being output on expected panics
    fn suppress_stack_trace() -> Self {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        Self(Some(prev))
    }
}

#[cfg(feature = "std")]
impl Drop for PanicHookGuard {
    // Automatically restore panic hook on drop
    fn drop(&mut self) {
        if let Some(hook) = self.0.take() {
            std::panic::set_hook(hook);
        }
    }
}

#[cfg(feature = "std")]
fn panic_message(data: &(dyn core::any::Any + Send)) -> Option<&str> {
    data.downcast_ref::<&str>()
        .copied()
        .or_else(|| data.downcast_ref::<String>().map(String::as_str))
}

#[cfg(feature = "std")]
pub fn expect_panic(
    test_fn: fn(),
    expected_message: Option<&'static str>,
) -> Result<(), libtest_mimic::Failed> {
    let result = {
        let _guard = PanicHookGuard::suppress_stack_trace();
        std::panic::catch_unwind(test_fn)
    };

    match result {
        Ok(_) => Err(libtest_mimic::Failed::from(
            "expected panic but test succeeded",
        )),
        Err(e) => {
            if let Some(expected) = expected_message {
                let msg = panic_message(e.as_ref()).unwrap_or("");
                if !msg.contains(expected) {
                    return Err(libtest_mimic::Failed::from(format!(
                        "panic message did not contain expected string \"{expected}\""
                    )));
                }
            }

            Ok(())
        }
    }
}

#[cfg(feature = "std")]
#[macro_export]
macro_rules! test_harness {
    () => {
        extern crate alloc;
        use alloc::format;
        use alloc::string::String;
        use alloc::vec::Vec;

        pub fn main() {
            let mut args = $crate::libtest_mimic::Arguments::from_args();
            args.test_threads
                .get_or_insert($crate::DEFAULT_TEST_THREADS);

            let tests = $crate::inventory::iter::<$crate::TestCase>()
                .map(|test_case| {
                    let test_fn = test_case.test_fn;
                    let module = test_case
                        .module
                        .find("::")
                        .map(|i| &test_case.module[i + 2..])
                        .unwrap_or("");
                    let trial_name = if module.is_empty() {
                        alloc::string::String::from(test_case.name)
                    } else {
                        alloc::format!("{}::{}", module, test_case.name)
                    };
                    $crate::libtest_mimic::Trial::test(trial_name, move || {
                        if test_case.should_panic {
                            $crate::test_harness::expect_panic(
                                test_fn,
                                test_case.should_panic_message,
                            )
                        } else {
                            test_fn();
                            Ok(())
                        }
                    })
                    .with_ignored_flag(test_case.should_ignore)
                })
                .collect::<Vec<_>>();
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
                let module = test
                    .module
                    .find("::")
                    .map(|i| &test.module[i + 2..])
                    .unwrap_or("");
                if module.is_empty() {
                    $crate::internal::cout!("{}", test.name);
                } else {
                    $crate::internal::cout!("{}::{}", module, test.name);
                }
                $crate::internal::cout!(" ... ");

                if test.should_ignore {
                    $crate::internal::coutln!("ignored");
                    continue;
                }

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
