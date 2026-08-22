// src/simulator.rs
use crate::estimator::{DurationEstimator, SurgeryFeatures};
use serde::Serialize;

/// Holds the actual timing for one surgery case
#[derive(Debug, Clone, Serialize)]
pub struct CaseRecord {
    /// The actual start time (minutes since day‐start)
    pub actual_start: f64,
    /// The sampled surgery duration (minutes)
    pub duration: f64,
    /// The actual end time (minutes since day‐start)
    pub end_time: f64,

    pub delay: f64,

    /// The estimated start time (minutes since day‐start)
    pub estimated_start: f64,
}

/// Results of one run through the day’s schedule
#[derive(Debug, Clone, Serialize)]
pub struct Replication {
    pub records: Vec<CaseRecord>,
}

/// Aggregated results over multiple replications
#[derive(Debug, Clone, Serialize)]
pub struct SimulationResults {
    pub reps: Vec<Replication>,
}

/// Cleaning time after each surgery (minutes)
const CLEANING_TIME: f64 = 15.0;

impl SimulationResults {
    /// Convenience: pull out the vector of replications
    pub fn new() -> Self {
        SimulationResults { reps: Vec::new() }
    }
}

impl Default for SimulationResults {
    fn default() -> Self {
        Self::new()
    }
}

/// Run the day’s schedule `num_replications` times
pub fn run_simulation<E: DurationEstimator>(
    schedule: Vec<SurgeryFeatures>,
    estimator: &mut E,
    num_replications: usize,
) -> SimulationResults {
    let mut results = SimulationResults::new();

    for _ in 0..num_replications {
        let mut records = Vec::with_capacity(schedule.len());
        let mut available_time = 0.0;

        for feature in &schedule {
            // Determine when we can actually start:
            // either the room is free, or the planned estimate
            let actual_start = if feature.estimated_start > available_time {
                feature.estimated_start
            } else {
                available_time
            };

            // Sample the surgical duration
            let duration = estimator.sample(feature);
            let end_time = actual_start + duration;

            // Record this case’s timings
            records.push(CaseRecord {
                actual_start,
                duration,
                end_time,
                delay: actual_start - feature.estimated_start,
                estimated_start: feature.estimated_start,
            });

            // Block the room for cleaning after the case
            available_time = end_time + CLEANING_TIME;
        }

        results.reps.push(Replication { records });
    }

    results
}
