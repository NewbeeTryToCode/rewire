//! ACT-R base-level activation from the event log: B(i) = ln( sum_j w_j * max(age_h, 0.5)^(-d) ).
//! Only real use counts (exposure in a snapshot or recall listing does not), at most one event per
//! node per session (highest weight). Node creation counts as the first presentation.

pub const USE_EVENTS: [&str; 2] = ["use", "confirm"];
pub const MIN_AGE_H: f64 = 0.5;
pub const FLOOR: f64 = -20.0;

/// Map activation to 0..1 for mixing with text relevance.
pub fn squash(b: f64) -> f64 {
    1.0 / (1.0 + (-(b + 2.0)).exp())
}
