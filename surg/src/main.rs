// src/main.rs

mod estimator;
mod simulator;
use std::fs;

use estimator::{Gender, StubGamma, SurgeryFeatures};
use simulator::{run_simulation, SimulationResults};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Define today’s schedule with *estimated* start times (in minutes)
    let schedule = vec![
        SurgeryFeatures {
            procedure_code: "ProcA".into(),
            surgeon_id: "Dr. A".into(),
            patient_age: 56,
            patient_gender: Gender::Female,
            estimated_start: 0.0, // first case at t=0
        },
        SurgeryFeatures {
            procedure_code: "ProcB".into(),
            surgeon_id: "Dr. B".into(),
            patient_age: 72,
            patient_gender: Gender::Male,
            estimated_start: 70.0, // planned 1-hour slot
        },
        SurgeryFeatures {
            procedure_code: "ProcC".into(),
            surgeon_id: "Dr. A".into(),
            patient_age: 45,
            patient_gender: Gender::Female,
            estimated_start: 120.0, // planned 2-hour slot
        },
        SurgeryFeatures {
            procedure_code: "ProcD".into(),
            surgeon_id: "Dr. C".into(),
            patient_age: 63,
            patient_gender: Gender::Male,
            estimated_start: 180.0, // planned 3-hour slot
        },
        SurgeryFeatures {
            procedure_code: "ProcE".into(),
            surgeon_id: "Dr. B".into(),
            patient_age: 50,
            patient_gender: Gender::Female,
            estimated_start: 240.0, // planned 4-hour slot
        },
    ];

    // 2. Instantiate your (stub) estimator
    let mut estimator = StubGamma::new(2.0, 30.0, 42);

    // 3. Run a single replication (or more if you like)
    let results: SimulationResults = run_simulation(schedule.clone(), &mut estimator, 1);

    // 4. Print comparison of estimated vs actual
    for (rep_idx, rep) in results.reps.iter().enumerate() {
        println!("\n=== Replication {} ===", rep_idx + 1);
        for (case_idx, record) in rep.records.iter().enumerate() {
            let est = schedule[case_idx].estimated_start;
            let actual_start = record.actual_start;
            let duration = record.duration;
            let end_time = record.end_time;
            let delay = actual_start - est;
            println!(
                "Case {:>2}: est_start={:>6.1} | act_start={:>6.1} | delay={:>6.1} | dur={:>6.1} | end={:>6.1}",
                case_idx + 1, est, actual_start, delay, duration, end_time
            );
        }
    }
    let json = serde_json::to_string_pretty(&results)?;
    fs::write("simulation_results.json", json)?;

    println!("✅ Results exported to simulation_results.json");
    Ok(())
}
