// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

extern crate iceoryx2_bb_loggers;

#[cfg(target_os = "linux")]
pub mod tests {

    use std::{
        sync::{atomic::Ordering, Barrier},
        time::Instant,
    };

    use iceoryx2_bb_concurrency::atomic::AtomicBool;
    use iceoryx2_bb_linux::epoll::*;
    use iceoryx2_bb_posix::{
        file_descriptor::FileDescriptorBased,
        process::Process,
        signal::{FetchableSignal, SignalHandler},
        socket_pair::StreamingSocket,
        user::User,
    };
    use iceoryx2_bb_testing::{assert_that, watchdog::Watchdog};

    const TIMEOUT: core::time::Duration = core::time::Duration::from_millis(50);

    #[test]
    fn attaching_a_fd_increases_len() {
        const NUMBER_OF_ATTACHMENTS: usize = 10;
        let sut = EpollBuilder::new().create().unwrap();

        let mut sockets = vec![];

        for _ in 0..NUMBER_OF_ATTACHMENTS / 2 {
            let (socket_1, socket_2) = StreamingSocket::create_pair().unwrap();
            sockets.push(socket_1);
            sockets.push(socket_2);
        }

        let mut guards = vec![];
        for n in 0..NUMBER_OF_ATTACHMENTS {
            assert_that!(sut.len(), eq n);
            guards.push(sut.add(sockets[n].file_descriptor()).attach().unwrap());
        }
    }

    #[test]
    fn when_guard_goes_out_of_scope_it_detaches_fd() {
        const NUMBER_OF_ATTACHMENTS: usize = 12;
        let sut = EpollBuilder::new().create().unwrap();

        let mut sockets = vec![];

        for _ in 0..NUMBER_OF_ATTACHMENTS / 2 {
            let (socket_1, socket_2) = StreamingSocket::create_pair().unwrap();
            sockets.push(socket_1);
            sockets.push(socket_2);
        }

        let mut guards = vec![];
        for n in 0..NUMBER_OF_ATTACHMENTS {
            guards.push(sut.add(sockets[n].file_descriptor()).attach().unwrap());
        }

        for n in 0..NUMBER_OF_ATTACHMENTS {
            assert_that!(sut.len(), eq NUMBER_OF_ATTACHMENTS - n);
            guards.pop();
        }
        assert_that!(sut.len(), eq 0);
    }

    #[test]
    fn attaching_one_fd_and_triggering_ready_to_read_works() {
        let (socket_1, socket_2) = StreamingSocket::create_pair().unwrap();
        let sut = EpollBuilder::new().create().unwrap();

        let _guard = sut
            .add(socket_1.file_descriptor())
            .event_type(EventType::ReadyToRead)
            .attach()
            .unwrap();

        let mut callback_was_called = false;
        assert_that!(sut.try_wait(|_| {callback_was_called = true;}).unwrap(), eq 0);
        assert_that!(callback_was_called, eq false);

        socket_2.try_send(b"hello").unwrap();

        let mut callback_was_called = false;
        let number_of_triggers = sut
            .try_wait(|event| {
                if let EpollEvent::FileDescriptor(fdev) = event {
                    assert_that!(fdev.originates_from(socket_1.file_descriptor()), eq true );
                    assert_that!(fdev.has_event(EventType::ReadyToRead), eq true);
                    assert_that!(fdev.has_event(EventType::ConnectionClosed), eq false);
                } else {
                    assert_that!(true, eq false);
                }
                callback_was_called = true;
            })
            .unwrap();

        assert_that!(number_of_triggers, eq 1);
        assert_that!(callback_was_called, eq true);
    }

    #[test]
    fn attaching_one_fd_and_triggering_connection_closed_works() {
        let (socket_1, socket_2) = StreamingSocket::create_pair().unwrap();
        let sut = EpollBuilder::new().create().unwrap();

        let _guard = sut
            .add(socket_1.file_descriptor())
            .event_type(EventType::ConnectionClosed)
            .attach()
            .unwrap();

        socket_2.try_send(b"hello").unwrap();
        let mut callback_was_called = false;
        let number_of_triggers = sut
            .try_wait(|_| {
                callback_was_called = true;
            })
            .unwrap();
        assert_that!(number_of_triggers, eq 0);
        assert_that!(callback_was_called, eq false);

        drop(socket_2);

        let mut callback_was_called = false;
        let number_of_triggers = sut
            .try_wait(|event| {
                if let EpollEvent::FileDescriptor(fdev) = event {
                    assert_that!(fdev.originates_from(socket_1.file_descriptor()), eq true );
                    assert_that!(fdev.has_event(EventType::ConnectionClosed), eq true);
                    assert_that!(fdev.has_event(EventType::ReadyToRead), eq false);
                } else {
                    assert_that!(true, eq false);
                }
                callback_was_called = true;
            })
            .unwrap();

        assert_that!(number_of_triggers, eq 1);
        assert_that!(callback_was_called, eq true);
    }

    #[test]
    fn when_guard_is_removed_fd_no_longer_triggers() {
        let (socket_1, socket_2) = StreamingSocket::create_pair().unwrap();
        let sut = EpollBuilder::new().create().unwrap();

        let guard = sut
            .add(socket_1.file_descriptor())
            .event_type(EventType::ReadyToRead)
            .attach()
            .unwrap();

        let mut callback_was_called = false;
        assert_that!(sut.try_wait(|_| {callback_was_called = true;}).unwrap(), eq 0);
        assert_that!(callback_was_called, eq false);

        drop(guard);
        socket_2.try_send(b"hello").unwrap();

        let mut callback_was_called = false;
        assert_that!(sut.try_wait(|_| {callback_was_called = true;}).unwrap(), eq 0);
        assert_that!(callback_was_called, eq false);
    }

    #[test]
    fn attaching_multiple_fd_and_triggering_many_ready_to_read_works() {
        const NUMBER_OF_ATTACHMENTS: usize = 12;
        let sut = EpollBuilder::new().create().unwrap();

        let mut sockets = vec![];
        let mut fd_values = vec![];

        for _ in 0..NUMBER_OF_ATTACHMENTS / 2 {
            let (socket_1, socket_2) = StreamingSocket::create_pair().unwrap();
            fd_values.push(unsafe { socket_1.file_descriptor().native_handle() });
            fd_values.push(unsafe { socket_2.file_descriptor().native_handle() });
            sockets.push(socket_1);
            sockets.push(socket_2);
        }

        let mut guards = vec![];
        for n in 0..NUMBER_OF_ATTACHMENTS {
            guards.push(
                sut.add(sockets[n].file_descriptor())
                    .event_type(EventType::ReadyToRead)
                    .attach()
                    .unwrap(),
            );
        }

        for socket in &sockets {
            socket.try_send(b"fuu").unwrap();
        }

        let mut callback_counter = 0;
        let number_of_triggers = sut
            .try_wait(|_| {
                callback_counter += 1;
            })
            .unwrap();
        assert_that!(number_of_triggers, eq NUMBER_OF_ATTACHMENTS);
        assert_that!(callback_counter, eq NUMBER_OF_ATTACHMENTS);
    }

    #[test]
    fn try_wait_does_not_block() {
        let (socket_1, _socket_2) = StreamingSocket::create_pair().unwrap();
        let sut = EpollBuilder::new().create().unwrap();

        let _guard = sut
            .add(socket_1.file_descriptor())
            .event_type(EventType::ReadyToRead)
            .attach()
            .unwrap();

        let mut callback_was_called = false;
        assert_that!(sut.try_wait(|_| {callback_was_called = true;}).unwrap(), eq 0);
        assert_that!(callback_was_called, eq false);
    }

    #[test]
    fn timed_wait_blocks_for_at_least_timeout() {
        let _watchdog = Watchdog::new();
        let (socket_1, _socket_2) = StreamingSocket::create_pair().unwrap();
        let sut = EpollBuilder::new().create().unwrap();

        let _guard = sut
            .add(socket_1.file_descriptor())
            .event_type(EventType::ReadyToRead)
            .attach()
            .unwrap();

        let start = Instant::now();
        let mut callback_was_called = false;
        assert_that!(sut.timed_wait(|_| {callback_was_called = true;}, TIMEOUT).unwrap(), eq 0);
        assert_that!(callback_was_called, eq false);
        assert_that!(start.elapsed(), time_at_least TIMEOUT);
    }

    #[test]
    fn timed_wait_wakes_up_by_trigger() {
        let _watchdog = Watchdog::new();
        let (socket_1, socket_2) = StreamingSocket::create_pair().unwrap();
        let sut = EpollBuilder::new().create().unwrap();
        let _guard = sut
            .add(socket_1.file_descriptor())
            .event_type(EventType::ReadyToRead)
            .attach()
            .unwrap();

        let callback_was_called = AtomicBool::new(false);
        let barrier = Barrier::new(2);
        std::thread::scope(|s| {
            s.spawn(|| {
                barrier.wait();
                assert_that!(sut.timed_wait(|_| {callback_was_called.store(true, Ordering::Relaxed);}, core::time::Duration::from_secs(12300)).unwrap(), eq 1);
            });

            barrier.wait();
            std::thread::sleep(TIMEOUT);
            assert_that!(callback_was_called.load(Ordering::Relaxed), eq false);

            socket_2.try_send(b"hello").unwrap();
            // thread should wake up now, if not the watchdog will let the unit test fail
        });
    }

    #[test]
    fn blocking_wait_wakes_up_by_trigger() {
        let _watchdog = Watchdog::new();
        let (socket_1, socket_2) = StreamingSocket::create_pair().unwrap();
        let sut = EpollBuilder::new().create().unwrap();
        let _guard = sut
            .add(socket_1.file_descriptor())
            .event_type(EventType::ReadyToRead)
            .attach()
            .unwrap();

        let callback_was_called = AtomicBool::new(false);
        let barrier = Barrier::new(2);
        std::thread::scope(|s| {
            s.spawn(|| {
                barrier.wait();
                assert_that!(sut.blocking_wait(|_| {callback_was_called.store(true, Ordering::Relaxed);}).unwrap(), eq 1);
            });

            barrier.wait();
            std::thread::sleep(TIMEOUT);
            assert_that!(callback_was_called.load(Ordering::Relaxed), eq false);

            socket_2.try_send(b"hello").unwrap();
            // thread should wake up now, if not the watchdog will let the unit test fail
        });
    }

    #[test]
    fn signals_can_be_received() {
        let _watchdog = Watchdog::new();
        let (socket_1, _socket_2) = StreamingSocket::create_pair().unwrap();
        let sut = EpollBuilder::new()
            .handle_signal(FetchableSignal::UserDefined1)
            .create()
            .unwrap();
        let _guard = sut
            .add(socket_1.file_descriptor())
            .event_type(EventType::ReadyToRead)
            .attach()
            .unwrap();

        let callback_was_called = AtomicBool::new(false);
        let barrier = Barrier::new(2);
        std::thread::scope(|s| {
            let t = s.spawn(|| {
                barrier.wait();
                assert_that!(sut.blocking_wait(|event| {
                if let EpollEvent::Signal(signal) = event {
                    assert_that!(signal.signal(), eq FetchableSignal::UserDefined1);
                    assert_that!(signal.origin_uid(), eq User::from_self().unwrap().uid());
                    assert_that!(signal.origin_pid(), eq Process::from_self().id());
                } else {
                    assert_that!(true, eq false);
                }
                callback_was_called.store(true, Ordering::Relaxed);
            }).unwrap(), eq 1);
            });

            barrier.wait();
            std::thread::sleep(TIMEOUT);
            assert_that!(callback_was_called.load(Ordering::Relaxed), eq false);

            while !t.is_finished() {
                SignalHandler::call_and_fetch(|| {
                    Process::from_self()
                        .send_signal(FetchableSignal::UserDefined1.into())
                        .unwrap();
                });
            }

            // thread should wake up now, if not the watchdog will let the unit test fail
        });
    }
}
