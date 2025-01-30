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

//! Helper struct to mark things that were visited in a loop. For instance in an event loop where
//! all unvisited things shall be removed from a list.
//!
//! ```
//! use iceoryx2_bb_elementary::visitor::*;
//!
//! struct MyThing {
//!   some_property: u64,
//!   visitor_marker: VisitorMarker,
//! }
//!
//! impl Visitable for MyThing {
//!   fn visitor_marker(&self) -> &VisitorMarker {
//!     &self.visitor_marker
//!   }
//! }
//!
//! let mut my_things: Vec<MyThing> = Vec::new();
//!
//! let global_visitor = Visitor::new();
//! // in event loop
//! global_visitor.next_cycle();
//! for thing in &my_things {
//!   // visit only things with property 123
//!   if thing.some_property == 123 {
//!     global_visitor.visit(&thing.visitor_marker);
//!   }
//! }
//!
//! // add a new thing
//! my_things.push(MyThing { some_property: 456, visitor_marker: global_visitor.create_visited_marker() });
//!
//! // remove all non-visited things
//! my_things.retain(|thing| thing.was_visited_by(&global_visitor));
//! ```

use core::sync::atomic::Ordering;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU8;

/// The [`Visitor`] can visit any object that implements [`Visitable`]. When visiting element
/// cyclicly the cycle shall always start with [`Visitor::next_cycle()`].
pub struct Visitor(IoxAtomicU8);

impl Visitor {
    /// Creates a new [`Visitor`] object.
    pub fn new() -> Self {
        Self(IoxAtomicU8::new(0))
    }

    /// Creates a new [`VisitorMarker`] so that it is identified as [`Visitable::was_visited_by()`]
    /// this [`Visitor`].
    pub fn create_visited_marker(&self) -> VisitorMarker {
        VisitorMarker(IoxAtomicU8::new(self.0.load(Ordering::Relaxed)))
    }

    /// Creates a new [`VisitorMarker`] so that it is identified as not
    /// [`Visitable::was_visited_by()`] this [`Visitor`].
    pub fn create_unvisited_marker(&self) -> VisitorMarker {
        VisitorMarker(IoxAtomicU8::new(
            self.0.load(Ordering::Relaxed).wrapping_sub(1),
        ))
    }

    /// When using the [`Visitor`] to cyclicly visit things, then the [`Visitor`] is going into the
    /// next visiting cycle with this method.
    pub fn next_cycle(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    /// Visit a [`Visitable`] object. After this call the method [`Visitable::was_visited_by()`]
    /// returns true for this [`Visitor`].
    pub fn visit<T: Visitable>(&self, rhs: &T) {
        rhs.visitor_marker()
            .0
            .store(self.0.load(Ordering::Relaxed), Ordering::Relaxed);
    }
}

/// This tracks the mark of the [`Visitor`] when it is visited.
pub struct VisitorMarker(IoxAtomicU8);

impl Visitable for VisitorMarker {
    fn visitor_marker(&self) -> &VisitorMarker {
        self
    }
}

/// Identifies structs that can be visited by the [`Visitor`].
pub trait Visitable {
    /// Returns a reference to the underlying [`VisitorMarker`] of the [`Visitable`] struct.
    fn visitor_marker(&self) -> &VisitorMarker;

    /// Returns true if it was visited by the [`Visitor`] with [`Visitor::visit()`] in the current
    /// cycle.
    fn was_visited_by(&self, rhs: &Visitor) -> bool {
        self.visitor_marker().0.load(Ordering::Relaxed) == rhs.0.load(Ordering::Relaxed)
    }
}
