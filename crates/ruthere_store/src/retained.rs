// Copyright 2026 the ruthere Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

/// Gap metadata returned when a retained-log query can no longer be answered
/// exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RetainedGap {
    /// The caller's requested cursor position.
    pub requested_since: u64,
    /// The oldest cursor position that is still queryable without resync.
    pub retained_floor_sequence: u64,
    /// The current sequence tail of the store.
    pub last_sequence: u64,
}

impl RetainedGap {
    /// Creates a retained-gap value from explicit sequence metadata.
    #[must_use]
    pub const fn new(
        requested_since: u64,
        retained_floor_sequence: u64,
        last_sequence: u64,
    ) -> Self {
        Self {
            requested_since,
            retained_floor_sequence,
            last_sequence,
        }
    }
}

/// Status for a retained-log query at one cursor position.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RetainedStatus {
    /// The retained log can answer the query and has more changes.
    Pending,
    /// The retained log can answer the query and has nothing newer.
    UpToDate,
    /// The retained log has compacted away part of the requested range.
    Gap(RetainedGap),
}

/// Result type for retained-log queries that may require resync.
pub type RetainedResult<T> = Result<T, RetainedGap>;
