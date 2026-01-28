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

#![cfg(all(loom, test, feature = "std"))]

extern crate iceoryx2_bb_loggers;

use loom::sync::Arc;
use loom::thread;

use iceoryx2_bb_lock_free::spsc::index_queue::*;

mod loom_oneshot;
use loom_oneshot::channel;

#[test]
fn spsc_index_queue_loom_tests() {
    loom::model(|| {
        const CAPACITY: usize = 4;
        let queue = Arc::new(FixedSizeIndexQueue::<CAPACITY>::new());
        let (sender, receiver) = channel();

        let queue_producer = Arc::clone(&queue);
        let queue_consumer = Arc::clone(&queue);

        let producer = thread::spawn(move || {
            let mut producer = queue_producer.acquire_producer().unwrap();
            assert!(producer.push(42));
            sender.send(42);
        });

        let consumer = thread::spawn(move || {
            receiver.recv();
            let mut consumer = queue_consumer.acquire_consumer().unwrap();
            assert_eq!(consumer.pop().unwrap(), 42);
        });

        producer.join().unwrap();
        consumer.join().unwrap();
    });
}
