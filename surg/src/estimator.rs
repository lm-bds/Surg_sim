use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Gamma};

/// Gender of a patient (extendable if needed)
#[derive(Debug, Clone)]
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
