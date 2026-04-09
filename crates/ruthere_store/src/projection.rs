// Copyright 2026 the ruthere Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use alloc::vec::Vec;
use core::cmp::Ordering;

use hashbrown::{HashMap, hash_map::Entry};
use ruthere_core::{
    Activity, Availability, ExtensionFacet, PresenceKey, PresenceSnapshot, Timestamp,
};

/// Policy used to project raw resource snapshots into a subject-level summary.
pub trait SubjectProjectionPolicy {
    /// Returns a relative dominance rank for an activity value.
    #[must_use]
    fn activity_rank(&self, value: Option<Activity>) -> u8;

    /// Returns a relative dominance rank for an availability value.
    #[must_use]
    fn availability_rank(&self, value: Option<Availability>) -> u8;
}

/// Default subject-level projection policy for common presence UIs.
///
/// The default policy prefers active work over passive observation, and prefers
/// more specific availability over weaker signals.
#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultSubjectProjectionPolicy;

impl SubjectProjectionPolicy for DefaultSubjectProjectionPolicy {
    fn activity_rank(&self, value: Option<Activity>) -> u8 {
        match value {
            Some(Activity::Editing) => 7,
            Some(Activity::Presenting) => 6,
            Some(Activity::Acting) => 5,
            Some(Activity::Navigating) => 4,
            Some(Activity::Observing) => 3,
            Some(Activity::Idle) => 2,
            Some(Activity::Custom(..)) => 1,
            None => 0,
        }
    }

    fn availability_rank(&self, value: Option<Availability>) -> u8 {
        match value {
            Some(Availability::Busy) => 5,
            Some(Availability::Available) => 4,
            Some(Availability::Away) => 3,
            Some(Availability::Unknown) => 2,
            Some(Availability::Offline) => 1,
            None => 0,
        }
    }
}

/// A projected subject-level view for one context.
///
/// This keeps the calm headline fields most callers want while preserving the
/// underlying resource snapshots for detailed inspection or UI drill-down.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubjectPresenceSummary<S, C, R, I, V, E>
where
    E: ExtensionFacet,
{
    /// The subject represented by the summary.
    pub subject: S,
    /// The context where the summary is meaningful.
    pub context: C,
    /// The resource that dominates the summary headline under the chosen policy.
    pub dominant_resource: Option<R>,
    /// The origin that dominates the summary headline under the chosen policy.
    pub dominant_origin: I,
    /// The most recent observation time across all contributing resources.
    pub observed_at: Timestamp,
    /// The projected availability.
    pub availability: Option<Availability>,
    /// The projected activity.
    pub activity: Option<Activity>,
    /// The most recent explicit last-seen timestamp across all contributing
    /// resources.
    pub last_seen: Option<Timestamp>,
    /// The underlying resource snapshots that contributed to the summary.
    pub resources: Vec<PresenceSnapshot<S, C, R, I, V, E>>,
}

#[derive(Clone, Debug)]
pub(crate) struct SubjectProjectionAccumulator<S, C, R, I, V, E>
where
    E: ExtensionFacet,
{
    pub(crate) subject: S,
    pub(crate) context: C,
    pub(crate) resources: Vec<PresenceSnapshot<S, C, R, I, V, E>>,
}

impl<S, C, R, I, V, E> SubjectProjectionAccumulator<S, C, R, I, V, E>
where
    S: PresenceKey,
    C: PresenceKey,
    E: ExtensionFacet,
{
    pub(crate) fn new(snapshot: PresenceSnapshot<S, C, R, I, V, E>) -> Self {
        Self {
            subject: snapshot.address.subject.clone(),
            context: snapshot.address.context.clone(),
            resources: Vec::from([snapshot]),
        }
    }

    pub(crate) fn push(&mut self, snapshot: PresenceSnapshot<S, C, R, I, V, E>) {
        self.resources.push(snapshot);
    }
}

pub(crate) fn group_snapshots_by_subject<S, C, R, I, V, E>(
    snapshots: Vec<PresenceSnapshot<S, C, R, I, V, E>>,
) -> Vec<SubjectProjectionAccumulator<S, C, R, I, V, E>>
where
    S: PresenceKey,
    C: PresenceKey,
    E: ExtensionFacet,
{
    let mut groups: HashMap<S, SubjectProjectionAccumulator<S, C, R, I, V, E>> = HashMap::new();

    for snapshot in snapshots {
        let subject = snapshot.address.subject.clone();
        match groups.entry(subject) {
            Entry::Occupied(entry) => entry.into_mut().push(snapshot),
            Entry::Vacant(entry) => {
                entry.insert(SubjectProjectionAccumulator::new(snapshot));
            }
        }
    }

    groups.into_values().collect()
}

pub(crate) fn summarize_subject<S, C, R, I, V, E, P>(
    accumulator: SubjectProjectionAccumulator<S, C, R, I, V, E>,
    policy: &P,
) -> SubjectPresenceSummary<S, C, R, I, V, E>
where
    S: PresenceKey,
    C: PresenceKey,
    R: PresenceKey,
    I: PresenceKey,
    E: ExtensionFacet,
    P: SubjectProjectionPolicy,
{
    let dominant = accumulator
        .resources
        .iter()
        .max_by(|left, right| compare_snapshot_dominance(policy, left, right))
        .expect("subject projection requires at least one snapshot");
    let observed_at = accumulator
        .resources
        .iter()
        .map(|snapshot| snapshot.observed_at)
        .max()
        .expect("subject projection requires at least one snapshot");

    let last_seen = accumulator
        .resources
        .iter()
        .filter_map(PresenceSnapshot::last_seen)
        .max();

    SubjectPresenceSummary {
        subject: accumulator.subject,
        context: accumulator.context,
        dominant_resource: dominant.address.resource.clone(),
        dominant_origin: dominant.origin.clone(),
        observed_at,
        availability: dominant.availability().or_else(|| {
            accumulator
                .resources
                .iter()
                .max_by(|left, right| compare_availability(policy, left, right))
                .and_then(PresenceSnapshot::availability)
        }),
        activity: dominant.activity().or_else(|| {
            accumulator
                .resources
                .iter()
                .max_by(|left, right| compare_activity(policy, left, right))
                .and_then(PresenceSnapshot::activity)
        }),
        last_seen,
        resources: accumulator.resources,
    }
}

fn compare_snapshot_dominance<S, C, R, I, V, E, P>(
    policy: &P,
    left: &PresenceSnapshot<S, C, R, I, V, E>,
    right: &PresenceSnapshot<S, C, R, I, V, E>,
) -> Ordering
where
    E: ExtensionFacet,
    P: SubjectProjectionPolicy,
{
    compare_activity(policy, left, right)
        .then_with(|| compare_availability(policy, left, right))
        .then_with(|| left.observed_at.cmp(&right.observed_at))
        .then_with(|| left.last_seen().cmp(&right.last_seen()))
}

fn compare_activity<S, C, R, I, V, E, P>(
    policy: &P,
    left: &PresenceSnapshot<S, C, R, I, V, E>,
    right: &PresenceSnapshot<S, C, R, I, V, E>,
) -> Ordering
where
    E: ExtensionFacet,
    P: SubjectProjectionPolicy,
{
    policy
        .activity_rank(left.activity())
        .cmp(&policy.activity_rank(right.activity()))
        .then_with(|| left.observed_at.cmp(&right.observed_at))
}

fn compare_availability<S, C, R, I, V, E, P>(
    policy: &P,
    left: &PresenceSnapshot<S, C, R, I, V, E>,
    right: &PresenceSnapshot<S, C, R, I, V, E>,
) -> Ordering
where
    E: ExtensionFacet,
    P: SubjectProjectionPolicy,
{
    policy
        .availability_rank(left.availability())
        .cmp(&policy.availability_rank(right.availability()))
        .then_with(|| left.observed_at.cmp(&right.observed_at))
}
