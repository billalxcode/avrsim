//! DC and transient analysis for the simulator.

#![forbid(unsafe_code)]

mod ac;
mod adaptive;
mod characterization;
mod corners;
mod dc;
#[cfg(test)]
mod determinism;
mod distortion;
mod noise;
mod sensitivity;
mod system;
mod transient;

#[cfg(test)]
mod test_support;

pub use ac::{AcSolution, ac_analysis, ac_sweep};
pub use adaptive::{AdaptiveConfig, transient_adaptive};
pub use characterization::{SwitchingTiming, measure_switching};
pub use corners::{ProcessCorner, with_process_corner, with_temperature};
pub use dc::{DcConfig, DcPoint, DcResult, dc_operating_point, dc_sweep};
pub use distortion::{DistortionConfig, DistortionResult, distortion_analysis};
pub use noise::{
    BOLTZMANN, DEFAULT_TEMPERATURE, NoiseConfig, NoisePoint, NoiseResult, noise_analysis,
};
pub use sensitivity::{DEFAULT_RELATIVE_STEP, DcSensitivity, dc_sensitivity};
pub use sim_device::IntegrationMethod;
pub use system::SimulationSystem;
pub use transient::{TransientConfig, TransientSample, transient, transient_with_method};
