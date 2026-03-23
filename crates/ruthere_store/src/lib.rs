// Copyright 2026 the ruthere Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![no_std]

//! In-memory runtime store for `ruthere` presence updates.
//!
//! `ruthere_store` owns sequencing, reduction, and expiry for a local store.
//! It intentionally does not yet define subscriptions, watcher identity, or
//! transport integration.

extern crate alloc;

use alloc::vec::Vec;
use hashbrown::{HashMap, hash_map::Entry};
use ruthere_core::{
    Expiry, ExtensionFacet, FacetChange, Never, PresenceAddress, PresenceFacet, PresenceFacetKind,
    PresenceKey, PresenceSnapshot, PresenceUpdate, Timestamp, Visibility,
};

/// A fully qualified key for one stored presence entry.
///
/// `ruthere_store` stores state per addressed scope and origin so concurrent
/// publishers do not overwrite one another accidentally.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PresenceEntryKey<S, C, R, I> {
    /// The addressed presence scope.
    pub address: PresenceAddress<S, C, R>,
    /// The origin that produced the stored facts.
    pub origin: I,
}

impl<S, C, R, I> PresenceEntryKey<S, C, R, I> {
    /// Creates a new store entry key.
    #[must_use]
    pub const fn new(address: PresenceAddress<S, C, R>, origin: I) -> Self {
        Self { address, origin }
    }
}

#[derive(Clone, Debug)]
struct SnapshotState<V, E>
where
    E: ExtensionFacet,
{
    visibility: Visibility<V>,
    observed_at: Timestamp,
    expiry: Expiry,
    facets: HashMap<PresenceFacetKind<E::Kind>, PresenceFacet<E>>,
}

impl<V, E> SnapshotState<V, E>
where
    V: Clone,
    E: ExtensionFacet,
{
    fn new(visibility: Visibility<V>, observed_at: Timestamp, expiry: Expiry) -> Self {
        Self {
            visibility,
            observed_at,
            expiry,
            facets: HashMap::new(),
        }
    }

    fn apply_changes(&mut self, changes: Vec<FacetChange<E>>) {
        for change in changes {
            match change {
                FacetChange::Set(facet) => {
                    let kind = facet.kind();
                    self.facets.insert(kind, facet);
                }
                FacetChange::Clear(kind) => {
                    self.facets.remove(&kind);
                }
            }
        }
    }
}

/// An in-memory store for presence updates and snapshots.
#[derive(Clone, Debug)]
pub struct InMemoryStore<S, C, R, I, V, E = Never>
where
    S: PresenceKey,
    C: PresenceKey,
    R: PresenceKey,
    I: PresenceKey,
    V: Clone,
    E: ExtensionFacet,
{
    next_sequence: u64,
    entries: HashMap<PresenceEntryKey<S, C, R, I>, SnapshotState<V, E>>,
}

impl<S, C, R, I, V, E> Default for InMemoryStore<S, C, R, I, V, E>
where
    S: PresenceKey,
    C: PresenceKey,
    R: PresenceKey,
    I: PresenceKey,
    V: Clone,
    E: ExtensionFacet,
{
    fn default() -> Self {
        Self {
            next_sequence: 0,
            entries: HashMap::new(),
        }
    }
}

impl<S, C, R, I, V, E> InMemoryStore<S, C, R, I, V, E>
where
    S: PresenceKey,
    C: PresenceKey,
    R: PresenceKey,
    I: PresenceKey,
    V: Clone,
    E: ExtensionFacet,
{
    /// Creates an empty in-memory store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of stored presence entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` when the store has no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the most recently assigned store sequence.
    #[must_use]
    pub const fn last_sequence(&self) -> u64 {
        self.next_sequence
    }

    /// Publishes an update into the store and returns the assigned sequence.
    ///
    /// Publish order is authoritative within one store instance.
    pub fn publish(&mut self, update: PresenceUpdate<S, C, R, I, V, E>) -> u64 {
        let PresenceUpdate {
            address,
            origin,
            visibility,
            observed_at,
            expiry,
            changes,
        } = update;

        let key = PresenceEntryKey::new(address, origin);
        let state = match self.entries.entry(key) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                entry.insert(SnapshotState::new(visibility.clone(), observed_at, expiry))
            }
        };

        state.visibility = visibility;
        state.observed_at = observed_at;
        state.expiry = expiry;
        state.apply_changes(changes);

        self.next_sequence = self.next_sequence.saturating_add(1);
        self.next_sequence
    }

    /// Returns a materialized snapshot for one stored entry.
    #[must_use]
    pub fn snapshot(
        &self,
        key: &PresenceEntryKey<S, C, R, I>,
    ) -> Option<PresenceSnapshot<S, C, R, I, V, E>> {
        self.entries
            .get(key)
            .map(|state| self.materialize_snapshot(key, state))
    }

    /// Returns all stored snapshots for an addressed scope across origins.
    ///
    /// Snapshot order is unspecified.
    #[must_use]
    pub fn snapshots_for_address(
        &self,
        address: &PresenceAddress<S, C, R>,
    ) -> Vec<PresenceSnapshot<S, C, R, I, V, E>> {
        self.entries
            .iter()
            .filter_map(|(key, state)| {
                if &key.address == address {
                    Some(self.materialize_snapshot(key, state))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns all stored snapshots for a context across addresses and origins.
    ///
    /// Snapshot order is unspecified.
    #[must_use]
    pub fn snapshots_in_context(&self, context: &C) -> Vec<PresenceSnapshot<S, C, R, I, V, E>> {
        self.entries
            .iter()
            .filter_map(|(key, state)| {
                if &key.address.context == context {
                    Some(self.materialize_snapshot(key, state))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Removes expired entries and returns how many were pruned.
    pub fn expire(&mut self, now: Timestamp) -> usize {
        let initial_len = self.entries.len();
        self.entries
            .retain(|_, state| !state.expiry.is_expired_by(now));
        initial_len.saturating_sub(self.entries.len())
    }

    fn materialize_snapshot(
        &self,
        key: &PresenceEntryKey<S, C, R, I>,
        state: &SnapshotState<V, E>,
    ) -> PresenceSnapshot<S, C, R, I, V, E> {
        PresenceSnapshot {
            address: key.address.clone(),
            origin: key.origin.clone(),
            visibility: state.visibility.clone(),
            observed_at: state.observed_at,
            expiry: state.expiry,
            facets: state.facets.values().cloned().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::{InMemoryStore, PresenceEntryKey};
    use ruthere_core::{
        Activity, Availability, BuiltinFacet, BuiltinFacetKind, Expiry, ExtensionFacet,
        FacetChange, PresenceAddress, PresenceFacet, PresenceFacetKind, PresenceUpdate, Timestamp,
        Visibility,
    };

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    enum DemoFacet {
        Focus(u64),
    }

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    enum DemoFacetKind {
        Focus,
    }

    impl ExtensionFacet for DemoFacet {
        type Kind = DemoFacetKind;

        fn kind(&self) -> Self::Kind {
            match self {
                Self::Focus(..) => DemoFacetKind::Focus,
            }
        }
    }

    #[test]
    fn publish_materializes_and_clears_facets() {
        let mut store = InMemoryStore::<u64, u64, u64, u64, &'static str>::new();
        let address = PresenceAddress::new(7_u64, 9_u64, Some(3_u64));

        let sequence = store.publish(
            PresenceUpdate::new(
                address.clone(),
                11_u64,
                Visibility::Restricted("ops"),
                Timestamp::new(20),
                Expiry::At(Timestamp::new(50)),
            )
            .with_change(FacetChange::Set(PresenceFacet::Builtin(
                BuiltinFacet::Availability(Availability::Available),
            )))
            .with_change(FacetChange::Set(PresenceFacet::Builtin(
                BuiltinFacet::LastSeen(Timestamp::new(19)),
            ))),
        );

        assert_eq!(sequence, 1);

        store.publish(
            PresenceUpdate::new(
                address.clone(),
                11_u64,
                Visibility::Restricted("ops"),
                Timestamp::new(21),
                Expiry::At(Timestamp::new(55)),
            )
            .with_change(FacetChange::Set(PresenceFacet::Builtin(
                BuiltinFacet::Activity(Activity::Editing),
            )))
            .with_change(FacetChange::Clear(PresenceFacetKind::Builtin(
                BuiltinFacetKind::LastSeen,
            ))),
        );

        let snapshot = store
            .snapshot(&PresenceEntryKey::new(address, 11_u64))
            .expect("missing stored snapshot");

        assert_eq!(snapshot.availability(), Some(Availability::Available));
        assert_eq!(snapshot.activity(), Some(Activity::Editing));
        assert_eq!(snapshot.last_seen(), None);
        assert_eq!(snapshot.observed_at, Timestamp::new(21));
        assert_eq!(snapshot.expiry, Expiry::At(Timestamp::new(55)));
    }

    #[test]
    fn snapshots_for_address_return_all_origins() {
        let mut store = InMemoryStore::<u64, u64, u64, u64, ()>::new();
        let address = PresenceAddress::new(1_u64, 2_u64, None::<u64>);

        for origin in [10_u64, 11_u64] {
            store.publish(
                PresenceUpdate::new(
                    address.clone(),
                    origin,
                    Visibility::Public,
                    Timestamp::new(origin),
                    Expiry::Never,
                )
                .with_change(FacetChange::Set(PresenceFacet::Builtin(
                    BuiltinFacet::Availability(Availability::Available),
                ))),
            );
        }

        let snapshots = store.snapshots_for_address(&address);

        assert_eq!(snapshots.len(), 2);
        assert!(
            snapshots
                .iter()
                .all(|snapshot| snapshot.availability() == Some(Availability::Available))
        );
    }

    #[test]
    fn snapshots_in_context_filter_to_one_context() {
        let mut store = InMemoryStore::<u64, u64, u64, u64, ()>::new();

        store.publish(
            PresenceUpdate::new(
                PresenceAddress::new(1_u64, 100_u64, None::<u64>),
                10_u64,
                Visibility::Public,
                Timestamp::new(1),
                Expiry::Never,
            )
            .with_change(FacetChange::Set(PresenceFacet::Builtin(
                BuiltinFacet::Activity(Activity::Observing),
            ))),
        );

        store.publish(
            PresenceUpdate::new(
                PresenceAddress::new(2_u64, 200_u64, None::<u64>),
                20_u64,
                Visibility::Public,
                Timestamp::new(2),
                Expiry::Never,
            )
            .with_change(FacetChange::Set(PresenceFacet::Builtin(
                BuiltinFacet::Availability(Availability::Away),
            ))),
        );

        let snapshots = store.snapshots_in_context(&100_u64);

        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].address.context, 100_u64);
        assert_eq!(snapshots[0].activity(), Some(Activity::Observing));
    }

    #[test]
    fn expire_prunes_whole_entries() {
        let mut store = InMemoryStore::<u64, u64, u64, u64, (), DemoFacet>::new();

        store.publish(
            PresenceUpdate::new(
                PresenceAddress::new(1_u64, 10_u64, None::<u64>),
                100_u64,
                Visibility::Public,
                Timestamp::new(5),
                Expiry::At(Timestamp::new(8)),
            )
            .with_change(FacetChange::Set(PresenceFacet::Extension(
                DemoFacet::Focus(99),
            ))),
        );

        store.publish(
            PresenceUpdate::new(
                PresenceAddress::new(2_u64, 10_u64, None::<u64>),
                101_u64,
                Visibility::Private,
                Timestamp::new(6),
                Expiry::Never,
            )
            .with_change(FacetChange::Set(PresenceFacet::Builtin(
                BuiltinFacet::Availability(Availability::Busy),
            ))),
        );

        let removed = store.expire(Timestamp::new(8));
        let survivors = store.snapshots_in_context(&10_u64);
        let survivor_subjects: Vec<u64> = survivors
            .iter()
            .map(|snapshot| snapshot.address.subject)
            .collect();

        assert_eq!(removed, 1);
        assert_eq!(store.len(), 1);
        assert_eq!(survivor_subjects, Vec::from([2_u64]));
        assert!(
            survivors[0]
                .facets
                .iter()
                .all(|facet| *facet != PresenceFacet::Extension(DemoFacet::Focus(99)))
        );
    }
}
