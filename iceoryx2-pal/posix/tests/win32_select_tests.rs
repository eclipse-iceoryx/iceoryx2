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

#[cfg(target_os = "windows")]
mod win32_select {
    use iceoryx2_pal_posix::posix::{settings::FD_SET_CAPACITY, *};
    use iceoryx2_pal_testing::assert_that;
    use win32_handle_translator::*;

    #[test]
    fn fd_set_capacity_correct() {
        let sut = fd_set::new_zeroed();
        assert_that!(FD_SET_CAPACITY, eq sut.fd_array.len());
    }

    #[test]
    fn fd_set_setting_fds_works() {
        let mut sut = fd_set::new_zeroed();

        let mut socket_fd = vec![];
        for i in 0..FD_SET_CAPACITY {
            socket_fd.push(HandleTranslator::get_instance().add(FdHandleEntry::Socket(
                SocketHandle {
                    fd: i,
                    recv_timeout: None,
                    send_timeout: None,
                },
            )));
        }

        for fd in socket_fd {
            assert_that!(unsafe {FD_ISSET(fd, &sut)}, eq false);
            unsafe { FD_SET(fd, &mut sut) }
            assert_that!(unsafe {FD_ISSET(fd, &sut)}, eq true);
        }
    }

    #[test]
    fn fd_set_clear_works() {
        let mut sut = fd_set::new_zeroed();

        let mut socket_fd = vec![];
        for i in 0..FD_SET_CAPACITY {
            socket_fd.push(HandleTranslator::get_instance().add(FdHandleEntry::Socket(
                SocketHandle {
                    fd: i,
                    recv_timeout: None,
                    send_timeout: None,
                },
            )));
        }

        for fd in &socket_fd {
            unsafe { FD_SET(*fd, &mut sut) }
        }

        unsafe { FD_ZERO(&mut sut) };

        for fd in &socket_fd {
            assert_that!(unsafe {FD_ISSET(*fd, &sut)}, eq false);
        }
    }

    #[test]
    fn fd_set_unsetting_fds_front_to_back_works() {
        let mut sut = fd_set::new_zeroed();

        let mut socket_fd = vec![];
        for i in 0..FD_SET_CAPACITY {
            socket_fd.push(HandleTranslator::get_instance().add(FdHandleEntry::Socket(
                SocketHandle {
                    fd: i,
                    recv_timeout: None,
                    send_timeout: None,
                },
            )));
        }

        for fd in &socket_fd {
            unsafe { FD_SET(*fd, &mut sut) }
        }

        for fd in &socket_fd {
            assert_that!(unsafe {FD_ISSET(*fd, &sut)}, eq true);
            unsafe { FD_CLR(*fd, &mut sut) }
            assert_that!(unsafe {FD_ISSET(*fd, &sut)}, eq false);
        }
    }

    #[test]
    fn fd_set_unsetting_fds_back_to_front_works() {
        let mut sut = fd_set::new_zeroed();

        let mut socket_fd = vec![];
        for i in 0..FD_SET_CAPACITY {
            socket_fd.push(HandleTranslator::get_instance().add(FdHandleEntry::Socket(
                SocketHandle {
                    fd: i,
                    recv_timeout: None,
                    send_timeout: None,
                },
            )));
        }

        for fd in &socket_fd {
            unsafe { FD_SET(*fd, &mut sut) }
        }

        for fd in socket_fd.iter().rev() {
            assert_that!(unsafe {FD_ISSET(*fd, &sut)}, eq true);
            unsafe { FD_CLR(*fd, &mut sut) }
            assert_that!(unsafe {FD_ISSET(*fd, &sut)}, eq false);
        }
    }
}
