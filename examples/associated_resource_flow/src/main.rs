// Copyright 2026 the ruthere Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Runnable example for direct and associated resources in `ruthere`.

use core::cmp::Ordering;

use ruthere_core::{
    Activity, Availability, ExtensionFacet, PresenceAddress, PresenceFacet, PresenceSnapshot,
    PresenceUpdate, Timestamp, Visibility,
};
use ruthere_store::{InMemoryStore, SubjectPresenceSummary};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct SubjectId(&'static str);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ContextId(&'static str);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ResourceId(&'static str);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct OriginId(&'static str);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum SubjectResourceRelationship {
    Owned,
    Associated,
    External,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum ResourceClass {
    Device,
    Tracker,
    Shipment,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum DeliveryStatus {
    OutForDelivery,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum ExampleFacet {
    Relationship(SubjectResourceRelationship),
    ResourceClass(ResourceClass),
    DeliveryStatus(DeliveryStatus),
    TrackerLocation(&'static str),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum ExampleFacetKind {
    Relationship,
    ResourceClass,
    DeliveryStatus,
    TrackerLocation,
}

impl ExtensionFacet for ExampleFacet {
    type Kind = ExampleFacetKind;

    fn kind(&self) -> Self::Kind {
        match self {
            Self::Relationship(..) => ExampleFacetKind::Relationship,
            Self::ResourceClass(..) => ExampleFacetKind::ResourceClass,
            Self::DeliveryStatus(..) => ExampleFacetKind::DeliveryStatus,
            Self::TrackerLocation(..) => ExampleFacetKind::TrackerLocation,
        }
    }
}

type Store = InMemoryStore<SubjectId, ContextId, ResourceId, OriginId, &'static str, ExampleFacet>;
type Snapshot =
    PresenceSnapshot<SubjectId, ContextId, ResourceId, OriginId, &'static str, ExampleFacet>;
type Summary =
    SubjectPresenceSummary<SubjectId, ContextId, ResourceId, OriginId, &'static str, ExampleFacet>;

fn main() {
    let mut store = Store::new();
    let travel = ContextId("travel");

    let laptop = PresenceAddress::new(
        SubjectId("bruce"),
        travel.clone(),
        Some(ResourceId("device:laptop")),
    );
    let keys = PresenceAddress::new(
        SubjectId("bruce"),
        travel.clone(),
        Some(ResourceId("tracker:keys")),
    );
    let package = PresenceAddress::new(
        SubjectId("bruce"),
        travel.clone(),
        Some(ResourceId("shipment:1Z999")),
    );

    let desktop_client = OriginId("desktop-client");
    let tracker_bridge = OriginId("find-my-bridge");
    let courier_bridge = OriginId("courier-bridge");

    let laptop_sequence = store.publish(
        PresenceUpdate::new(
            laptop,
            desktop_client,
            Visibility::Restricted("travel-party"),
            Timestamp::new(100),
            ruthere_core::Expiry::At(Timestamp::new(160)),
        )
        .set_availability(Availability::Available)
        .set_activity(Activity::Editing)
        .set_extension(ExampleFacet::Relationship(
            SubjectResourceRelationship::Owned,
        ))
        .set_extension(ExampleFacet::ResourceClass(ResourceClass::Device)),
    );

    let keys_sequence = store.publish(
        PresenceUpdate::new(
            keys,
            tracker_bridge,
            Visibility::Restricted("travel-party"),
            Timestamp::new(97),
            ruthere_core::Expiry::At(Timestamp::new(140)),
        )
        .set_last_seen(Timestamp::new(97))
        .set_extension(ExampleFacet::Relationship(
            SubjectResourceRelationship::Associated,
        ))
        .set_extension(ExampleFacet::ResourceClass(ResourceClass::Tracker))
        .set_extension(ExampleFacet::TrackerLocation("office")),
    );

    let package_sequence = store.publish(
        PresenceUpdate::new(
            package,
            courier_bridge,
            Visibility::Public,
            Timestamp::new(102),
            ruthere_core::Expiry::At(Timestamp::new(220)),
        )
        .set_extension(ExampleFacet::Relationship(
            SubjectResourceRelationship::External,
        ))
        .set_extension(ExampleFacet::ResourceClass(ResourceClass::Shipment))
        .set_extension(ExampleFacet::DeliveryStatus(DeliveryStatus::OutForDelivery)),
    );

    let travel_party_view = |visibility: &Visibility<&'static str>| {
        matches!(
            visibility,
            Visibility::Public | Visibility::Restricted("travel-party")
        )
    };
    let public_only =
        |visibility: &Visibility<&'static str>| matches!(visibility, Visibility::Public);

    println!("associated_resource_flow");
    println!("published sequences: {laptop_sequence}, {keys_sequence}, {package_sequence}");
    println!();

    let travel_party_summary = store
        .subject_summary_in_context_visible(&SubjectId("bruce"), &travel, &travel_party_view)
        .expect("travel-party summary should exist");
    println!("travel-party summary");
    print_summary(&travel_party_summary);
    println!();

    let public_summary = store
        .subject_summary_in_context_visible(&SubjectId("bruce"), &travel, &public_only)
        .expect("public summary should exist");
    println!("public summary");
    print_summary(&public_summary);
    println!();

    let mut visible_resources = store.snapshots_in_context_visible(&travel, &travel_party_view);
    visible_resources.sort_by(compare_snapshots);
    println!("travel-party resource drill-down");
    for snapshot in &visible_resources {
        print_snapshot(snapshot);
    }
}

fn compare_snapshots(left: &Snapshot, right: &Snapshot) -> Ordering {
    left.address
        .resource
        .cmp(&right.address.resource)
        .then_with(|| left.origin.cmp(&right.origin))
}

fn print_summary(summary: &Summary) {
    let dominant = summary
        .dominant_resource
        .as_ref()
        .map_or("none", |resource| resource.0);

    println!("  subject: {}", summary.subject.0);
    println!("  context: {}", summary.context.0);
    println!(
        "  headline: availability={:?}, activity={:?}",
        summary.availability, summary.activity
    );
    println!("  dominant resource: {dominant}");
    println!("  dominant origin: {}", summary.dominant_origin.0);
    println!("  raw resources: {}", summary.resources.len());
}

fn print_snapshot(snapshot: &Snapshot) {
    let resource = snapshot
        .address
        .resource
        .as_ref()
        .map_or("none", |resource| resource.0);

    println!("  resource: {resource}");
    println!("    origin: {}", snapshot.origin.0);
    println!("    visibility: {}", visibility_label(&snapshot.visibility));
    println!(
        "    builtins: availability={:?}, activity={:?}, last_seen={:?}",
        snapshot.availability(),
        snapshot.activity(),
        snapshot.last_seen()
    );
    println!(
        "    relationship: {}",
        relationship(snapshot).map_or("none", relationship_label)
    );
    println!(
        "    class: {}",
        resource_class(snapshot).map_or("none", resource_class_label)
    );
    if let Some(status) = delivery_status(snapshot) {
        println!("    delivery status: {}", delivery_status_label(status));
    }
    if let Some(location) = tracker_location(snapshot) {
        println!("    tracker location: {location}");
    }
}

fn relationship(snapshot: &Snapshot) -> Option<SubjectResourceRelationship> {
    snapshot.facets.iter().find_map(|facet| match facet {
        PresenceFacet::Extension(ExampleFacet::Relationship(value)) => Some(*value),
        PresenceFacet::Builtin(..)
        | PresenceFacet::Extension(
            ExampleFacet::ResourceClass(..)
            | ExampleFacet::DeliveryStatus(..)
            | ExampleFacet::TrackerLocation(..),
        ) => None,
    })
}

fn resource_class(snapshot: &Snapshot) -> Option<ResourceClass> {
    snapshot.facets.iter().find_map(|facet| match facet {
        PresenceFacet::Extension(ExampleFacet::ResourceClass(value)) => Some(*value),
        PresenceFacet::Builtin(..)
        | PresenceFacet::Extension(
            ExampleFacet::Relationship(..)
            | ExampleFacet::DeliveryStatus(..)
            | ExampleFacet::TrackerLocation(..),
        ) => None,
    })
}

fn delivery_status(snapshot: &Snapshot) -> Option<DeliveryStatus> {
    snapshot.facets.iter().find_map(|facet| match facet {
        PresenceFacet::Extension(ExampleFacet::DeliveryStatus(value)) => Some(*value),
        PresenceFacet::Builtin(..)
        | PresenceFacet::Extension(
            ExampleFacet::Relationship(..)
            | ExampleFacet::ResourceClass(..)
            | ExampleFacet::TrackerLocation(..),
        ) => None,
    })
}

fn tracker_location(snapshot: &Snapshot) -> Option<&'static str> {
    snapshot.facets.iter().find_map(|facet| match facet {
        PresenceFacet::Extension(ExampleFacet::TrackerLocation(value)) => Some(*value),
        PresenceFacet::Builtin(..)
        | PresenceFacet::Extension(
            ExampleFacet::Relationship(..)
            | ExampleFacet::ResourceClass(..)
            | ExampleFacet::DeliveryStatus(..),
        ) => None,
    })
}

fn relationship_label(value: SubjectResourceRelationship) -> &'static str {
    match value {
        SubjectResourceRelationship::Owned => "owned",
        SubjectResourceRelationship::Associated => "associated",
        SubjectResourceRelationship::External => "external",
    }
}

fn resource_class_label(value: ResourceClass) -> &'static str {
    match value {
        ResourceClass::Device => "device",
        ResourceClass::Tracker => "tracker",
        ResourceClass::Shipment => "shipment",
    }
}

fn delivery_status_label(value: DeliveryStatus) -> &'static str {
    match value {
        DeliveryStatus::OutForDelivery => "out-for-delivery",
    }
}

fn visibility_label(value: &Visibility<&'static str>) -> &'static str {
    match value {
        Visibility::Public => "public",
        Visibility::Restricted(label) => label,
        Visibility::Private => "private",
    }
}
