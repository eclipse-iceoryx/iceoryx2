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

//! Combines error handling with logging.

/// Macro to combine error handling with log messages. It automatically fails and converts the
/// error with [`From`].
///
/// ```
/// use iceoryx2_bb_log::fail;
///
/// #[derive(Debug)]
/// struct MyDataType {
///     value: u64
/// }
///
/// impl MyDataType {
///     fn doStuff(&self, value: u64) -> Result<(), ()> {
///         if value == 0 { Err(()) } else { Ok(()) }
///     }
///
///     fn doMoreStuff(&self) -> Result<(), u64> {
///         // fail when doStuff.is_err() and return the error 1234
///         fail!(from self, when self.doStuff(0),
///                 with 1234, "Failed while calling doStuff");
///         Ok(())
///     }
///
///     fn doMore(&self) -> Result<(), u64> {
///         if self.value == 0 {
///             // without condition, return error 4567
///             fail!(from self, with 4567, "Value is zero");
///         }
///
///         Ok(())
///     }
///
///     fn evenMore(&self) -> Result<(), u64> {
///         // forward error when it is compatible or convertable
///         fail!(from self, when self.doMore(), "doMore failed");
///         Ok(())
///     }
/// }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! fail {
    (with $error_value:expr, $($message:expr),*) => {
        debug!($($message),*);
        return Err($error_value);
    };
    (from $origin:expr, with $error_value:expr, $($message:expr),*) => {
        debug!(from $origin, $($message),*);
        return Err($error_value);
    };
    (from $origin:expr, when $call:expr, with $error_value:expr, $($message:expr),*) => {
        {
            let result = $call;
            match result.is_err() {
                true => {
                    debug!(from $origin, $($message),*);
                    return Err($error_value);
                }
                false => {
                    result.ok().unwrap()
                }
            }
        }
    };

    (from $origin:expr, when $call:expr, map $($error_origin:path => $error_value:expr);*,
            unmatched $error_unmatched:expr, $($message:expr),*) => {
        {
            match $call {
                Err(e) => {
                    debug!(from $origin, $($message),*);
                    match e {
                        $($error_origin => return Err($error_value)),*,
                        _ => return Err($error_unmatched),
                    }
                },
                Ok(v) => v,
            }
        }
    };




    (when $call:expr, $($message:expr),*) => {
        {
            let result = $call;
            match result.is_err() {
                true => {
                    debug!($($message),*);
                    result?
                }
                false => {
                    result.ok().unwrap()
                }
            }
        }
    };
    (from $origin:expr, when $call:expr, to $error:ty, $($message:expr),*) => {
        {
            let result = $call;
            match result.is_err() {
                true => {
                    debug!(from $origin, $($message),*);
                    let error = <$error>::from(result.err().unwrap());
                    Err(error)?
                }
                false => {
                    result.ok().unwrap()
                }
            }
        }
    };
    (from $origin:expr, when $call:expr, $($message:expr),*) => {
        {
            let result = $call;
            match result.is_err() {
                true => {
                    debug!(from $origin, $($message),*);
                    result?
                }
                false => {
                    result.ok().unwrap()
                }
            }
        }
    };
}
