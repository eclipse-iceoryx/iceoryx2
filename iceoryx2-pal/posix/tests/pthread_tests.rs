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

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod pthread {
    use core::ffi::CStr;
    use iceoryx2_pal_posix::posix::{self, Errno, MemZeroedStruct};

    const SUCCESS: posix::int = Errno::ESUCCES as _;
    const ERANGE: posix::int = Errno::ERANGE as _;
    const NAME_BUFFER_SIZE: usize = 16;
    const TOO_SMALL_BUFFER_SIZE: usize = NAME_BUFFER_SIZE - 1;
    const UNWRITTEN: posix::c_char = 1;
    const NAME: &CStr = c"fifteen-chars-a";
    const TOO_LONG_NAME: &CStr = c"sixteen-chars-ab";

    struct Run {
        names: &'static [&'static CStr],
        buffer_size: usize,
        set_results: Vec<posix::int>,
        get_result: Option<posix::int>,
        buffer: [posix::c_char; NAME_BUFFER_SIZE],
    }

    extern "C" fn set_and_get_name(arg: *mut posix::void) -> *mut posix::void {
        let run = unsafe { &mut *arg.cast::<Run>() };
        let thread = unsafe { posix::pthread_self() };
        for name in run.names {
            run.set_results
                .push(unsafe { posix::pthread_setname_np(thread, name.as_ptr()) });
        }
        run.get_result = Some(unsafe {
            posix::pthread_getname_np(thread, run.buffer.as_mut_ptr(), run.buffer_size)
        });
        core::ptr::null_mut()
    }

    fn run_in_thread(names: &'static [&'static CStr], buffer_size: usize) -> Run {
        let mut run = Run {
            names,
            buffer_size,
            set_results: Vec::new(),
            get_result: None,
            buffer: [UNWRITTEN; NAME_BUFFER_SIZE],
        };
        let mut attr = posix::pthread_attr_t::new_zeroed();
        let mut thread = posix::pthread_t::new_zeroed();
        unsafe {
            assert_eq!(posix::pthread_attr_init(&mut attr), SUCCESS);
            assert_eq!(
                posix::pthread_create(
                    &mut thread,
                    &attr,
                    set_and_get_name,
                    (&mut run as *mut Run).cast()
                ),
                SUCCESS
            );
            assert_eq!(posix::pthread_join(thread, core::ptr::null_mut()), SUCCESS);
            assert_eq!(posix::pthread_attr_destroy(&mut attr), SUCCESS);
        }
        run
    }

    fn terminated(name: &CStr) -> Vec<posix::c_char> {
        name.to_bytes_with_nul()
            .iter()
            .map(|&c| c as posix::c_char)
            .collect()
    }

    #[test]
    fn name_that_fits_is_read_back_with_terminator() {
        let run = run_in_thread(&[NAME], NAME_BUFFER_SIZE);

        assert_eq!(run.set_results, [SUCCESS]);
        assert_eq!(run.get_result, Some(SUCCESS));
        assert_eq!(run.buffer[..], terminated(NAME)[..]);
    }

    #[test]
    fn name_that_does_not_fit_fails_and_keeps_previous_name() {
        let run = run_in_thread(&[NAME, TOO_LONG_NAME], NAME_BUFFER_SIZE);

        assert_eq!(run.set_results, [SUCCESS, ERANGE]);
        assert_eq!(run.get_result, Some(SUCCESS));
        assert_eq!(run.buffer[..], terminated(NAME)[..]);
    }

    #[test]
    fn buffer_without_room_for_terminator_fails() {
        let run = run_in_thread(&[], TOO_SMALL_BUFFER_SIZE);

        assert_eq!(run.get_result, Some(ERANGE));
    }
}
