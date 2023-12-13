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

//! Used internally to perform error handling based on errnos

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! handle_errno {
    ($error_type:ident, from $origin:expr,
     $( $errno:ident$(::$errno_suf:ident)? => ($error_value:ident$(($inner_value:expr))?, $($message:expr),*)),*) => {
        match posix::Errno::get() {
            $($errno$(::$errno_suf)? => { iceoryx2_bb_log::fail!(from $origin, with $error_type::$error_value$(($inner_value))?, $($message),*); } ),*
        }
    };
    ($error_type:ident, from $origin:expr,
     $( fatal $fat_errno:ident$(::$fat_errno_suf:ident)? => ($($fat_message:expr),*));*,
     $( $errno:ident$(::$errno_suf:ident)? => ($error_value:ident$(($inner_value:expr))?, $($message:expr),*)),*) => {
        match posix::Errno::get() {
            $($fat_errno$(::$fat_errno_suf)? => { iceoryx2_bb_log::fatal_panic!(from $origin, $($fat_message),*); } ),*
            $($errno$(::$errno_suf)? => { iceoryx2_bb_log::fail!(from $origin, with $error_type::$error_value$(($inner_value))?, $($message),*); } ),*
        }
    };
    ($error_type:ident, from $origin:expr,
     $( success_when $condition:expr, $suc_errno:ident$(::$suc_errno_suf:ident)? => ($success_value:expr, $suc_err_value:ident, $($suc_err_message:expr),*));*,
     $( $errno:ident$(::$errno_suf:ident)? => ($error_value:ident$(($inner_value:expr))?, $($message:expr),*)),*) => {
        match posix::Errno::get() {
            $($suc_errno$(::$suc_errno_suf)? => {
                if $condition {
                    return Ok($success_value);
                } else {
                    iceoryx2_bb_log::fail!(from $origin, with $error_type::$suc_err_value, $($suc_err_message),*);
                }
            } ),*
            $($errno$(::$errno_suf)? => { iceoryx2_bb_log::fail!(from $origin, with $error_type::$error_value$(($inner_value))?, $($message),*); } ),*
        }
    };
    ($error_type:ident, from $origin:expr,
     $( success $suc_errno:ident$(::$suc_errno_suf:ident)? => $success_value:expr);*,
     $( $errno:ident$(::$errno_suf:ident)? => ($error_value:ident$(($inner_value:expr))?, $($message:expr),*)),*) => {
        match posix::Errno::get() {
            $($suc_errno$(::$suc_errno_suf)? => { return Ok($success_value); } ),*
            $($errno$(::$errno_suf)? => { iceoryx2_bb_log::fail!(from $origin, with $error_type::$error_value$(($inner_value))?, $($message),*); } ),*
        }
    };
    ($error_type:ident, from $origin:expr,
     $( success $suc_errno:ident$(::$suc_errno_suf:ident)? => $success_value:expr);*,
     $( fatal $fat_errno:ident$(::$fat_errno_suf:ident)? => ($($fat_message:expr),*));*,
     $( $errno:ident$(::$errno_suf:ident)? => ($error_value:ident$(($inner_value:expr))?, $($message:expr),*)),*) => {
        match posix::Errno::get() {
            $($suc_errno$(::$suc_errno_suf)? => { return Ok($success_value); } ),*
            $($fat_errno$(::$fat_errno_suf)? => { iceoryx2_bb_log::fatal_panic!(from $origin, $($fat_message),*); } ),*
            $($errno$(::$errno_suf)? => { iceoryx2_bb_log::fail!(from $origin, with $error_type::$error_value$(($inner_value))?, $($message),*); } ),*
        }
    };
    ($error_type:ident, from $origin:expr, errno_source $errno_source:expr,
     $( success $suc_errno:ident$(::$suc_errno_suf:ident)? => $success_value:expr);*,
     $( $errno:ident$(::$errno_suf:ident)? => ($error_value:ident$(($inner_value:expr))?, $($message:expr),*)),*) => {
        match $errno_source {
            $($suc_errno$(::$suc_errno_suf)? => { return Ok($success_value); } ),*
            $($errno$(::$errno_suf)? => { iceoryx2_bb_log::fail!(from $origin, with $error_type::$error_value$(($inner_value))?, $($message),*); } ),*
        }
    };
    ($error_type:ident, from $origin:expr, errno_source $errno_source:expr,
     $( success $suc_errno:ident$(::$suc_errno_suf:ident)? => $success_value:expr);*,
     $( fatal $fat_errno:ident$(::$fat_errno_suf:ident)? => ($($fat_message:expr),*));*,
     $( $errno:ident$(::$errno_suf:ident)? => ($error_value:ident$(($inner_value:expr))?, $($message:expr),*)),*) => {
        match $errno_source {
            $($suc_errno$(::$suc_errno_suf)? => { return Ok($success_value); } ),*
            $($fat_errno$(::$fat_errno_suf)? => { iceoryx2_bb_log::fatal_panic!(from $origin, $($fat_message),*); } ),*
            $($errno$(::$errno_suf)? => { iceoryx2_bb_log::fail!(from $origin, with $error_type::$error_value$(($inner_value))?, $($message),*); } ),*
        }
    };
    ($error_type:ident, from $origin:expr, errno_source $errno_source:expr,
     continue_on_success,
     $( success $suc_errno:ident$(::$suc_errno_suf:ident)? => $success_value:expr);*,
     $( $errno:ident$(::$errno_suf:ident)? => ($error_value:ident$(($inner_value:expr))?, $($message:expr),*)),*) => {
        match $errno_source {
            $($suc_errno$(::$suc_errno_suf)? => { $success_value } ),*
            $($errno$(::$errno_suf)? => { iceoryx2_bb_log::fail!(from $origin, with $error_type::$error_value$(($inner_value))?, $($message),*); } ),*
        }
    };
    ($error_type:ident, from $origin:expr, errno_source $errno_source:expr,
     continue_on_success,
     $( success $suc_errno:ident$(::$suc_errno_suf:ident)? => $success_value:expr);*,
     $( fatal $fat_errno:ident$(::$fat_errno_suf:ident)? => ($($fat_message:expr),*));*,
     $( $errno:ident$(::$errno_suf:ident)? => ($error_value:ident$(($inner_value:expr))?, $($message:expr),*)),*) => {
        match $errno_source {
            $($suc_errno$(::$suc_errno_suf)? => { $success_value } ),*
            $($fat_errno$(::$fat_errno_suf)? => { iceoryx2_bb_log::fatal_panic!(from $origin, $($fat_message),*); } ),*
            $($errno$(::$errno_suf)? => { iceoryx2_bb_log::fail!(from $origin, with $error_type::$error_value$(($inner_value))?, $($message),*); } ),*
        }
    };

}
