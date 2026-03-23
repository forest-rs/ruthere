// Copyright 2026 the ruthere Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Runnable end-to-end example for the basic `ruthere` presence flow.

use ruthere_core::{
    Activity, Availability, Expiry, PresenceAddress, PresenceUpdate, Timestamp, Visibility,
};
use ruthere_store::{InMemoryStore, PresenceEntryKey, SubjectPresenceSummary};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct SubjectId(&'static str);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ContextId(&'static str);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ResourceId(&'static str);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct OriginId(&'static str);

type Store = InMemoryStore<SubjectId, ContextId, ResourceId, OriginId, &'static str>;
type Snapshot =
    ruthere_core::PresenceSnapshot<SubjectId, ContextId, ResourceId, OriginId, &'static str>;
type Summary = SubjectPresenceSummary<
    SubjectId,
    ContextId,
    ResourceId,
    OriginId,
    &'static str,
    ruthere_core::Never,
>;

fn main() {
    let mut store = Store::new();

    let doc = ContextId("doc-42");
    let alice_browser = PresenceAddress::new(
        SubjectId("alice"),
        doc.clone(),
        Some(ResourceId("browser-tab")),
    );
    let alice_mobile =
        PresenceAddress::new(SubjectId("alice"), doc.clone(), Some(ResourceId("mobile")));

    let browser_origin = OriginId("session/browser");
    let mobile_origin = OriginId("session/mobile");

    let first_sequence = store.publish(
        PresenceUpdate::new(
            alice_browser.clone(),
            browser_origin.clone(),
            Visibility::Restricted("doc-members"),
            Timestamp::new(100),
            Expiry::At(Timestamp::new(160)),
        )
        .set_availability(Availability::Available)
        .set_activity(Activity::Observing)
        .set_last_seen(Timestamp::new(100)),
    );

    let second_sequence = store.publish(
        PresenceUpdate::new(
            alice_browser.clone(),
            browser_origin.clone(),
            Visibility::Restricted("doc-members"),
            Timestamp::new(110),
            Expiry::At(Timestamp::new(170)),
        )
        .set_activity(Activity::Editing)
        .set_last_seen(Timestamp::new(110)),
    );

    let third_sequence = store.publish(
        PresenceUpdate::new(
            alice_mobile.clone(),
            mobile_origin.clone(),
            Visibility::Restricted("doc-members"),
            Timestamp::new(105),
            Expiry::At(Timestamp::new(120)),
        )
        .set_availability(Availability::Away)
        .set_activity(Activity::Observing),
    );

    println!("Published sequences: {first_sequence}, {second_sequence}, {third_sequence}");
    println!("Store sequence after publishes: {}", store.last_sequence());

    let browser_key = PresenceEntryKey::new(alice_browser.clone(), browser_origin.clone());
    let browser_snapshot = store
        .snapshot(&browser_key)
        .expect("browser entry should be present after publish");

    println!();
    println!("Single snapshot lookup:");
    print_snapshot(&browser_snapshot);

    println!();
    println!("All snapshots in doc-42 before expiry:");
    let mut snapshots = store.snapshots_in_context(&doc);
    snapshots.sort_by(snapshot_sort_key);
    for snapshot in &snapshots {
        print_snapshot(snapshot);
    }

    println!();
    println!("Projected subject summaries in doc-42 before expiry:");
    let mut summaries = store.subject_summaries_in_context(&doc);
    summaries.sort_by(summary_sort_key);
    for summary in &summaries {
        print_summary(summary);
    }

    let removed = store.expire(Timestamp::new(125));
    println!();
    println!("Expired entries at t=125: {removed}");

    println!();
    println!("All snapshots in doc-42 after expiry:");
    let mut snapshots = store.snapshots_in_context(&doc);
    snapshots.sort_by(snapshot_sort_key);
    for snapshot in &snapshots {
        print_snapshot(snapshot);
    }

    println!();
    println!("Projected subject summaries in doc-42 after expiry:");
    let mut summaries = store.subject_summaries_in_context(&doc);
    summaries.sort_by(summary_sort_key);
    for summary in &summaries {
        print_summary(summary);
    }
}

fn snapshot_sort_key(left: &Snapshot, right: &Snapshot) -> core::cmp::Ordering {
    left.address
        .subject
        .cmp(&right.address.subject)
        .then_with(|| left.address.resource.cmp(&right.address.resource))
        .then_with(|| left.origin.cmp(&right.origin))
}

fn summary_sort_key(left: &Summary, right: &Summary) -> core::cmp::Ordering {
    left.subject.cmp(&right.subject)
}

fn print_snapshot(snapshot: &Snapshot) {
    let subject = snapshot.address.subject.0;
    let context = snapshot.address.context.0;
    let resource = snapshot
        .address
        .resource
        .as_ref()
        .map_or("none", |resource| resource.0);
    let origin = snapshot.origin.0;
    let availability = snapshot.availability().map_or("none", availability_label);
    let activity = snapshot.activity().map_or("none", activity_label);
    let last_seen = snapshot.last_seen().map_or_else(
        || String::from("none"),
        |timestamp| timestamp.get().to_string(),
    );
    let visibility = visibility_label(&snapshot.visibility);
    let expires = expiry_label(snapshot.expiry);

    println!(
        "subject={subject} context={context} resource={resource} origin={origin} availability={availability} activity={activity} last_seen={last_seen} visibility={visibility} expiry={expires}"
    );
}

fn print_summary(summary: &Summary) {
    let subject = summary.subject.0;
    let context = summary.context.0;
    let dominant_resource = summary
        .dominant_resource
        .as_ref()
        .map_or("none", |resource| resource.0);
    let dominant_origin = summary.dominant_origin.0;
    let availability = summary.availability.map_or("none", availability_label);
    let activity = summary.activity.map_or("none", activity_label);
    let last_seen = summary.last_seen.map_or_else(
        || String::from("none"),
        |timestamp| timestamp.get().to_string(),
    );
    let observed_at = summary.observed_at.get();
    let resource_count = summary.resources.len();

    println!(
        "subject={subject} context={context} dominant_resource={dominant_resource} dominant_origin={dominant_origin} availability={availability} activity={activity} last_seen={last_seen} observed_at={observed_at} resource_count={resource_count}"
    );
}

fn availability_label(value: Availability) -> &'static str {
    match value {
        Availability::Available => "available",
        Availability::Busy => "busy",
        Availability::Away => "away",
        Availability::Offline => "offline",
        Availability::Unknown => "unknown",
    }
}

fn activity_label(value: Activity) -> &'static str {
    match value {
        Activity::Idle => "idle",
        Activity::Observing => "observing",
        Activity::Navigating => "navigating",
        Activity::Editing => "editing",
        Activity::Presenting => "presenting",
        Activity::Acting => "acting",
        Activity::Custom(..) => "custom",
    }
}

fn visibility_label(value: &Visibility<&'static str>) -> &'static str {
    match value {
        Visibility::Public => "public",
        Visibility::Restricted(label) => label,
        Visibility::Private => "private",
    }
}

fn expiry_label(value: Expiry) -> String {
    match value {
        Expiry::Never => String::from("never"),
        Expiry::At(timestamp) => timestamp.get().to_string(),
    }
}
