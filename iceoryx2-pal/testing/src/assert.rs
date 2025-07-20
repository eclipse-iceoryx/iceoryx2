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

#[macro_export(local_inner_macros)]
macro_rules! assert_that {
    ($lhs:expr, eq $rhs:expr) => {
        {
            let lval = &$lhs;
            let rval = &$rhs;

            if !(lval == rval) {
                assert_that!(message $lhs, $rhs, lval, rval, "==");
            }
        }
   };
    ($lhs:expr, ne $rhs:expr) => {
        {
            let lval = &$lhs;
            let rval = &$rhs;

            if !(lval != rval) {
                assert_that!(message $lhs, $rhs, lval, rval, "!=");
            }
        }
    };
    ($lhs:expr, lt $rhs:expr) => {
        {
            let lval = &$lhs;
            let rval = &$rhs;

            if !(lval < rval) {
                assert_that!(message $lhs, $rhs, lval, rval, "<");
            }
        }
    };
    ($lhs:expr, le $rhs:expr) => {
        {
            let lval = &$lhs;
            let rval = &$rhs;

            if !(lval <= rval) {
                assert_that!(message $lhs, $rhs, lval, rval, "<=");
            }
        }
    };
    ($lhs:expr, gt $rhs:expr) => {
        {
            let lval = &$lhs;
            let rval = &$rhs;

            if !(lval > rval) {
                assert_that!(message $lhs, $rhs, lval, rval, ">");
            }
        }
    };
    ($lhs:expr, ge $rhs:expr) => {
        {
            let lval = &$lhs;
            let rval = &$rhs;

            if !(lval >= rval) {
                assert_that!(message $lhs, $rhs, lval, rval, ">=");
            }
        }
    };
    ($lhs:expr, aligned_to $rhs:expr) => {
        {
            let lval = $lhs as usize;
            let rval = $rhs as usize;
            let act_result = lval % rval;

            if !(act_result == 0) {
                assert_that!(message $lhs, $rhs, lval, rval, "aligned to");
            }
        }
    };
    ($lhs:expr, mod $rhs:expr, is $result:expr) => {
        {
            let lval = &$lhs;
            let rval = &$rhs;
            let act_result = lval % rval;

            if !(act_result == $result) {
                assert_that!(message $lhs, $rhs, lval, rval, "%", $result, act_result);
            }
        }
    };
    ($lhs:expr, is_ok) => {
        {
            let lval = $lhs.is_ok();

            if !lval {
                assert_that!(message_result $lhs, "is_ok()");
            }
        }
    };
    ($lhs:expr, is_err) => {
        {
            let lval = $lhs.is_err();

            if !lval {
                assert_that!(message_result $lhs, "is_err()");
            }
        }
    };
    ($lhs:expr, is_some) => {
        {
            let lval = $lhs.is_some();

            if !lval {
                assert_that!(message_result $lhs, "is_some()");
            }
        }
    };
    ($lhs:expr, is_none) => {
        {
            let lval = $lhs.is_none();

            if !lval {
                assert_that!(message_result $lhs, "is_none()");
            }
        }
    };
    ($lhs:expr, is_empty) => {
        {
            let lval = $lhs.is_empty();

            if !lval {
                assert_that!(message_result $lhs, "is_empty()");
            }
        }
    };
    ($lhs:expr, is_not_empty) => {
        {
            let lval = !$lhs.is_empty();

            if !lval {
                assert_that!(message_result $lhs, "is_empty() (not)");
            }
        }
    };
    ($lhs:expr, len $rhs:expr) => {
        {
            let lval = $lhs.len();
            if !(lval == $rhs) {
                assert_that!(message_property $lhs, lval, "len()", $rhs);
            }
        }
    };
    ($lhs:expr, any_of $rhs:expr) => {
        {
            let mut found = false;
            for value in &$rhs {
                if *value == $lhs {
                    found = true;
                    break;
                }
            }
            if !found {
                assert_that!(message_any_of $lhs, $rhs);
            }
        }
    };
    ($lhs:expr, contains $rhs:expr) => {
        {
            let mut does_contain = false;
            for value in &$lhs {
                if *value == $rhs {
                    does_contain = true;
                    break;
                }
            }
            if !does_contain {
                assert_that!(message_contains $lhs, $rhs);
            }
        }
    };
    ($lhs:expr, contains_match |$element:ident| $predicate:expr) => {
        {
            let mut does_contain = false;
            for $element in &$lhs {
                if $predicate {
                    does_contain = true;
                    break;
                }
            }
            if !does_contain {
                assert_that!(message_contains_match $lhs, core::stringify!($predicate));
            }
        }
    };
    ($lhs:expr, not_contains_match |$element:ident| $predicate:expr) => {
        {
            let mut does_contain = false;
            for $element in &$lhs {
                if $predicate {
                    does_contain = true;
                    break;
                }
            }
            if does_contain {
                assert_that!(message_not_contains_match $lhs, core::stringify!($predicate));
            }
        }
    };
    ($lhs:expr, time_at_least $rhs:expr) => {
        {
            let lval = $lhs.as_secs_f32();
            let rval = $rhs.as_secs_f32();
            let rval_adjusted = rval * (1.0 - iceoryx2_pal_testing::AT_LEAST_TIMING_VARIANCE).clamp(0.0, 1.0);

            if !(lval >= rval_adjusted) {
                assert_that!(message_time_at_least $lhs, $rhs, lval, rval, rval_adjusted);
            }
        }
    };
    ($call:expr, block_until $rhs:expr) => {
        {
            let watchdog = iceoryx2_pal_testing::watchdog::Watchdog::new();

            while $call() != $rhs {
                std::thread::yield_now();
                std::thread::sleep(core::time::Duration::from_millis(10));
                std::thread::yield_now();
            }
        }
    };
    [color_start] => {
        {
            use std::io::IsTerminal;
            if std::io::stderr().is_terminal() {
                "\x1b[1;4;33m"
            } else {
                ""
            }
        }
   };
    [color_end] => {
        {
            use std::io::IsTerminal;
            if std::io::stderr().is_terminal() {
                "\x1b[0m"
            } else {
                ""
            }
        }
    };
    [message_any_of $lhs:expr, $rhs:expr] => {
        core::panic!(
            "assertion failed: {}expr: {} any_of {} ({:?});  contents: {:?}{}",
                     assert_that![color_start],
                     core::stringify!($lhs),
                     core::stringify!($rhs),
                     $rhs,
                     $lhs,
                     assert_that![color_end]
        );
    };
    [message_contains $lhs:expr, $rhs:expr] => {
        core::panic!(
            "assertion failed: {}expr: {} contains {} ({:?});  contents: {:?}{}",
            assert_that![color_start],
            core::stringify!($lhs),
            core::stringify!($rhs),
            $rhs,
            $lhs,
            assert_that![color_end]
        );
    };
    [message_contains_match $lhs:expr, $predicate:expr] => {
        core::panic!(
            "assertion failed: {}expr: {} contains no element matching predicate: {}{}",
            assert_that![color_start],
            core::stringify!($lhs),
            $predicate,
            assert_that![color_end]
        );
    };
    [message_not_contains_match $lhs:expr, $predicate:expr] => {
        core::panic!(
            "assertion failed: {}expr: {} contains element matching predicate: {}{}",
            assert_that![color_start],
            core::stringify!($lhs),
            $predicate,
            assert_that![color_end]
        );
    };
    [message_property $lhs:expr, $lval:expr, $property:expr, $rhs:expr] => {
        core::panic!(
            "assertion failed: {}expr: {}.{} == {};  value: {} == {}{}",
            assert_that![color_start],
            core::stringify!($lhs),
            $property,
            $rhs,
            $lval,
            $rhs,
            assert_that![color_end]
        );
    };
    [message_result $lhs:expr, $state:expr] => {
        core::panic!(
            "assertion failed: {}{}.{}{}",
            assert_that![color_start],
            core::stringify!($lhs),
            $state,
            assert_that![color_end]
        );
    };
    [message_time_at_least $lhs:expr, $rhs:expr, $lval:expr, $rval:expr, $rval_adjusted:expr] => {
        core::panic!(
            "assertion failed: [ time test ] {}expr: {} at least {};  value: {:?} at least {:?} (jitter adjusted: {:?}){}",
            assert_that![color_start],
            core::stringify!($lhs),
            core::stringify!($rhs),
            $lval,
            $rval,
            $rval_adjusted,
            assert_that![color_end]
        );
    };
    [message $lhs:expr, $rhs:expr, $lval:expr, $rval:expr, $symbol:expr] => {
        core::panic!(
            "assertion failed: {}expr: {} {} {};  value: {:?} {} {:?}{}",
            assert_that![color_start],
            core::stringify!($lhs),
            $symbol,
            core::stringify!($rhs),
            $lval,
            $symbol,
            $rval,
            assert_that![color_end]
        );
    };
    [message $lhs:expr, $rhs:expr, $lval:expr, $rval:expr, $symbol:expr, $exp_result:expr, $act_result:expr] => {
        core::panic!(
            "assertion failed: {}expr: {} {} {} == {:?};  value: {:?} {} {:?} == {:?}{}",
            assert_that![color_start],
            core::stringify!($lhs),
            $symbol,
            core::stringify!($rhs),
            $exp_result,
            $lval,
            $symbol,
            $rval,
            $act_result,
            assert_that![color_end]
        );
    }
}
