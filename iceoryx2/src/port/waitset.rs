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

use std::{marker::PhantomData, rc::Rc};

use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_posix::{
    file_descriptor::FileDescriptor, file_descriptor_set::SynchronousMultiplexing,
};

use crate::service;

use super::{event_id::EventId, listener::Listener};

pub struct AttachmentId(FileDescriptor);

pub struct Attachment<'a, T> {
    id: AttachmentId,
    waitset: &'a WaitSet,
    this: Rc<T>,
}

impl<'a, T> Attachment<'a, T> {
    fn release(self) -> T {
        todo!()
    }

    fn originates_from(&self, this: &T) -> bool {
        todo!()
    }

    fn get(&self) -> &T {
        todo!()
    }

    fn get_mut(&mut self) -> &mut T {
        todo!()
    }

    fn call(&self) {
        todo!()
    }
}

pub trait Listen {}

pub struct WaitSet {
    attachments: Vec<Rc<dyn SynchronousMultiplexing>>,
}

// can only be created through node to have memory available and to be able to use Rc

impl WaitSet {
    pub fn attach<L: Listen + SynchronousMultiplexing>(&mut self, listener: L) -> Attachment<L> {
        todo!()
    }

    pub fn attach_custom<T: SynchronousMultiplexing>(&mut self, custom: T) -> Attachment<T> {
        todo!()
    }

    pub fn attach_fn<L: Listen, F: FnMut(&L)>(&mut self, listener: L, fn_call: F) -> Attachment<L> {
        todo!()
    }

    pub fn attach_custom_fn<T: SynchronousMultiplexing, F: FnMut(&T)>(
        &mut self,
        custom: T,
        fn_call: F,
    ) -> Attachment<T> {
        todo!()
    }

    fn try_wait<F: FnMut(AttachmentId) -> CallbackProgression>(&self, fn_call: F) {
        todo!()
    }

    fn try_wait_one(&self) -> (AttachmentId) {
        todo!()
    }

    fn try_wait_fn(&self) {
        todo!()
    }

    pub fn capacity() -> usize {
        todo!()
    }

    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn is_empty(&self) -> bool {
        todo!()
    }
}
