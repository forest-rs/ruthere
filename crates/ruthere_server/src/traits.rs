// Copyright 2026 the ruthere Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use alloc::vec::Vec;

use ruthere_core::{ExtensionFacet, Never, PresenceUpdate, Timestamp};

use crate::{StoreChange, VisibilityPolicy, WatcherCursor};

/// Write-side ingest and lifecycle contract for a presence server.
///
/// This trait captures the narrow "server receives updates and advances local
/// lifecycle state" seam without taking ownership of transport, auth, or
/// richer query behavior.
pub trait PresenceIngress<S, C, R, I, V, E = Never>
where
    E: ExtensionFacet,
{
    /// Receives one presence update and returns the assigned server-local
    /// sequence.
    fn receive(&mut self, update: PresenceUpdate<S, C, R, I, V, E>) -> u64;

    /// Returns the most recently assigned server-local sequence.
    fn last_sequence(&self) -> u64;

    /// Applies expiry at `now` and returns how many entries were removed.
    fn expire(&mut self, now: Timestamp) -> usize;

    /// Receives multiple presence updates and returns the assigned sequences.
    fn receive_all<It>(&mut self, updates: It) -> Vec<u64>
    where
        It: IntoIterator<Item = PresenceUpdate<S, C, R, I, V, E>>,
    {
        updates
            .into_iter()
            .map(|update| self.receive(update))
            .collect()
    }
}

/// Watch-side polling contract for a presence server.
///
/// This trait captures retained-change polling over server-local watcher
/// cursors without implying push delivery or observer identity.
pub trait PresenceWatch<S, C, R, I, V, E = Never>
where
    E: ExtensionFacet,
{
    /// Returns a watcher cursor that starts at the beginning of the retained
    /// change log.
    fn watcher_cursor(&self) -> WatcherCursor;

    /// Returns a watcher cursor positioned at the current sequence tail.
    fn watcher_cursor_from_current(&self) -> WatcherCursor;

    /// Returns `true` when the server has retained changes beyond the cursor.
    fn has_pending(&self, cursor: WatcherCursor) -> bool;

    /// Returns `true` when the server has retained visible changes beyond the
    /// cursor.
    fn has_pending_visible<P>(&self, cursor: WatcherCursor, visibility: &P) -> bool
    where
        P: VisibilityPolicy<V>;

    /// Drains retained changes beyond the cursor and advances it.
    fn poll(&self, cursor: &mut WatcherCursor) -> Vec<StoreChange<S, C, R, I, V, E>>;

    /// Drains retained visible changes beyond the cursor and advances it.
    fn poll_visible<P>(
        &self,
        cursor: &mut WatcherCursor,
        visibility: &P,
    ) -> Vec<StoreChange<S, C, R, I, V, E>>
    where
        P: VisibilityPolicy<V>;
}
