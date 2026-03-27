// Copyright 2026 the ruthere Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![no_std]

//! Write-side publication helpers for `ruthere`.
//!
//! `ruthere_beacon` owns publication ergonomics for one addressed presence
//! source. It intentionally does not own storage, transport, background
//! scheduling, or wall-clock access.

use core::marker::PhantomData;

use ruthere_core::{ExtensionFacet, Never, PresenceAddress, PresenceUpdate, Timestamp, Visibility};

/// Expiry policy resolved against each observation timestamp.
///
/// `After` uses the same raw units as [`Timestamp`]. Callers decide whether
/// those units mean wall-clock milliseconds, logical ticks, or something else.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ExpiryPolicy {
    /// Never expire published updates automatically.
    Never,
    /// Expire a fixed number of timestamp units after observation.
    After(u64),
}

impl ExpiryPolicy {
    /// Resolves the concrete expiry attached to an observation at `observed_at`.
    #[must_use]
    pub const fn resolve(self, observed_at: Timestamp) -> ruthere_core::Expiry {
        match self {
            Self::Never => ruthere_core::Expiry::Never,
            Self::After(delta) => {
                ruthere_core::Expiry::At(Timestamp::new(observed_at.get().saturating_add(delta)))
            }
        }
    }
}

/// A configured write-side presence publisher for one addressed source.
///
/// A beacon captures stable publication metadata, then builds
/// [`PresenceUpdate`] values on demand for the caller to publish through a
/// store, transport, or another runtime layer.
///
/// # Example
///
/// ```
/// use ruthere_beacon::{ExpiryPolicy, PresenceBeacon};
/// use ruthere_core::{
///     Activity, Availability, PresenceAddress, Timestamp, Visibility,
/// };
///
/// let beacon = PresenceBeacon::new(
///     PresenceAddress::new("alice", "doc-42", Some("browser-tab")),
///     "session/browser",
/// )
/// .with_visibility(Visibility::Restricted("doc-members"))
/// .with_expiry_policy(ExpiryPolicy::After(60));
///
/// let update = beacon
///     .heartbeat_at(Timestamp::new(100))
///     .set_availability(Availability::Available)
///     .set_activity(Activity::Editing);
///
/// assert_eq!(update.observed_at, Timestamp::new(100));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresenceBeacon<S, C, R, I, V, E = Never>
where
    E: ExtensionFacet,
{
    address: PresenceAddress<S, C, R>,
    origin: I,
    visibility: Visibility<V>,
    expiry_policy: ExpiryPolicy,
    extension: PhantomData<fn() -> E>,
}

impl<S, C, R, I, V> PresenceBeacon<S, C, R, I, V, Never> {
    /// Creates a beacon for one addressed source without extension facets.
    ///
    /// New beacons default to public visibility and never-expiring updates.
    #[must_use]
    pub fn new(address: PresenceAddress<S, C, R>, origin: I) -> Self {
        Self {
            address,
            origin,
            visibility: Visibility::Public,
            expiry_policy: ExpiryPolicy::Never,
            extension: PhantomData,
        }
    }
}

impl<S, C, R, I, V, E> PresenceBeacon<S, C, R, I, V, E>
where
    E: ExtensionFacet,
{
    /// Creates a beacon for one addressed source with an explicit extension
    /// facet type.
    ///
    /// New beacons default to public visibility and never-expiring updates.
    #[must_use]
    pub fn new_typed(address: PresenceAddress<S, C, R>, origin: I) -> Self {
        Self {
            address,
            origin,
            visibility: Visibility::Public,
            expiry_policy: ExpiryPolicy::Never,
            extension: PhantomData,
        }
    }

    /// Returns the configured address.
    #[must_use]
    pub const fn address(&self) -> &PresenceAddress<S, C, R> {
        &self.address
    }

    /// Returns the configured origin.
    #[must_use]
    pub const fn origin(&self) -> &I {
        &self.origin
    }

    /// Returns the configured visibility.
    #[must_use]
    pub const fn visibility(&self) -> &Visibility<V> {
        &self.visibility
    }

    /// Returns the configured expiry policy.
    #[must_use]
    pub const fn expiry_policy(&self) -> ExpiryPolicy {
        self.expiry_policy
    }

    /// Replaces the configured visibility.
    #[must_use]
    pub fn with_visibility(mut self, visibility: Visibility<V>) -> Self {
        self.visibility = visibility;
        self
    }

    /// Replaces the configured expiry policy.
    #[must_use]
    pub fn with_expiry_policy(mut self, expiry_policy: ExpiryPolicy) -> Self {
        self.expiry_policy = expiry_policy;
        self
    }

    /// Builds an empty update at the provided observation timestamp.
    #[must_use]
    pub fn update_at(&self, observed_at: Timestamp) -> PresenceUpdate<S, C, R, I, V, E>
    where
        S: Clone,
        C: Clone,
        R: Clone,
        I: Clone,
        V: Clone,
    {
        PresenceUpdate::new(
            self.address.clone(),
            self.origin.clone(),
            self.visibility.clone(),
            observed_at,
            self.expiry_policy.resolve(observed_at),
        )
    }

    /// Builds an update that refreshes last-seen at the provided observation
    /// timestamp.
    #[must_use]
    pub fn heartbeat_at(&self, observed_at: Timestamp) -> PresenceUpdate<S, C, R, I, V, E>
    where
        S: Clone,
        C: Clone,
        R: Clone,
        I: Clone,
        V: Clone,
    {
        self.update_at(observed_at).set_last_seen(observed_at)
    }
}

#[cfg(test)]
mod tests {
    use super::{ExpiryPolicy, PresenceBeacon};
    use ruthere_core::{
        Activity, Availability, BuiltinFacet, Expiry, ExtensionFacet, FacetChange, PresenceAddress,
        PresenceFacet, PresenceFacetKind, Timestamp, Visibility,
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
    fn expiry_policy_resolves_from_observed_at() {
        assert_eq!(
            ExpiryPolicy::Never.resolve(Timestamp::new(10)),
            Expiry::Never
        );
        assert_eq!(
            ExpiryPolicy::After(25).resolve(Timestamp::new(10)),
            Expiry::At(Timestamp::new(35))
        );
    }

    #[test]
    fn beacon_defaults_to_public_never_expiring_updates() {
        let beacon = PresenceBeacon::<u64, u64, u64, u64, &'static str>::new(
            PresenceAddress::new(7, 9, Some(3)),
            11,
        );

        let update = beacon.update_at(Timestamp::new(100));

        assert_eq!(update.address.subject, 7);
        assert_eq!(update.address.context, 9);
        assert_eq!(update.address.resource, Some(3));
        assert_eq!(update.origin, 11);
        assert_eq!(update.visibility, Visibility::Public);
        assert_eq!(update.expiry, Expiry::Never);
        assert!(update.changes.is_empty());
    }

    #[test]
    fn beacon_heartbeat_uses_configured_visibility_and_policy() {
        let beacon = PresenceBeacon::<u64, u64, u64, u64, &'static str>::new(
            PresenceAddress::new(7, 9, Some(3)),
            11,
        )
        .with_visibility(Visibility::Restricted("members"))
        .with_expiry_policy(ExpiryPolicy::After(60));

        let update = beacon.heartbeat_at(Timestamp::new(100));

        assert_eq!(update.visibility, Visibility::Restricted("members"));
        assert_eq!(update.expiry, Expiry::At(Timestamp::new(160)));
        assert!(update.changes.iter().any(|change| matches!(
            change,
            FacetChange::Set(PresenceFacet::Builtin(BuiltinFacet::LastSeen(value)))
                if *value == Timestamp::new(100)
        )));
    }

    #[test]
    fn beacon_updates_compose_with_core_update_helpers() {
        let beacon = PresenceBeacon::<u64, u64, u64, u64, (), DemoFacet>::new_typed(
            PresenceAddress::new(7, 9, Some(3)),
            11,
        )
        .with_expiry_policy(ExpiryPolicy::After(60));

        let update = beacon
            .heartbeat_at(Timestamp::new(100))
            .set_availability(Availability::Available)
            .set_activity(Activity::Editing)
            .set_extension(DemoFacet::Focus(42));

        assert!(update.changes.iter().any(|change| matches!(
            change,
            FacetChange::Set(PresenceFacet::Builtin(BuiltinFacet::Availability(
                Availability::Available
            )))
        )));
        assert!(update.changes.iter().any(|change| matches!(
            change,
            FacetChange::Set(PresenceFacet::Builtin(BuiltinFacet::Activity(
                Activity::Editing
            )))
        )));
        assert!(update.changes.iter().any(|change| matches!(
            change,
            FacetChange::Set(PresenceFacet::Extension(DemoFacet::Focus(42)))
        )));
        assert!(update.changes.iter().any(|change| matches!(
            change,
            FacetChange::Set(PresenceFacet::Builtin(BuiltinFacet::LastSeen(value)))
                if *value == Timestamp::new(100)
        )));
        assert_eq!(DemoFacet::Focus(42).kind(), DemoFacetKind::Focus);
        let _ = PresenceFacetKind::Extension(DemoFacetKind::Focus);
    }
}
