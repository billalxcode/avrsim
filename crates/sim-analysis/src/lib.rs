//! DC and transient analysis for the simulator.

#![forbid(unsafe_code)]

mod adaptive;
mod dc;
mod system;
mod transient;

#[cfg(test)]
mod test_support;

pub use adaptive::{AdaptiveConfig, transient_adaptive};
pub use dc::{DcConfig, DcPoint, DcResult, dc_operating_point, dc_sweep};
pub use system::SimulationSystem;
pub use transient::{TransientConfig, TransientSample, transient};
