// Copyright 2026 the ruthere Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use alloc::vec::Vec;

use ruthere_core::{ExtensionFacet, PresenceKey};

use crate::{InMemoryStore, StoreChange, VisibilityPolicy};

/// A local watcher cursor over retained store changes.
///
/// The cursor owns only a store sequence position. It does not own watcher
/// identity, filtering policy, push delivery, or async behavior.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WatcherCursor {
    sequence: u64,
}

impl WatcherCursor {
    /// Creates a watcher cursor that starts at sequence `0`.
    #[must_use]
    pub const fn new() -> Self {
        Self { sequence: 0 }
    }

    /// Creates a watcher cursor that starts at the provided sequence.
    #[must_use]
    pub const fn from_sequence(sequence: u64) -> Self {
        Self { sequence }
    }

    /// Returns the currently retained sequence position.
    #[must_use]
    pub const fn sequence(self) -> u64 {
        self.sequence
    }

    /// Returns `true` when the store has retained changes beyond the cursor.
    #[must_use]
    pub fn has_pending<S, C, R, I, V, E>(self, store: &InMemoryStore<S, C, R, I, V, E>) -> bool
    where
        S: PresenceKey,
        C: PresenceKey,
        R: PresenceKey,
        I: PresenceKey,
        V: Clone,
        E: ExtensionFacet,
    {
        store.has_changes_since(self.sequence)
    }

    /// Returns `true` when the store has retained visible changes beyond the
    /// cursor.
    #[must_use]
    pub fn has_pending_visible<S, C, R, I, V, E, P>(
        self,
        store: &InMemoryStore<S, C, R, I, V, E>,
        visibility: &P,
    ) -> bool
    where
        S: PresenceKey,
        C: PresenceKey,
        R: PresenceKey,
        I: PresenceKey,
        V: Clone,
        E: ExtensionFacet,
        P: VisibilityPolicy<V>,
    {
        store.has_visible_changes_since(self.sequence, visibility)
    }

    /// Drains retained changes beyond the cursor and advances to the newest
    /// returned sequence.
    #[must_use]
    pub fn poll<S, C, R, I, V, E>(
        &mut self,
        store: &InMemoryStore<S, C, R, I, V, E>,
    ) -> Vec<StoreChange<S, C, R, I, V, E>>
    where
        S: PresenceKey,
        C: PresenceKey,
        R: PresenceKey,
        I: PresenceKey,
        V: Clone,
        E: ExtensionFacet,
    {
        let changes = store.changes_since(self.sequence);
        if let Some(last) = changes.last() {
            self.sequence = last.sequence;
        }
        changes
    }

    /// Drains retained visible changes beyond the cursor and advances to the
    /// newest visible sequence.
    #[must_use]
    pub fn poll_visible<S, C, R, I, V, E, P>(
        &mut self,
        store: &InMemoryStore<S, C, R, I, V, E>,
        visibility: &P,
    ) -> Vec<StoreChange<S, C, R, I, V, E>>
    where
        S: PresenceKey,
        C: PresenceKey,
        R: PresenceKey,
        I: PresenceKey,
        V: Clone,
        E: ExtensionFacet,
        P: VisibilityPolicy<V>,
    {
        let changes = store.changes_since_visible(self.sequence, visibility);
        if let Some(last) = changes.last() {
            self.sequence = last.sequence;
        }
        changes
    }
}

impl From<u64> for WatcherCursor {
    fn from(sequence: u64) -> Self {
        Self::from_sequence(sequence)
    }
}

impl From<WatcherCursor> for u64 {
    fn from(cursor: WatcherCursor) -> Self {
        cursor.sequence()
    }
}
