// Copyright 2026 the ruthere Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use ruthere_core::Visibility;

/// Policy used to decide whether a stored visibility label is observable.
pub trait VisibilityPolicy<V> {
    /// Returns `true` when the provided visibility label should be included.
    #[must_use]
    fn allows(&self, visibility: &Visibility<V>) -> bool;
}

impl<V, F> VisibilityPolicy<V> for F
where
    F: Fn(&Visibility<V>) -> bool,
{
    fn allows(&self, visibility: &Visibility<V>) -> bool {
        self(visibility)
    }
}
