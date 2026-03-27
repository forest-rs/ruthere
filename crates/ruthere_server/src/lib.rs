// Copyright 2026 the ruthere Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![no_std]

//! Process-local server seam for `ruthere`.
//!
//! `ruthere_server` receives presence updates, manages expiry against an
//! internal store, and provides watcher polling helpers. It intentionally does
//! not own transport, authentication, or push delivery.
//!
//! The crate also exposes narrow `PresenceIngress` and `PresenceWatch`
//! contracts for runtimes that want the current server seam without depending
//! on the concrete `PresenceServer` type directly.

extern crate alloc;

use alloc::vec::Vec;

use ruthere_core::{ExtensionFacet, Never, PresenceKey, PresenceUpdate, Timestamp};
use ruthere_store::InMemoryStore;
pub use ruthere_store::{StoreChange, StoreChangeKind, VisibilityPolicy, WatcherCursor};
mod traits;
pub use traits::{PresenceIngress, PresenceWatch};

/// A small process-local server over an internal in-memory store.
#[derive(Clone, Debug)]
pub struct PresenceServer<S, C, R, I, V, E = Never>
where
    S: PresenceKey,
    C: PresenceKey,
    R: PresenceKey,
    I: PresenceKey,
    V: Clone,
    E: ExtensionFacet,
{
    store: InMemoryStore<S, C, R, I, V, E>,
}

impl<S, C, R, I, V, E> Default for PresenceServer<S, C, R, I, V, E>
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
            store: InMemoryStore::default(),
        }
    }
}

impl<S, C, R, I, V, E> PresenceServer<S, C, R, I, V, E>
where
    S: PresenceKey,
    C: PresenceKey,
    R: PresenceKey,
    I: PresenceKey,
    V: Clone,
    E: ExtensionFacet,
{
    /// Creates an empty process-local server.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of stored entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Returns `true` when the server currently has no stored entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Returns the most recently assigned store sequence.
    #[must_use]
    pub const fn last_sequence(&self) -> u64 {
        self.store.last_sequence()
    }

    /// Returns a read-only view of the underlying store.
    #[must_use]
    pub const fn store(&self) -> &InMemoryStore<S, C, R, I, V, E> {
        &self.store
    }

    /// Consumes the server and returns the underlying store.
    #[must_use]
    pub fn into_store(self) -> InMemoryStore<S, C, R, I, V, E> {
        self.store
    }

    /// Receives one presence update and returns the assigned sequence.
    pub fn receive(&mut self, update: PresenceUpdate<S, C, R, I, V, E>) -> u64 {
        self.store.publish(update)
    }

    /// Receives multiple presence updates and returns the assigned sequences.
    #[must_use]
    pub fn receive_all<It>(&mut self, updates: It) -> Vec<u64>
    where
        It: IntoIterator<Item = PresenceUpdate<S, C, R, I, V, E>>,
    {
        updates
            .into_iter()
            .map(|update| self.receive(update))
            .collect()
    }

    /// Applies expiry and returns how many entries were removed.
    pub fn expire(&mut self, now: Timestamp) -> usize {
        self.store.expire(now)
    }

    /// Returns a watcher cursor that starts at the beginning of the retained
    /// change log.
    #[must_use]
    pub const fn watcher_cursor(&self) -> WatcherCursor {
        WatcherCursor::new()
    }

    /// Returns a watcher cursor positioned at the current sequence tail.
    #[must_use]
    pub const fn watcher_cursor_from_current(&self) -> WatcherCursor {
        WatcherCursor::from_sequence(self.last_sequence())
    }

    /// Returns `true` when the server has retained changes beyond the cursor.
    #[must_use]
    pub fn has_pending(&self, cursor: WatcherCursor) -> bool {
        cursor.has_pending(&self.store)
    }

    /// Returns `true` when the server has retained visible changes beyond the
    /// cursor.
    #[must_use]
    pub fn has_pending_visible<P>(&self, cursor: WatcherCursor, visibility: &P) -> bool
    where
        P: VisibilityPolicy<V>,
    {
        cursor.has_pending_visible(&self.store, visibility)
    }

    /// Drains retained changes beyond the cursor and advances it.
    #[must_use]
    pub fn poll(&self, cursor: &mut WatcherCursor) -> Vec<StoreChange<S, C, R, I, V, E>> {
        cursor.poll(&self.store)
    }

    /// Drains retained visible changes beyond the cursor and advances it.
    #[must_use]
    pub fn poll_visible<P>(
        &self,
        cursor: &mut WatcherCursor,
        visibility: &P,
    ) -> Vec<StoreChange<S, C, R, I, V, E>>
    where
        P: VisibilityPolicy<V>,
    {
        cursor.poll_visible(&self.store, visibility)
    }
}

impl<S, C, R, I, V, E> PresenceIngress<S, C, R, I, V, E> for PresenceServer<S, C, R, I, V, E>
where
    S: PresenceKey,
    C: PresenceKey,
    R: PresenceKey,
    I: PresenceKey,
    V: Clone,
    E: ExtensionFacet,
{
    fn receive(&mut self, update: PresenceUpdate<S, C, R, I, V, E>) -> u64 {
        Self::receive(self, update)
    }

    fn last_sequence(&self) -> u64 {
        Self::last_sequence(self)
    }

    fn expire(&mut self, now: Timestamp) -> usize {
        Self::expire(self, now)
    }
}

impl<S, C, R, I, V, E> PresenceWatch<S, C, R, I, V, E> for PresenceServer<S, C, R, I, V, E>
where
    S: PresenceKey,
    C: PresenceKey,
    R: PresenceKey,
    I: PresenceKey,
    V: Clone,
    E: ExtensionFacet,
{
    fn watcher_cursor(&self) -> WatcherCursor {
        Self::watcher_cursor(self)
    }

    fn watcher_cursor_from_current(&self) -> WatcherCursor {
        Self::watcher_cursor_from_current(self)
    }

    fn has_pending(&self, cursor: WatcherCursor) -> bool {
        Self::has_pending(self, cursor)
    }

    fn has_pending_visible<P>(&self, cursor: WatcherCursor, visibility: &P) -> bool
    where
        P: VisibilityPolicy<V>,
    {
        Self::has_pending_visible(self, cursor, visibility)
    }

    fn poll(&self, cursor: &mut WatcherCursor) -> Vec<StoreChange<S, C, R, I, V, E>> {
        Self::poll(self, cursor)
    }

    fn poll_visible<P>(
        &self,
        cursor: &mut WatcherCursor,
        visibility: &P,
    ) -> Vec<StoreChange<S, C, R, I, V, E>>
    where
        P: VisibilityPolicy<V>,
    {
        Self::poll_visible(self, cursor, visibility)
    }
}

#[cfg(test)]
mod tests {
    use super::{PresenceIngress, PresenceServer, PresenceWatch};
    use ruthere_core::{
        Activity, Availability, Expiry, PresenceAddress, PresenceUpdate, Timestamp, Visibility,
    };

    fn receive_through_ingress<S>(
        server: &mut S,
        update: PresenceUpdate<u64, u64, u64, u64, &'static str>,
    ) -> u64
    where
        S: PresenceIngress<u64, u64, u64, u64, &'static str>,
    {
        server.receive(update)
    }

    fn poll_visible_through_watch<S, P>(
        server: &S,
        cursor: &mut super::WatcherCursor,
        visibility: &P,
    ) -> alloc::vec::Vec<super::StoreChange<u64, u64, u64, u64, &'static str>>
    where
        S: PresenceWatch<u64, u64, u64, u64, &'static str>,
        P: super::VisibilityPolicy<&'static str>,
    {
        server.poll_visible(cursor, visibility)
    }

    #[test]
    fn server_receives_updates_and_expires_entries() {
        let mut server = PresenceServer::<u64, u64, u64, u64, &'static str>::new();

        let sequence = receive_through_ingress(
            &mut server,
            PresenceUpdate::new(
                PresenceAddress::new(7, 9, Some(3)),
                11,
                Visibility::Restricted("members"),
                Timestamp::new(100),
                Expiry::At(Timestamp::new(160)),
            )
            .set_availability(Availability::Available),
        );

        assert_eq!(sequence, 1);
        assert_eq!(server.len(), 1);
        assert_eq!(server.last_sequence(), 1);

        let removed = server.expire(Timestamp::new(160));

        assert_eq!(removed, 1);
        assert!(server.is_empty());
        assert_eq!(server.last_sequence(), 2);
    }

    #[test]
    fn server_polls_visible_changes_through_watchers() {
        let mut server = PresenceServer::<u64, u64, u64, u64, &'static str>::new();
        let member_view = |visibility: &Visibility<&'static str>| {
            matches!(
                visibility,
                Visibility::Public | Visibility::Restricted("members")
            )
        };
        let public_only =
            |visibility: &Visibility<&'static str>| matches!(visibility, Visibility::Public);

        receive_through_ingress(
            &mut server,
            PresenceUpdate::new(
                PresenceAddress::new(7, 9, Some(3)),
                11,
                Visibility::Restricted("members"),
                Timestamp::new(100),
                Expiry::At(Timestamp::new(160)),
            )
            .set_activity(Activity::Editing),
        );
        receive_through_ingress(
            &mut server,
            PresenceUpdate::new(
                PresenceAddress::new(8, 9, Some(4)),
                12,
                Visibility::Public,
                Timestamp::new(101),
                Expiry::Never,
            )
            .set_availability(Availability::Available),
        );

        let mut member_cursor =
            <PresenceServer<u64, u64, u64, u64, &'static str> as PresenceWatch<
                u64,
                u64,
                u64,
                u64,
                &'static str,
            >>::watcher_cursor(&server);
        let mut public_cursor = server.watcher_cursor();

        assert!(
            <PresenceServer<u64, u64, u64, u64, &'static str> as PresenceWatch<
                u64,
                u64,
                u64,
                u64,
                &'static str,
            >>::has_pending_visible(&server, member_cursor, &member_view)
        );
        assert!(server.has_pending_visible(public_cursor, &public_only));

        let member_changes = poll_visible_through_watch(&server, &mut member_cursor, &member_view);
        let public_changes = server.poll_visible(&mut public_cursor, &public_only);

        assert_eq!(member_changes.len(), 2);
        assert_eq!(public_changes.len(), 1);
        assert_eq!(member_cursor.sequence(), 2);
        assert_eq!(public_cursor.sequence(), 2);
        assert!(!server.has_pending_visible(member_cursor, &member_view));
        assert!(!server.has_pending_visible(public_cursor, &public_only));
    }

    #[test]
    fn watcher_cursor_from_current_starts_at_tail() {
        let mut server = PresenceServer::<u64, u64, u64, u64, ()>::new();

        server.receive(
            PresenceUpdate::new(
                PresenceAddress::new(7, 9, Some(3)),
                11,
                Visibility::Public,
                Timestamp::new(100),
                Expiry::Never,
            )
            .set_activity(Activity::Observing),
        );

        let mut cursor = server.watcher_cursor_from_current();
        let changes = server.poll(&mut cursor);

        assert!(changes.is_empty());
        assert_eq!(cursor.sequence(), 1);
    }
}
