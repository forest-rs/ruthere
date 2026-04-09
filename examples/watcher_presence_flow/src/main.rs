// Copyright 2026 the ruthere Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Runnable end-to-end example for the watcher-oriented `ruthere` flow.

use std::{
    env,
    fmt::Write,
    io::{self, IsTerminal},
    ops::BitOr,
};

use ruthere_beacon::{ExpiryPolicy, PresenceBeacon};
use ruthere_core::{
    Activity, Availability, BuiltinFacet, FacetChange, PresenceAddress, PresenceFacet, Timestamp,
    Visibility,
};
use ruthere_server::{
    PresenceServer, StoreChange, StoreChangeKind, VisibilityPolicy, WatcherCursor,
};
use ruthere_store::{RetainedGap, RetainedStatus};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct SubjectId(&'static str);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ContextId(&'static str);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ResourceId(&'static str);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct OriginId(&'static str);

type Server = PresenceServer<SubjectId, ContextId, ResourceId, OriginId, &'static str>;
type SummaryView = ruthere_store::SubjectPresenceSummary<
    SubjectId,
    ContextId,
    ResourceId,
    OriginId,
    &'static str,
    ruthere_core::Never,
>;
type Change =
    StoreChange<SubjectId, ContextId, ResourceId, OriginId, &'static str, ruthere_core::Never>;

const COLOR_BLUE: u8 = 34;
const COLOR_CYAN: u8 = 36;
const COLOR_GREEN: u8 = 32;
const COLOR_MAGENTA: u8 = 35;
const COLOR_RED: u8 = 31;
const COLOR_WHITE: u8 = 37;
const COLOR_YELLOW: u8 = 33;

fn main() {
    let ui = Ui::detect();
    let mut server = Server::new();

    let doc = ContextId("doc-42");
    let alice_browser = PresenceAddress::new(
        SubjectId("alice"),
        doc.clone(),
        Some(ResourceId("browser-tab")),
    );
    let alice_mobile =
        PresenceAddress::new(SubjectId("alice"), doc.clone(), Some(ResourceId("mobile")));
    let bob_browser = PresenceAddress::new(
        SubjectId("bob"),
        doc.clone(),
        Some(ResourceId("browser-tab")),
    );

    let alice_browser = PresenceBeacon::new(alice_browser, OriginId("session/browser"))
        .with_visibility(Visibility::Restricted("doc-members"))
        .with_expiry_policy(ExpiryPolicy::After(70));
    let alice_mobile = PresenceBeacon::new(alice_mobile, OriginId("session/mobile"))
        .with_visibility(Visibility::Restricted("doc-members"))
        .with_expiry_policy(ExpiryPolicy::After(5));
    let bob_browser = PresenceBeacon::new(bob_browser, OriginId("session/presenter"))
        .with_visibility(Visibility::Public)
        .with_expiry_policy(ExpiryPolicy::After(69));

    let member_view = |visibility: &Visibility<&'static str>| {
        matches!(
            visibility,
            Visibility::Public | Visibility::Restricted("doc-members")
        )
    };
    let public_only =
        |visibility: &Visibility<&'static str>| matches!(visibility, Visibility::Public);

    let mut member_cursor = server.watcher_cursor();
    let mut public_cursor = server.watcher_cursor();

    ui.banner("👀", "Watcher Presence Flow");
    ui.kv("context", doc.0);
    ui.kv("server", "process-local in-memory server");
    ui.kv("watchers", "doc-members, public-only");

    ui.section("🔎", "Initial Poll");
    poll_viewer(
        &ui,
        &server,
        &doc,
        "doc-members watcher",
        &mut member_cursor,
        &member_view,
    );
    poll_viewer(
        &ui,
        &server,
        &doc,
        "public-only watcher",
        &mut public_cursor,
        &public_only,
    );

    ui.section("1️⃣", "Restricted Edit Session Starts");
    let sequence = server.receive(
        alice_browser
            .heartbeat_at(Timestamp::new(100))
            .set_availability(Availability::Available)
            .set_activity(Activity::Editing),
    );
    print_publish(
        &ui,
        sequence,
        "alice begins editing from a browser tab",
        "doc-members",
    );
    poll_viewer(
        &ui,
        &server,
        &doc,
        "doc-members watcher",
        &mut member_cursor,
        &member_view,
    );
    poll_viewer(
        &ui,
        &server,
        &doc,
        "public-only watcher",
        &mut public_cursor,
        &public_only,
    );

    ui.section("2️⃣", "Public Viewer Appears");
    let sequence = server.receive(
        bob_browser
            .heartbeat_at(Timestamp::new(111))
            .set_availability(Availability::Available)
            .set_activity(Activity::Observing),
    );
    print_publish(&ui, sequence, "bob arrives as a public observer", "public");
    poll_viewer(
        &ui,
        &server,
        &doc,
        "doc-members watcher",
        &mut member_cursor,
        &member_view,
    );
    poll_viewer(
        &ui,
        &server,
        &doc,
        "public-only watcher",
        &mut public_cursor,
        &public_only,
    );

    ui.section("3️⃣", "Second Resource Joins");
    let sequence = server.receive(
        alice_mobile
            .update_at(Timestamp::new(115))
            .set_availability(Availability::Away)
            .set_activity(Activity::Observing),
    );
    print_publish(
        &ui,
        sequence,
        "alice also appears from mobile with a weaker signal",
        "doc-members",
    );
    poll_viewer(
        &ui,
        &server,
        &doc,
        "doc-members watcher",
        &mut member_cursor,
        &member_view,
    );
    poll_viewer(
        &ui,
        &server,
        &doc,
        "public-only watcher",
        &mut public_cursor,
        &public_only,
    );

    ui.section("😴", "Idle Poll");
    poll_viewer(
        &ui,
        &server,
        &doc,
        "doc-members watcher",
        &mut member_cursor,
        &member_view,
    );
    poll_viewer(
        &ui,
        &server,
        &doc,
        "public-only watcher",
        &mut public_cursor,
        &public_only,
    );

    ui.section("⏳", "Expiry At t=125");
    let removed = server.expire(Timestamp::new(125));
    ui.kv("removed entries", &removed.to_string());
    poll_viewer(
        &ui,
        &server,
        &doc,
        "doc-members watcher",
        &mut member_cursor,
        &member_view,
    );
    poll_viewer(
        &ui,
        &server,
        &doc,
        "public-only watcher",
        &mut public_cursor,
        &public_only,
    );

    ui.section("🧹", "Compaction And Resync");
    let compacted = server.compact_changes_through(3);
    ui.kv("removed retained changes", &compacted.to_string());
    ui.kv(
        "retained floor",
        &format!("#{}", server.retained_floor_sequence()),
    );

    let mut stale_cursor = WatcherCursor::new();
    ui.item("late doc-members watcher");
    ui.detail("cursor before", &format!("#{}", stale_cursor.sequence()));
    match server.poll_visible(&mut stale_cursor, &member_view) {
        Ok(changes) => {
            ui.detail("changes", &changes.len().to_string());
            ui.detail("cursor after", &format!("#{}", stale_cursor.sequence()));
        }
        Err(gap) => {
            ui.detail("changes", &ui.warning_value("gap"));
            print_retained_gap(&ui, gap);
            let baseline = server
                .store()
                .subject_summaries_in_context_visible(&doc, &member_view);
            ui.detail("baseline subjects", &baseline.len().to_string());

            stale_cursor.reset_to(gap.last_sequence);
            ui.detail(
                "cursor after rebuild",
                &format!("#{}", stale_cursor.sequence()),
            );

            let changes = server
                .poll_visible(&mut stale_cursor, &member_view)
                .expect("cursor should be queryable after baseline rebuild");
            ui.detail("changes after rebuild", &changes.len().to_string());
            ui.detail("cursor after", &format!("#{}", stale_cursor.sequence()));
            for change in &changes {
                print_change(&ui, change);
            }
        }
    }
}

fn poll_viewer<P>(
    ui: &Ui,
    server: &Server,
    context: &ContextId,
    watcher: &str,
    cursor: &mut WatcherCursor,
    visibility: &P,
) where
    P: VisibilityPolicy<&'static str>,
{
    let before = cursor.sequence();

    ui.item(watcher);
    ui.detail("cursor before", &format!("#{before}"));

    match server.pending_status_visible(*cursor, visibility) {
        RetainedStatus::UpToDate => {
            ui.detail("changes", &ui.muted_value("none"));
            ui.detail("cursor after", &format!("#{before}"));
            ui.detail("summary refresh", &ui.muted_value("skipped"));
            return;
        }
        RetainedStatus::Gap(gap) => {
            ui.detail("changes", &ui.warning_value("gap"));
            ui.detail("cursor after", &format!("#{before}"));
            ui.detail(
                "summary refresh",
                &ui.warning_value("baseline rebuild required"),
            );
            print_retained_gap(ui, gap);
            return;
        }
        RetainedStatus::Pending => {}
    }

    let changes = server
        .poll_visible(cursor, visibility)
        .expect("pending status should make visible poll queryable");
    let after = cursor.sequence();

    ui.detail("changes", &changes.len().to_string());
    ui.detail("cursor after", &ui.cursor_value(&format!("#{after}")));
    ui.detail("summary refresh", &ui.success_value("applied"));

    for change in &changes {
        print_change(ui, change);
    }

    let mut summaries = server
        .store()
        .subject_summaries_in_context_visible(context, visibility);
    summaries.sort_by(summary_sort_key);

    ui.detail("visible subjects", &summaries.len().to_string());
    for summary in &summaries {
        print_summary(ui, summary);
    }
}

fn print_retained_gap(ui: &Ui, gap: RetainedGap) {
    ui.subitem("retained log gap");
    ui.subdetail("requested since", &format!("#{}", gap.requested_since));
    ui.subdetail(
        "retained floor",
        &format!("#{}", gap.retained_floor_sequence),
    );
    ui.subdetail("store tail", &format!("#{}", gap.last_sequence));
}

fn print_publish(ui: &Ui, sequence: u64, title: &str, visibility: &'static str) {
    ui.item(&format!("#{sequence} published"));
    ui.detail("event", title);
    ui.detail("visibility", &ui.visibility_value(visibility));
}

fn print_change(ui: &Ui, change: &Change) {
    match &change.kind {
        StoreChangeKind::Published(update) => {
            let subject = update.address.subject.0;
            let resource = update
                .address
                .resource
                .as_ref()
                .map_or("none", |resource| resource.0);
            let visibility = visibility_label(&update.visibility);

            ui.subitem(&format!(
                "#{} published {subject} via {resource}",
                change.sequence
            ));
            ui.subdetail("origin", update.origin.0);
            ui.subdetail("visibility", &ui.visibility_value(visibility));
            ui.subdetail("delta", &describe_changes(&update.changes));
        }
        StoreChangeKind::Expired(expired) => {
            let subject = expired.key.address.subject.0;
            let resource = expired
                .key
                .address
                .resource
                .as_ref()
                .map_or("none", |resource| resource.0);
            let visibility = visibility_label(&expired.visibility);

            ui.subitem(&format!(
                "#{} expired {subject} via {resource}",
                change.sequence
            ));
            ui.subdetail("origin", expired.key.origin.0);
            ui.subdetail("visibility", &ui.visibility_value(visibility));
        }
    }
}

fn print_summary(ui: &Ui, summary: &SummaryView) {
    let resource = summary
        .dominant_resource
        .as_ref()
        .map_or("none", |resource| resource.0);
    let last_seen = summary.last_seen.map_or_else(
        || String::from("none"),
        |timestamp| timestamp.get().to_string(),
    );

    ui.subitem(&format!("{} in {}", summary.subject.0, summary.context.0));
    ui.subdetail(
        "headline",
        &ui.headline_value(summary.availability, summary.activity),
    );
    ui.subdetail("dominant resource", resource);
    ui.subdetail("dominant origin", summary.dominant_origin.0);
    ui.subdetail("last seen", &last_seen);
    ui.subdetail("observed at", &summary.observed_at.get().to_string());
    ui.subdetail("raw snapshots", &summary.resources.len().to_string());
}

fn summary_sort_key(left: &SummaryView, right: &SummaryView) -> core::cmp::Ordering {
    left.subject.cmp(&right.subject)
}

fn describe_changes(changes: &[FacetChange<ruthere_core::Never>]) -> String {
    let mut description = String::new();

    for (index, change) in changes.iter().enumerate() {
        if index > 0 {
            description.push_str(", ");
        }

        match change {
            FacetChange::Set(PresenceFacet::Builtin(BuiltinFacet::Availability(value))) => {
                let _ = write!(description, "availability={}", availability_label(*value));
            }
            FacetChange::Set(PresenceFacet::Builtin(BuiltinFacet::Activity(value))) => {
                let _ = write!(description, "activity={}", activity_label(*value));
            }
            FacetChange::Set(PresenceFacet::Builtin(BuiltinFacet::LastSeen(value))) => {
                let _ = write!(description, "last_seen={}", value.get());
            }
            FacetChange::Set(PresenceFacet::Extension(_)) => {
                description.push_str("extension=set");
            }
            FacetChange::Clear(kind) => {
                let _ = write!(description, "clear {}", clear_kind_label(kind));
            }
        }
    }

    description
}

fn clear_kind_label(kind: &ruthere_core::PresenceFacetKind<ruthere_core::Never>) -> &'static str {
    match kind {
        ruthere_core::PresenceFacetKind::Builtin(ruthere_core::BuiltinFacetKind::Availability) => {
            "availability"
        }
        ruthere_core::PresenceFacetKind::Builtin(ruthere_core::BuiltinFacetKind::Activity) => {
            "activity"
        }
        ruthere_core::PresenceFacetKind::Builtin(ruthere_core::BuiltinFacetKind::LastSeen) => {
            "last_seen"
        }
        ruthere_core::PresenceFacetKind::Extension(_) => "extension",
    }
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
        Activity::Custom(_) => "custom",
    }
}

fn visibility_label(value: &Visibility<&'static str>) -> &'static str {
    match value {
        Visibility::Public => "public",
        Visibility::Restricted(label) => label,
        Visibility::Private => "private",
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

    fn subitem(&self, title: &str) {
        println!(
            "      {}",
            self.paint(&format!("◦ {title}"), COLOR_WHITE, StyleFlags::BOLD)
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

    fn subdetail(&self, label: &str, value: &str) {
        println!(
            "        {} {}",
            self.paint(&format!("{label:>16}:"), COLOR_WHITE, StyleFlags::NONE),
            value
        );
    }

    fn availability_value(&self, value: Option<Availability>) -> String {
        match value {
            Some(Availability::Available) => self.paint("available", COLOR_GREEN, StyleFlags::BOLD),
            Some(Availability::Busy) => self.paint("busy", COLOR_RED, StyleFlags::BOLD),
            Some(Availability::Away) => self.paint("away", COLOR_YELLOW, StyleFlags::BOLD),
            Some(Availability::Offline) => String::from("offline"),
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

    fn cursor_value(&self, value: &str) -> String {
        self.paint(value, COLOR_CYAN, StyleFlags::BOLD)
    }

    fn muted_value(&self, value: &str) -> String {
        self.paint(value, COLOR_WHITE, StyleFlags::NONE)
    }

    fn success_value(&self, value: &str) -> String {
        self.paint(value, COLOR_GREEN, StyleFlags::BOLD)
    }

    fn warning_value(&self, value: &str) -> String {
        self.paint(value, COLOR_YELLOW, StyleFlags::BOLD)
    }

    fn paint(&self, value: &str, color: u8, flags: StyleFlags) -> String {
        if !self.color {
            return value.to_string();
        }

        let mut codes = String::new();
        if flags.contains(StyleFlags::BOLD) {
            codes.push('1');
        }
        if !codes.is_empty() {
            codes.push(';');
        }
        let _ = write!(codes, "{color}");

        format!("\u{1b}[{codes}m{value}\u{1b}[0m")
    }
}
