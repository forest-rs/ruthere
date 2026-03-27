// Copyright 2026 the ruthere Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Runnable end-to-end example for the basic `ruthere` presence flow.

use std::{
    env,
    fmt::Write,
    io::{self, IsTerminal},
    ops::BitOr,
};

use ruthere_beacon::{ExpiryPolicy, PresenceBeacon};
use ruthere_core::{Activity, Availability, Expiry, PresenceAddress, Timestamp, Visibility};
use ruthere_store::{
    InMemoryStore, PresenceEntryKey, StoreChange, StoreChangeKind, SubjectPresenceSummary,
};

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

const COLOR_BLUE: u8 = 34;
const COLOR_CYAN: u8 = 36;
const COLOR_GREEN: u8 = 32;
const COLOR_MAGENTA: u8 = 35;
const COLOR_RED: u8 = 31;
const COLOR_WHITE: u8 = 37;
const COLOR_YELLOW: u8 = 33;

fn main() {
    let ui = Ui::detect();
    let mut store = Store::new();

    let doc = ContextId("doc-42");
    let alice_browser = PresenceAddress::new(
        SubjectId("alice"),
        doc.clone(),
        Some(ResourceId("browser-tab")),
    );
    let alice_mobile =
        PresenceAddress::new(SubjectId("alice"), doc.clone(), Some(ResourceId("mobile")));

    let alice_browser = PresenceBeacon::new(alice_browser, OriginId("session/browser"))
        .with_visibility(Visibility::Restricted("doc-members"))
        .with_expiry_policy(ExpiryPolicy::After(60));
    let alice_mobile = PresenceBeacon::new(alice_mobile, OriginId("session/mobile"))
        .with_visibility(Visibility::Restricted("doc-members"))
        .with_expiry_policy(ExpiryPolicy::After(15));

    let first_sequence = store.publish(
        alice_browser
            .heartbeat_at(Timestamp::new(100))
            .set_availability(Availability::Available)
            .set_activity(Activity::Observing),
    );

    let second_sequence = store.publish(
        alice_browser
            .heartbeat_at(Timestamp::new(110))
            .set_activity(Activity::Editing),
    );

    let third_sequence = store.publish(
        alice_mobile
            .update_at(Timestamp::new(105))
            .set_availability(Availability::Away)
            .set_activity(Activity::Observing),
    );

    ui.banner("🧭", "Basic Presence Flow");
    ui.kv(
        "published sequences",
        &format!("{first_sequence}, {second_sequence}, {third_sequence}"),
    );
    ui.kv(
        "store sequence after publish",
        &store.last_sequence().to_string(),
    );

    ui.section("📡", "Retained Changes Since #0");
    for change in store.changes_since(0) {
        print_change(&ui, &change);
    }

    let member_view = |visibility: &Visibility<&'static str>| {
        matches!(
            visibility,
            Visibility::Public | Visibility::Restricted("doc-members")
        )
    };
    let public_only =
        |visibility: &Visibility<&'static str>| matches!(visibility, Visibility::Public);

    ui.section("🔐", "Visibility-Filtered Changes Since #0");
    ui.detail("viewer", "doc-members");
    let member_changes = store.changes_since_visible(0, &member_view);
    print_changes_or_empty(
        &ui,
        &member_changes,
        "no retained changes visible to doc members",
    );
    ui.detail("viewer", "public-only");
    let public_changes = store.changes_since_visible(0, &public_only);
    print_changes_or_empty(
        &ui,
        &public_changes,
        "no retained changes visible to public-only viewers",
    );

    let browser_key = PresenceEntryKey::new(
        alice_browser.address().clone(),
        alice_browser.origin().clone(),
    );
    let browser_snapshot = store
        .snapshot(&browser_key)
        .expect("browser entry should be present after publish");

    ui.section("🧾", "Single Snapshot Lookup");
    print_snapshot(&ui, &browser_snapshot);

    ui.section("🗂️", "Raw Resource Snapshots Before Expiry");
    let mut snapshots = store.snapshots_in_context(&doc);
    snapshots.sort_by(snapshot_sort_key);
    for snapshot in &snapshots {
        print_snapshot(&ui, snapshot);
    }

    ui.section("🔐", "Visibility-Filtered Resource Views Before Expiry");
    ui.detail("viewer", "doc-members");
    let mut member_snapshots = store.snapshots_in_context_visible(&doc, &member_view);
    member_snapshots.sort_by(snapshot_sort_key);
    print_snapshots_or_empty(
        &ui,
        &member_snapshots,
        "no snapshots visible to doc members",
    );
    ui.detail("viewer", "public-only");
    let mut public_snapshots = store.snapshots_in_context_visible(&doc, &public_only);
    public_snapshots.sort_by(snapshot_sort_key);
    print_snapshots_or_empty(
        &ui,
        &public_snapshots,
        "no snapshots visible to public-only viewers",
    );

    ui.section("👤", "Projected Subject Summaries Before Expiry");
    let mut summaries = store.subject_summaries_in_context(&doc);
    summaries.sort_by(summary_sort_key);
    for summary in &summaries {
        print_summary(&ui, summary);
    }

    ui.section("🔐", "Visibility-Filtered Subject Summaries Before Expiry");
    ui.detail("viewer", "doc-members");
    let mut member_summaries = store.subject_summaries_in_context_visible(&doc, &member_view);
    member_summaries.sort_by(summary_sort_key);
    print_summaries_or_empty(
        &ui,
        &member_summaries,
        "no subject summaries visible to doc members",
    );
    ui.detail("viewer", "public-only");
    let mut public_summaries = store.subject_summaries_in_context_visible(&doc, &public_only);
    public_summaries.sort_by(summary_sort_key);
    print_summaries_or_empty(
        &ui,
        &public_summaries,
        "no subject summaries visible to public-only viewers",
    );

    let removed = store.expire(Timestamp::new(125));
    ui.section("⏳", "Expiry Applied At t=125");
    ui.kv("removed entries", &removed.to_string());

    ui.section("📡", "Retained Changes Since #3");
    for change in store.changes_since(3) {
        print_change(&ui, &change);
    }

    ui.section("🔐", "Visibility-Filtered Changes Since #3");
    ui.detail("viewer", "doc-members");
    let member_changes = store.changes_since_visible(3, &member_view);
    print_changes_or_empty(
        &ui,
        &member_changes,
        "no retained changes visible to doc members",
    );
    ui.detail("viewer", "public-only");
    let public_changes = store.changes_since_visible(3, &public_only);
    print_changes_or_empty(
        &ui,
        &public_changes,
        "no retained changes visible to public-only viewers",
    );

    ui.section("🗂️", "Raw Resource Snapshots After Expiry");
    let mut snapshots = store.snapshots_in_context(&doc);
    snapshots.sort_by(snapshot_sort_key);
    for snapshot in &snapshots {
        print_snapshot(&ui, snapshot);
    }

    ui.section("👤", "Projected Subject Summaries After Expiry");
    let mut summaries = store.subject_summaries_in_context(&doc);
    summaries.sort_by(summary_sort_key);
    for summary in &summaries {
        print_summary(&ui, summary);
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

fn print_snapshot(ui: &Ui, snapshot: &Snapshot) {
    let subject = snapshot.address.subject.0;
    let context = snapshot.address.context.0;
    let resource = snapshot
        .address
        .resource
        .as_ref()
        .map_or("none", |resource| resource.0);
    let origin = snapshot.origin.0;
    let last_seen = snapshot.last_seen().map_or_else(
        || String::from("none"),
        |timestamp| timestamp.get().to_string(),
    );
    let visibility = visibility_label(&snapshot.visibility);
    let expires = expiry_label(snapshot.expiry);

    ui.item(&format!("{subject} in {context} via {resource}"));
    ui.detail("origin", origin);
    ui.detail(
        "availability",
        &ui.availability_value(snapshot.availability()),
    );
    ui.detail("activity", &ui.activity_value(snapshot.activity()));
    ui.detail("last seen", &last_seen);
    ui.detail("visibility", &ui.visibility_value(visibility));
    ui.detail("expiry", &expires);
}

fn print_snapshots_or_empty(ui: &Ui, snapshots: &[Snapshot], empty: &str) {
    if snapshots.is_empty() {
        ui.empty(empty);
        return;
    }

    for snapshot in snapshots {
        print_snapshot(ui, snapshot);
    }
}

fn print_summary(ui: &Ui, summary: &Summary) {
    let subject = summary.subject.0;
    let context = summary.context.0;
    let dominant_resource = summary
        .dominant_resource
        .as_ref()
        .map_or("none", |resource| resource.0);
    let dominant_origin = summary.dominant_origin.0;
    let last_seen = summary.last_seen.map_or_else(
        || String::from("none"),
        |timestamp| timestamp.get().to_string(),
    );
    let observed_at = summary.observed_at.get();
    let resource_count = summary.resources.len();

    ui.item(&format!("{subject} in {context}"));
    ui.detail(
        "headline",
        &ui.headline_value(summary.availability, summary.activity),
    );
    ui.detail("dominant resource", dominant_resource);
    ui.detail("dominant origin", dominant_origin);
    ui.detail("last seen", &last_seen);
    ui.detail("observed at", &observed_at.to_string());
    ui.detail("resource count", &resource_count.to_string());
}

fn print_summaries_or_empty(ui: &Ui, summaries: &[Summary], empty: &str) {
    if summaries.is_empty() {
        ui.empty(empty);
        return;
    }

    for summary in summaries {
        print_summary(ui, summary);
    }
}

fn print_changes_or_empty(
    ui: &Ui,
    changes: &[StoreChange<SubjectId, ContextId, ResourceId, OriginId, &'static str>],
    empty: &str,
) {
    if changes.is_empty() {
        ui.empty(empty);
        return;
    }

    for change in changes {
        print_change(ui, change);
    }
}

fn print_change(
    ui: &Ui,
    change: &StoreChange<SubjectId, ContextId, ResourceId, OriginId, &'static str>,
) {
    match &change.kind {
        StoreChangeKind::Published(update) => {
            let subject = update.address.subject.0;
            let context = update.address.context.0;
            let resource = update
                .address
                .resource
                .as_ref()
                .map_or("none", |resource| resource.0);
            let origin = update.origin.0;
            let change_count = update.changes.len();
            ui.item(&format!("#{} published", change.sequence));
            ui.detail("subject", subject);
            ui.detail("context", context);
            ui.detail("resource", resource);
            ui.detail("origin", origin);
            ui.detail("facet changes", &change_count.to_string());
        }
        StoreChangeKind::Expired(expired) => {
            let subject = expired.key.address.subject.0;
            let context = expired.key.address.context.0;
            let resource = expired
                .key
                .address
                .resource
                .as_ref()
                .map_or("none", |resource| resource.0);
            let origin = expired.key.origin.0;
            ui.item(&format!("#{} expired", change.sequence));
            ui.detail("subject", subject);
            ui.detail("context", context);
            ui.detail("resource", resource);
            ui.detail("origin", origin);
            ui.detail(
                "visibility",
                &ui.visibility_value(visibility_label(&expired.visibility)),
            );
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Ui {
    color: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StyleFlags(u8);

impl StyleFlags {
    const NONE: Self = Self(0);
    const BOLD: Self = Self(1 << 0);
    const DIM: Self = Self(1 << 1);

    fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

impl BitOr for StyleFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl Ui {
    fn detect() -> Self {
        Self {
            color: io::stdout().is_terminal() && env::var_os("NO_COLOR").is_none(),
        }
    }

    fn banner(&self, icon: &str, title: &str) {
        println!(
            "{}",
            self.paint(&format!("{icon} {title}"), COLOR_CYAN, StyleFlags::BOLD)
        );
    }

    fn section(&self, icon: &str, title: &str) {
        println!();
        println!(
            "{}",
            self.paint(&format!("{icon} {title}"), COLOR_CYAN, StyleFlags::BOLD)
        );
    }

    fn item(&self, title: &str) {
        println!(
            "  {}",
            self.paint(&format!("• {title}"), COLOR_WHITE, StyleFlags::BOLD)
        );
    }

    fn empty(&self, value: &str) {
        println!(
            "  {}",
            self.paint(&format!("(none) {value}"), COLOR_WHITE, StyleFlags::NONE)
        );
    }

    fn kv(&self, label: &str, value: &str) {
        println!(
            "  {} {}",
            self.paint(&format!("{label:>24}:"), COLOR_WHITE, StyleFlags::NONE),
            value
        );
    }

    fn detail(&self, label: &str, value: &str) {
        println!(
            "    {} {}",
            self.paint(&format!("{label:>18}:"), COLOR_WHITE, StyleFlags::NONE),
            value
        );
    }

    fn availability_value(&self, value: Option<Availability>) -> String {
        match value {
            Some(Availability::Available) => self.paint("available", COLOR_GREEN, StyleFlags::BOLD),
            Some(Availability::Busy) => self.paint("busy", COLOR_RED, StyleFlags::BOLD),
            Some(Availability::Away) => self.paint("away", COLOR_YELLOW, StyleFlags::BOLD),
            Some(Availability::Offline) => self.paint("offline", COLOR_WHITE, StyleFlags::NONE),
            Some(Availability::Unknown) => String::from("unknown"),
            None => String::from("none"),
        }
    }

    fn activity_value(&self, value: Option<Activity>) -> String {
        match value {
            Some(Activity::Editing) => self.paint("editing", COLOR_MAGENTA, StyleFlags::BOLD),
            Some(Activity::Presenting) => self.paint("presenting", COLOR_BLUE, StyleFlags::BOLD),
            Some(Activity::Acting) => self.paint("acting", COLOR_CYAN, StyleFlags::BOLD),
            Some(Activity::Observing) => self.paint("observing", COLOR_GREEN, StyleFlags::NONE),
            Some(Activity::Navigating) => String::from("navigating"),
            Some(Activity::Idle) => String::from("idle"),
            Some(Activity::Custom(..)) => String::from("custom"),
            None => String::from("none"),
        }
    }

    fn visibility_value(&self, value: &'static str) -> String {
        self.paint(value, COLOR_BLUE, StyleFlags::NONE)
    }

    fn headline_value(
        &self,
        availability: Option<Availability>,
        activity: Option<Activity>,
    ) -> String {
        format!(
            "{} • {}",
            self.availability_value(availability),
            self.activity_value(activity)
        )
    }

    fn paint(&self, value: &str, color: u8, flags: StyleFlags) -> String {
        if !self.color {
            return value.to_string();
        }

        let mut codes = String::new();
        if flags.contains(StyleFlags::BOLD) {
            codes.push('1');
        }
        if flags.contains(StyleFlags::DIM) {
            if !codes.is_empty() {
                codes.push(';');
            }
            codes.push('2');
        }
        if !codes.is_empty() {
            codes.push(';');
        }
        let _ = write!(&mut codes, "{color}");

        format!("\u{1b}[{codes}m{value}\u{1b}[0m")
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
