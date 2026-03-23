// Copyright 2026 the ruthere Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use ruthere_core::{ExtensionFacet, Never, PresenceUpdate, Visibility};

use crate::PresenceEntryKey;

/// A retained store change with a store-assigned monotonic sequence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoreChange<S, C, R, I, V, E = Never>
where
    E: ExtensionFacet,
{
    /// The store-assigned sequence for this change.
    pub sequence: u64,
    /// The kind of retained store change.
    pub kind: StoreChangeKind<S, C, R, I, V, E>,
}

/// A retained store change kind.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StoreChangeKind<S, C, R, I, V, E = Never>
where
    E: ExtensionFacet,
{
    /// A published presence update.
    Published(PresenceUpdate<S, C, R, I, V, E>),
    /// An entry removed by expiry.
    Expired(StoreExpired<S, C, R, I, V>),
}

/// Retained metadata for an entry removed by expiry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoreExpired<S, C, R, I, V> {
    /// The removed entry key.
    pub key: PresenceEntryKey<S, C, R, I>,
    /// The visibility label the entry had when it expired.
    pub visibility: Visibility<V>,
}
