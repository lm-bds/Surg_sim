use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::{Distribution, Gamma};
use std::collections::HashMap;

/// Gender of a patient (extendable if needed)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Gender {
    Male,
    Female,
    Other,
}

/// Features of a surgery case, used by the estimator
#[derive(Debug, Clone)]
pub struct SurgeryFeatures {
    pub procedure_code: String,
    pub surgeon_id: String,
    pub patient_age: u32,
    pub patient_gender: Gender,
    pub estimated_start: f64,
    // add more fields as needed
}

/// Trait for any service-time duration estimator
pub trait DurationEstimator {
    /// Sample a service time (in minutes) given the case features
    fn sample(&mut self, features: &SurgeryFeatures) -> f64;
}

/// A stub gamma-based estimator with fixed shape & scale parameters
pub struct StubGamma {
    rng: StdRng,
    shape: f64,
    scale: f64,
}

impl StubGamma {
    /// Create a new stub estimator
    ///
    /// # Arguments
    /// * `shape` - α (k) parameter of the gamma distribution
    /// * `scale` - θ parameter of the gamma distribution
    /// * `seed`  - RNG seed for reproducibility
    pub fn new(shape: f64, scale: f64, seed: u64) -> Self {
        StubGamma {
            rng: StdRng::seed_from_u64(seed),
            shape,
            scale,
        }
    }
}

impl DurationEstimator for StubGamma {
    fn sample(&mut self, _features: &SurgeryFeatures) -> f64 {
        // construct the gamma distribution
        let gamma = Gamma::new(self.shape, self.scale).expect("Invalid gamma parameters");
        // sample a duration using the RNG
        gamma.sample(&mut self.rng)
    }
}

/// A feature-aware duration estimator.
///
/// Models the *mean* surgery time as a generalised-linear function of the
/// case features (age, gender, procedure, surgeon) and then samples the
/// realised duration from a Gamma distribution centred on that mean. Keeping
/// the stochastic draw (rather than returning the point estimate) preserves
/// the Monte-Carlo behaviour the simulator relies on: each replication should
/// see realistic run-to-run variation, not a deterministic duration.
///
/// Coefficients are illustrative defaults (a real deployment would fit them
/// to historical `Duration(mins)` data, e.g. from the SurgSimNN training set).
/// The structure is what matters for the portfolio: `SurgeryFeatures` now
/// actually drives the estimate instead of being ignored.
pub struct FeatureAwareEstimator {
    rng: StdRng,
    // Shared baseline (minutes) before any feature effects.
    intercept: f64,
    // Effect of each year of patient age (minutes / year).
    age_coef: f64,
    // Multiplicative-ish offset per gender (minutes).
    gender_coef: HashMap<Gender, f64>,
    // Per-procedure additive offset (minutes). Unknown procedures -> 0.
    procedure_coef: HashMap<String, f64>,
    // Per-surgeon additive offset (minutes). Unknown surgeons -> 0.
    surgeon_coef: HashMap<String, f64>,
    // Gamma noise shape (kept fixed; scale is derived from the mean so the
    // coefficient of variation stays roughly constant across cases).
    noise_shape: f64,
}

impl FeatureAwareEstimator {
    pub fn new(seed: u64) -> Self {
        let mut gender_coef = HashMap::new();
        gender_coef.insert(Gender::Male, -5.0);
        gender_coef.insert(Gender::Female, 5.0);
        gender_coef.insert(Gender::Other, 0.0);

        // Illustrative procedure effects (minutes). Replace with fitted values.
        let mut procedure_coef = HashMap::new();
        procedure_coef.insert("ProcA".to_string(), -20.0);
        procedure_coef.insert("ProcB".to_string(), 0.0);
        procedure_coef.insert("ProcC".to_string(), 25.0);
        procedure_coef.insert("ProcD".to_string(), 40.0);
        procedure_coef.insert("ProcE".to_string(), 75.0);

        // Illustrative surgeon effects (minutes). Replace with fitted values.
        let mut surgeon_coef = HashMap::new();
        surgeon_coef.insert("Dr. A".to_string(), -10.0);
        surgeon_coef.insert("Dr. B".to_string(), 0.0);
        surgeon_coef.insert("Dr. C".to_string(), 15.0);

        FeatureAwareEstimator {
            rng: StdRng::seed_from_u64(seed),
            intercept: 55.0,
            age_coef: 0.4,
            gender_coef,
            procedure_coef,
            surgeon_coef,
            noise_shape: 4.0,
        }
    }

    /// Deterministic mean duration (minutes) for a given case.
    fn mean_duration(&self, features: &SurgeryFeatures) -> f64 {
        let mut mean = self.intercept + self.age_coef * (features.patient_age as f64);
        mean += *self
            .gender_coef
            .get(&features.patient_gender)
            .unwrap_or(&0.0);
        mean += *self
            .procedure_coef
            .get(&features.procedure_code)
            .unwrap_or(&0.0);
        mean += *self.surgeon_coef.get(&features.surgeon_id).unwrap_or(&0.0);
        // Floor the mean so the Gamma scale stays positive.
        mean.max(5.0)
    }
}

impl DurationEstimator for FeatureAwareEstimator {
    fn sample(&mut self, features: &SurgeryFeatures) -> f64 {
        let mean = self.mean_duration(features);
        // Gamma mean = shape * scale  =>  scale = mean / shape.
        // Using a fixed shape gives a roughly constant coefficient of variation,
        // which matches real OR data better than a fixed scale would.
        let scale = mean / self.noise_shape;
        let gamma = Gamma::new(self.noise_shape, scale).expect("Invalid gamma parameters");
        let sampled = gamma.sample(&mut self.rng);
        // Round to whole minutes, floor at 1 so a case is never zero-length.
        (sampled).max(1.0).round()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn case(proc: &str, surgeon: &str, age: u32, gender: Gender) -> SurgeryFeatures {
        SurgeryFeatures {
            procedure_code: proc.to_string(),
            surgeon_id: surgeon.to_string(),
            patient_age: age,
            patient_gender: gender,
            estimated_start: 0.0,
        }
    }

    #[test]
    fn trait_is_implemented_for_both_estimators() {
        let mut stub = StubGamma::new(2.0, 30.0, 1);
        let mut feat = FeatureAwareEstimator::new(1);
        let c = case("ProcB", "Dr. B", 50, Gender::Female);
        // Both must satisfy the DurationEstimator contract (return > 0).
        assert!(stub.sample(&c) > 0.0);
        assert!(feat.sample(&c) > 0.0);
    }

    #[test]
    fn feature_aware_uses_procedure_not_just_noise() {
        let est = FeatureAwareEstimator::new(42);
        // Same seed, same patient demographics, different procedures.
        let a = case("ProcA", "Dr. B", 50, Gender::Female); // -20 min offset
        let e = case("ProcE", "Dr. B", 50, Gender::Female); // +75 min offset
                                                            // Mean should be strictly greater for the longer procedure.
        let mean_a = est.mean_duration(&a);
        let mean_e = est.mean_duration(&e);
        assert!(
            mean_e > mean_a + 50.0,
            "expected ProcE mean >> ProcA mean ({} vs {})",
            mean_e,
            mean_a
        );
    }

    #[test]
    fn unknown_procedure_and_surgeon_default_to_zero_offset() {
        let est = FeatureAwareEstimator::new(7);
        let known = case("ProcC", "Dr. A", 60, Gender::Male);
        let unknown = case("ZZZ999", "Dr. Nobody", 60, Gender::Male);
        // ProcC contributes +25 minutes and Dr. A contributes -10 minutes.
        assert_eq!(
            est.mean_duration(&known),
            est.mean_duration(&unknown) + 15.0
        );
    }

    #[test]
    fn samples_stay_positive_and_reproducible() {
        let mut est = FeatureAwareEstimator::new(99);
        let c = case("ProcD", "Dr. C", 70, Gender::Male);
        let s1 = est.sample(&c);
        let mut est2 = FeatureAwareEstimator::new(99);
        let s2 = est2.sample(&c);
        assert!(s1 >= 1.0);
        assert_eq!(s1, s2, "same seed must give same first sample");
    }
}
