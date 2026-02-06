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
#![allow(dead_code)]

/// A utility oneshot channel for loom test.
/// Reference: https://github.com/tokio-rs/tokio/blob/064181f386414a543c0819af8c65866bbd879895/tokio/src/runtime/tests/loom_oneshot.rs
use loom::sync::Arc;
use loom::sync::Mutex;
use loom::sync::Notify;

pub(crate) fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(Inner {
        notify: Notify::new(),
        value: Mutex::new(None),
    });

    let tx = Sender {
        inner: inner.clone(),
    };
    let rx = Receiver { inner };

    (tx, rx)
}

pub(crate) struct Sender<T> {
    inner: Arc<Inner<T>>,
}

pub(crate) struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

struct Inner<T> {
    notify: Notify,
    value: Mutex<Option<T>>,
}

impl<T> Sender<T> {
    pub(crate) fn send(self, value: T) {
        *self.inner.value.lock().unwrap() = Some(value);
        self.inner.notify.notify();
    }
}

impl<T> Receiver<T> {
    pub(crate) fn recv(self) -> T {
        loop {
            if let Some(v) = self.inner.value.lock().unwrap().take() {
                return v;
            }

            self.inner.notify.wait();
        }
    }
}
