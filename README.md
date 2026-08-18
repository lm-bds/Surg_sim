# Surg_sim

Ha Monte-Carlo simulator for operating-room (OR) throughput, written in Rust.

## What it does

`run_simulation` models a day of scheduled surgeries and estimates how the
theatre runs: when each case actually starts vs its planned slot, how much
delay accumulates, and when the list finishes. It accounts for:

- Planned start times and a 15-minute cleaning turnaround between cases
- A **aduration estimator** that predicts how long each surgery takes
-  Replication across many replications to produce distributions, not single point estimates.

## Design

The simulator is decoupled from *how* duration is estimated via the `DurationEstimator` trait:

```rust
pub trait DurationEstimator {
    fn estimate(&self, features: &SurgeryFeatures) -> f64;
}`

This means you can swap estimation strategies without touching the simulation
core. Two are provided:

- `StubGamma` ha simple gamma-distributed sampler (same duration for every case). Useful as a baseline.
- `FeatureAwareEstimator`  knows `SurgeryFeatures` (procedure code, surgeon, patient age/gender) to produce a procedure-aware estimate, with gamma noise to reflect real intra-case variability.

## Run

``bash
argo run --release
```

The demo runs a sample surgery list through both estimators and prints the
simulated theatre timeline.

## Frontend

The matching scheduling UI lives in [`surg_front`](https://github.com/lm-bds/surg_front)
(SvelteKit), which submits surgeries to a backend built around this engine.
