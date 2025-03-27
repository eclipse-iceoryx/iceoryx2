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

//! Helper struct to tag things cyclicly. For instance in an event loop where
//! all untagged things shall be removed from a list.
//!
//! ```
//! use iceoryx2_bb_elementary::cyclic_tagger::*;
//!
//! struct MyThing {
//!   some_property: u64,
//!   tag: Tag,
//! }
//!
//! impl Taggable for MyThing {
//!   fn tag(&self) -> &Tag {
//!     &self.tag
//!   }
//! }
//!
//! let mut my_things: Vec<MyThing> = Vec::new();
//!
//! let global_tagger = CyclicTagger::new();
//! // in event loop
//! global_tagger.next_cycle();
//! for thing in &my_things {
//!   // tag only things with property 123
//!   if thing.some_property == 123 {
//!     global_tagger.tag(&thing.tag);
//!   }
//! }
//!
//! // add a new thing
//! my_things.push(MyThing { some_property: 456, tag: global_tagger.create_tag() });
//!
//! // remove all non-taged things
//! my_things.retain(|thing| thing.was_tagged_by(&global_tagger));
//! ```

use core::sync::atomic::Ordering;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU8;

/// The [`CyclicTagger`] can tag any object that implements [`Taggable`]. When tagging elements
/// cyclicly the cycle shall always start with [`CyclicTagger::next_cycle()`].
#[derive(Debug, Default)]
pub struct CyclicTagger(IoxAtomicU8);

impl CyclicTagger {
    /// Creates a new [`CyclicTagger`] object.
    pub fn new() -> Self {
        CyclicTagger::default()
    }

    /// Creates a new [`Tag`] so that it is identified as [`Taggable::was_tagged_by()`].
    pub fn create_tag(&self) -> Tag {
        Tag(IoxAtomicU8::new(self.0.load(Ordering::Relaxed)))
    }

    /// Creates a new [`Tag`] so that it is identified as not
    /// [`Taggable::was_tagged_by()`] this [`CyclicTagger`].
    pub fn create_untagged_tag(&self) -> Tag {
        Tag(IoxAtomicU8::new(
            self.0.load(Ordering::Relaxed).wrapping_sub(1),
        ))
    }

    /// When using the [`CyclicTagger`] to cyclicly tag things, then the [`CyclicTagger`] is going
    /// into the next tagging cycle with this method.
    pub fn next_cycle(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    /// Tag a [`Taggable`] object. After this call the method [`Taggable::was_tagged_by()`]
    /// returns true for this [`CyclicTagger`].
    pub fn tag<T: Taggable>(&self, rhs: &T) {
        rhs.tag()
            .0
            .store(self.0.load(Ordering::Relaxed), Ordering::Relaxed);
    }
}

/// This tracks the mark of the [`CyclicTagger`] when it is tagged.
#[derive(Debug)]
pub struct Tag(IoxAtomicU8);

impl Taggable for Tag {
    fn tag(&self) -> &Tag {
        self
    }
}

/// Identifies structs that can be tagged by the [`CyclicTagger`].
pub trait Taggable {
    /// Returns a reference to the underlying [`Tag`] of the [`Taggable`] struct.
    fn tag(&self) -> &Tag;

    /// Returns true if it was tagged by the [`CyclicTagger`] with [`CyclicTagger::tag()`] in the current
    /// cycle.
    fn was_tagged_by(&self, rhs: &CyclicTagger) -> bool {
        self.tag().0.load(Ordering::Relaxed) == rhs.0.load(Ordering::Relaxed)
    }
}
