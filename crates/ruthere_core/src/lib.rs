// Copyright 2026 the ruthere Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![no_std]

//! Semantic core types for scoped, time-sensitive presence facts.
//!
//! `ruthere_core` owns the vocabulary for publishing presence updates and for
//! materializing current presence state. It intentionally does not own watcher
//! identity, store sequencing, subscription delivery, or visibility evaluation.

extern crate alloc;

use alloc::vec::Vec;
use core::fmt::Debug;
use core::hash::Hash;

/// Key types accepted by the core presence model.
///
/// The trait is intentionally small so applications can use their own subject,
/// context, resource, origin, and visibility label types without adapters.
pub trait PresenceKey: Clone + Debug + Eq + Hash {}

impl<T> PresenceKey for T where T: Clone + Debug + Eq + Hash {}

/// A zero-variant helper used when a caller does not need an extension type.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Never {}

/// Application-defined extension facets stored alongside built-in presence
/// facets.
pub trait ExtensionFacet: Clone {
    /// A stable kind identifier used to clear a previously asserted facet.
    type Kind: PresenceKey;

    /// Returns the kind of this extension facet.
    fn kind(&self) -> Self::Kind;
}

impl ExtensionFacet for Never {
    type Kind = Self;

    fn kind(&self) -> Self::Kind {
        match *self {}
    }
}

/// A monotonic or wall-clock timestamp supplied by the embedding application.
///
/// The core only requires timestamps to be comparable. Higher-level crates can
/// decide whether the value represents unix milliseconds, logical time, or
/// another domain-specific clock.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Timestamp(u64);

impl Timestamp {
    /// Creates a timestamp from its raw integer representation.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw integer representation of the timestamp.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Expiry attached to presence updates and snapshots.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Expiry {
    /// The asserted presence does not expire automatically.
    Never,
    /// The asserted presence expires once the given timestamp is reached.
    At(Timestamp),
}

impl Expiry {
    /// Returns `true` when this expiry has elapsed by `now`.
    #[must_use]
    pub fn is_expired_by(self, now: Timestamp) -> bool {
        match self {
            Self::Never => false,
            Self::At(deadline) => deadline <= now,
        }
    }
}

/// A visibility label carried by presence facts.
///
/// The core does not interpret the `Restricted` payload. Stores or outer policy
/// layers decide whether a particular observer can see a labeled fact.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Visibility<V> {
    /// Visible without further policy checks.
    #[default]
    Public,
    /// Visible only under an embedding-defined visibility label.
    Restricted(V),
    /// Not intended for observation outside the asserting origin.
    Private,
}

/// Built-in availability state for a subject.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Availability {
    /// Explicitly available.
    Available,
    /// Present but not interruptible.
    Busy,
    /// Temporarily away.
    Away,
    /// Explicitly offline.
    Offline,
    /// Presence is known but availability is not.
    Unknown,
}

/// Built-in coarse-grained activity state for a subject.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Activity {
    /// No active work is currently known.
    Idle,
    /// Observing or browsing without active modification.
    Observing,
    /// Moving through some application-defined space.
    Navigating,
    /// Actively editing or producing work.
    Editing,
    /// Presenting or sharing activity outward.
    Presenting,
    /// Performing automated or delegated work.
    Acting,
    /// Application-specific activity code.
    Custom(u16),
}

/// The kind of a built-in facet.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BuiltinFacetKind {
    /// The facet stores [`Availability`].
    Availability,
    /// The facet stores [`Activity`].
    Activity,
    /// The facet stores a last-seen [`Timestamp`].
    LastSeen,
}

/// Closed built-in presence facets supported by the core.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BuiltinFacet {
    /// Availability state.
    Availability(Availability),
    /// Activity state.
    Activity(Activity),
    /// Last observed activity time.
    LastSeen(Timestamp),
}

impl BuiltinFacet {
    /// Returns the kind of this built-in facet.
    #[must_use]
    pub const fn kind(self) -> BuiltinFacetKind {
        match self {
            Self::Availability(..) => BuiltinFacetKind::Availability,
            Self::Activity(..) => BuiltinFacetKind::Activity,
            Self::LastSeen(..) => BuiltinFacetKind::LastSeen,
        }
    }
}

/// The kind of a presence facet, including application-defined extensions.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PresenceFacetKind<K = Never> {
    /// A built-in facet kind.
    Builtin(BuiltinFacetKind),
    /// An application-defined facet kind.
    Extension(K),
}

impl<K> PresenceFacetKind<K> {
    /// Returns the built-in kind when this is a built-in facet.
    #[must_use]
    pub fn builtin(self) -> Option<BuiltinFacetKind> {
        match self {
            Self::Builtin(kind) => Some(kind),
            Self::Extension(..) => None,
        }
    }
}

/// A single presence facet, either built in or application defined.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PresenceFacet<E = Never> {
    /// A built-in facet.
    Builtin(BuiltinFacet),
    /// An application-defined facet.
    Extension(E),
}

impl<E> PresenceFacet<E>
where
    E: ExtensionFacet,
{
    /// Returns the kind of this facet.
    #[must_use]
    pub fn kind(&self) -> PresenceFacetKind<E::Kind> {
        match self {
            Self::Builtin(facet) => PresenceFacetKind::Builtin(facet.kind()),
            Self::Extension(facet) => PresenceFacetKind::Extension(facet.kind()),
        }
    }
}

/// A change applied within a presence update.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FacetChange<E = Never>
where
    E: ExtensionFacet,
{
    /// Asserts or replaces a facet for the current update scope.
    Set(PresenceFacet<E>),
    /// Clears a previously asserted facet of the given kind.
    Clear(PresenceFacetKind<E::Kind>),
}

/// The addressed scope of a presence update or snapshot.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PresenceAddress<S, C, R> {
    /// The subject whose presence is described.
    pub subject: S,
    /// The context where that presence is meaningful.
    pub context: C,
    /// The optional resource within the subject that asserted the presence.
    pub resource: Option<R>,
}

impl<S, C, R> PresenceAddress<S, C, R> {
    /// Creates a new addressed presence scope.
    #[must_use]
    pub const fn new(subject: S, context: C, resource: Option<R>) -> Self {
        Self {
            subject,
            context,
            resource,
        }
    }
}

/// A published batch of facet changes from a single origin and scope.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PresenceUpdate<S, C, R, I, V, E = Never>
where
    E: ExtensionFacet,
{
    /// The scope of the asserted presence.
    pub address: PresenceAddress<S, C, R>,
    /// The origin that produced this update.
    pub origin: I,
    /// The visibility label carried by the update.
    pub visibility: Visibility<V>,
    /// When the origin observed or asserted the update.
    pub observed_at: Timestamp,
    /// When the update should expire.
    pub expiry: Expiry,
    /// The facet changes carried by this update.
    pub changes: Vec<FacetChange<E>>,
}

impl<S, C, R, I, V, E> PresenceUpdate<S, C, R, I, V, E>
where
    E: ExtensionFacet,
{
    /// Creates an empty presence update for the given scope and metadata.
    #[must_use]
    pub fn new(
        address: PresenceAddress<S, C, R>,
        origin: I,
        visibility: Visibility<V>,
        observed_at: Timestamp,
        expiry: Expiry,
    ) -> Self {
        Self {
            address,
            origin,
            visibility,
            observed_at,
            expiry,
            changes: Vec::new(),
        }
    }

    /// Appends a facet change and returns the updated value.
    #[must_use]
    pub fn with_change(mut self, change: FacetChange<E>) -> Self {
        self.changes.push(change);
        self
    }
}

/// A materialized current view of presence for one scope and origin.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PresenceSnapshot<S, C, R, I, V, E = Never>
where
    E: ExtensionFacet,
{
    /// The scope of the materialized presence.
    pub address: PresenceAddress<S, C, R>,
    /// The origin that contributed this snapshot.
    pub origin: I,
    /// The carried visibility label.
    pub visibility: Visibility<V>,
    /// The most recent observation time represented by the snapshot.
    pub observed_at: Timestamp,
    /// The expiry attached to the snapshot.
    pub expiry: Expiry,
    /// The current materialized facets.
    pub facets: Vec<PresenceFacet<E>>,
}

impl<S, C, R, I, V, E> PresenceSnapshot<S, C, R, I, V, E>
where
    E: ExtensionFacet,
{
    /// Creates an empty snapshot for the given scope and metadata.
    #[must_use]
    pub fn new(
        address: PresenceAddress<S, C, R>,
        origin: I,
        visibility: Visibility<V>,
        observed_at: Timestamp,
        expiry: Expiry,
    ) -> Self {
        Self {
            address,
            origin,
            visibility,
            observed_at,
            expiry,
            facets: Vec::new(),
        }
    }

    /// Appends a facet and returns the updated value.
    #[must_use]
    pub fn with_facet(mut self, facet: PresenceFacet<E>) -> Self {
        self.facets.push(facet);
        self
    }

    /// Returns the materialized built-in availability, if present.
    #[must_use]
    pub fn availability(&self) -> Option<Availability> {
        self.facets.iter().find_map(|facet| match facet {
            PresenceFacet::Builtin(BuiltinFacet::Availability(value)) => Some(*value),
            PresenceFacet::Builtin(..) | PresenceFacet::Extension(..) => None,
        })
    }

    /// Returns the materialized built-in activity, if present.
    #[must_use]
    pub fn activity(&self) -> Option<Activity> {
        self.facets.iter().find_map(|facet| match facet {
            PresenceFacet::Builtin(BuiltinFacet::Activity(value)) => Some(*value),
            PresenceFacet::Builtin(..) | PresenceFacet::Extension(..) => None,
        })
    }

    /// Returns the materialized built-in last-seen timestamp, if present.
    #[must_use]
    pub fn last_seen(&self) -> Option<Timestamp> {
        self.facets.iter().find_map(|facet| match facet {
            PresenceFacet::Builtin(BuiltinFacet::LastSeen(value)) => Some(*value),
            PresenceFacet::Builtin(..) | PresenceFacet::Extension(..) => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Activity, Availability, BuiltinFacet, BuiltinFacetKind, Expiry, ExtensionFacet,
        FacetChange, Never, PresenceAddress, PresenceFacet, PresenceFacetKind, PresenceSnapshot,
        PresenceUpdate, Timestamp, Visibility,
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
    fn builtin_facet_reports_its_kind() {
        assert_eq!(
            BuiltinFacet::Availability(Availability::Busy).kind(),
            BuiltinFacetKind::Availability
        );
        assert_eq!(
            BuiltinFacet::Activity(Activity::Editing).kind(),
            BuiltinFacetKind::Activity
        );
        assert_eq!(
            BuiltinFacet::LastSeen(Timestamp::new(9)).kind(),
            BuiltinFacetKind::LastSeen
        );
    }

    #[test]
    fn extension_facet_reports_its_kind() {
        let facet = PresenceFacet::Extension(DemoFacet::Focus(42));

        assert_eq!(
            facet.kind(),
            PresenceFacetKind::Extension(DemoFacetKind::Focus)
        );
    }

    #[test]
    fn expiry_reports_when_deadline_has_elapsed() {
        assert!(!Expiry::Never.is_expired_by(Timestamp::new(10)));
        assert!(!Expiry::At(Timestamp::new(11)).is_expired_by(Timestamp::new(10)));
        assert!(Expiry::At(Timestamp::new(10)).is_expired_by(Timestamp::new(10)));
        assert!(Expiry::At(Timestamp::new(9)).is_expired_by(Timestamp::new(10)));
    }

    #[test]
    fn snapshot_accessors_find_materialized_builtins() {
        let snapshot = PresenceSnapshot::new(
            PresenceAddress::new(1_u64, 7_u64, Some(3_u64)),
            99_u64,
            Visibility::<()>::Public,
            Timestamp::new(12),
            Expiry::At(Timestamp::new(20)),
        )
        .with_facet(PresenceFacet::Builtin(BuiltinFacet::Availability(
            Availability::Available,
        )))
        .with_facet(PresenceFacet::Builtin(BuiltinFacet::Activity(
            Activity::Observing,
        )))
        .with_facet(PresenceFacet::Builtin(BuiltinFacet::LastSeen(
            Timestamp::new(11),
        )))
        .with_facet(PresenceFacet::Extension(DemoFacet::Focus(5)));

        assert_eq!(snapshot.availability(), Some(Availability::Available));
        assert_eq!(snapshot.activity(), Some(Activity::Observing));
        assert_eq!(snapshot.last_seen(), Some(Timestamp::new(11)));
    }

    #[test]
    fn update_builder_preserves_scope_and_changes() {
        let update: PresenceUpdate<u64, u64, u64, u64, &'static str, Never> = PresenceUpdate::new(
            PresenceAddress::new(10_u64, 20_u64, None::<u64>),
            30_u64,
            Visibility::Restricted("team-a"),
            Timestamp::new(40),
            Expiry::Never,
        )
        .with_change(FacetChange::Set(PresenceFacet::Builtin(
            BuiltinFacet::Availability(Availability::Away),
        )))
        .with_change(FacetChange::Clear(PresenceFacetKind::Builtin(
            BuiltinFacetKind::LastSeen,
        )));

        assert_eq!(update.address.subject, 10);
        assert_eq!(update.address.context, 20);
        assert_eq!(update.address.resource, None);
        assert_eq!(update.observed_at, Timestamp::new(40));
        assert_eq!(update.changes.len(), 2);
    }
}
