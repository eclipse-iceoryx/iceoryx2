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

use alloc::collections::BTreeMap;
use core::ops::DerefMut;

/// One finding, stamped with the update it was last recorded in.
struct Entry<V> {
    value: V,
    seen: u64,
}

/// The findings a backend has reported and not yet seen cleared. `K`
/// identifies what a finding is about, `V` is what was found about it.
pub struct Findings<K, V> {
    entries: BTreeMap<K, Entry<V>>,
    epoch: u64,
}

impl<K, V> Default for Findings<K, V> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            epoch: 0,
        }
    }
}

impl<K: Ord, V: PartialEq> Findings<K, V> {
    /// Begins an update, reporting through `report` each finding it
    /// records that was not held before or was held with another value.
    pub fn update<R: FnMut(&K, &V)>(&mut self, report: R) -> Update<K, V, &mut Self, R> {
        Update::new(self, report)
    }
}

/// One update of the findings. Committed, the findings it did not record
/// are cleared. Dropped without a commit, the findings held before stand.
pub struct Update<K, V, F: DerefMut<Target = Findings<K, V>>, R> {
    findings: F,
    report: R,
    epoch: u64,
}

impl<K: Ord, V: PartialEq, F: DerefMut<Target = Findings<K, V>>, R: FnMut(&K, &V)>
    Update<K, V, F, R>
{
    /// Begins an update of the findings `findings` borrows.
    pub fn new(mut findings: F, report: R) -> Self {
        findings.epoch = findings.epoch.wrapping_add(1);
        let epoch = findings.epoch;
        Self {
            findings,
            report,
            epoch,
        }
    }

    /// Records a finding, reporting it if new or found with another
    /// value.
    pub fn record(&mut self, key: K, value: V) {
        let epoch = self.epoch;
        match self.findings.entries.get_mut(&key) {
            Some(entry) if entry.value == value => entry.seen = epoch,
            _ => {
                (self.report)(&key, &value);
                self.findings
                    .entries
                    .insert(key, Entry { value, seen: epoch });
            }
        }
    }

    /// Ends the update, clearing the findings it did not record.
    pub fn commit(mut self) {
        let epoch = self.epoch;
        self.findings.entries.retain(|_, entry| entry.seen == epoch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec::Vec;

    use iceoryx2_bb_testing::assert_that;

    const KEY: u8 = 1;
    const VALUE: u8 = 10;
    const OTHER_VALUE: u8 = 11;

    /// Updates the findings to `found`, returning what was reported.
    fn refresh(sut: &mut Findings<u8, u8>, found: &[(u8, u8)]) -> Vec<(u8, u8)> {
        let mut reported = Vec::new();
        let mut update = sut.update(|key, value| reported.push((*key, *value)));
        for (key, value) in found {
            update.record(*key, *value);
        }
        update.commit();
        reported
    }

    #[test]
    fn a_new_finding_is_reported() {
        let mut sut = Findings::default();
        assert_that!(refresh(&mut sut, &[(KEY, VALUE)]), eq alloc::vec![(KEY, VALUE)]);
    }

    #[test]
    fn a_finding_held_with_the_same_value_is_not_reported_again() {
        let mut sut = Findings::default();
        refresh(&mut sut, &[(KEY, VALUE)]);
        assert_that!(refresh(&mut sut, &[(KEY, VALUE)]), len 0);
    }

    #[test]
    fn a_finding_held_with_another_value_is_reported_again() {
        let mut sut = Findings::default();
        refresh(&mut sut, &[(KEY, VALUE)]);
        assert_that!(refresh(&mut sut, &[(KEY, OTHER_VALUE)]), eq alloc::vec![(KEY, OTHER_VALUE)]);
    }

    #[test]
    fn a_finding_a_walk_does_not_record_is_cleared() {
        let mut sut = Findings::default();
        refresh(&mut sut, &[(KEY, VALUE)]);
        refresh(&mut sut, &[]);
        assert_that!(refresh(&mut sut, &[(KEY, VALUE)]), eq alloc::vec![(KEY, VALUE)]);
    }

    #[test]
    fn an_update_dropped_without_a_commit_clears_nothing() {
        let mut sut = Findings::default();
        refresh(&mut sut, &[(KEY, VALUE)]);

        let _ = sut.update(|_, _| {});

        assert_that!(refresh(&mut sut, &[(KEY, VALUE)]), len 0);
    }
}
