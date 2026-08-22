# Surg_sim

A reproducible operating-room duration and schedule simulator in Rust. It contains a pluggable duration-estimator interface, a Monte Carlo room-schedule engine, a CLI example and a small HTTP prediction service.

The bundled coefficients are illustrative defaults, not a fitted or clinically validated model.

## Verify

```bash
cd surg
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

## CLI simulation

```bash
cd surg
cargo run --bin surg
```

This runs both estimators and writes `simulation_results.json`.

## Prediction API

```bash
cd surg
cargo run --bin server
```

The server listens on `127.0.0.1:3001` by default. Override it with `SURG_ADDR`.
Browser requests are accepted from `http://localhost:5173` by default; set
`SURG_ALLOWED_ORIGIN` to the deployed frontend origin.

```bash
curl -X POST http://127.0.0.1:3001/api/predict-duration \
  -H 'content-type: application/json' \
  -d '{"surgeries":[{"surgeon":"Dr. A","procedure":"ProcC","diagnosis":null,"predictedStart":null}]}'
```

Response:

```json
{"predictions":[{"procedure":"ProcC","surgeon":"Dr. A","predictedMinutes":86.0}]}
```

The exact duration is reproducible for a request but is a stochastic draw from an illustrative Gamma model. The companion [surg_front](https://github.com/lm-bds/surg_front) client uses this endpoint.

## Library API

Implement `DurationEstimator::sample(&mut self, &SurgeryFeatures)` to plug in another estimator. `run_simulation(schedule, estimator, replications)` returns every sampled case record and delay.

## Licence

MIT.
