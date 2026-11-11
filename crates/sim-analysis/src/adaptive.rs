//! Adaptive transient stepping with step-doubling error estimation.

use crate::system::SimulationSystem;
use crate::transient::{TransientConfig, TransientSample, initial_solution};
use sim_core::{Circuit, SimError, Solution, Time};

/// Adaptive transient configuration.
#[derive(Clone, Debug)]
pub struct AdaptiveConfig {
    /// Initial time step in seconds.
    pub initial_dt: f64,
    /// Minimum time step in seconds.
    pub min_dt: f64,
    /// Maximum time step in seconds.
    pub max_dt: f64,
    /// Maximum accepted node voltage error.
    pub tolerance: f64,
    /// Maximum accepted samples.
    pub steps: usize,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            initial_dt: 1e-7,
            min_dt: 1e-12,
            max_dt: 1e-3,
            tolerance: 1e-3,
            steps: 1000,
        }
    }
}

struct Trial {
    accepted: bool,
    solution: Solution,
    next_dt: f64,
}

/// Runs a transient simulation with adaptive time steps.
pub fn transient_adaptive(
    circuit: &Circuit,
    base: &TransientConfig,
    config: &AdaptiveConfig,
) -> Result<Vec<TransientSample>, SimError> {
    validate(base, config)?;
    let mut system = SimulationSystem::new(circuit)?;
    let mut state = AdaptiveState {
        solution: initial_solution(&mut system, base)?,
        dt: config.initial_dt,
        time: Time::ZERO,
        samples: Vec::with_capacity(config.steps),
    };
    system.update(&state.solution)?;
    while state.samples.len() < config.steps {
        step_adaptive(&mut system, &mut state, base, config)?;
    }
    Ok(state.samples)
}

struct AdaptiveState {
    solution: Solution,
    dt: f64,
    time: Time,
    samples: Vec<TransientSample>,
}

fn step_adaptive(
    system: &mut SimulationSystem,
    state: &mut AdaptiveState,
    base: &TransientConfig,
    config: &AdaptiveConfig,
) -> Result<(), SimError> {
    let trial = try_step(system, &state.solution, state.dt, base, config)?;
    if !trial.accepted {
        system.update(&state.solution)?;
        state.dt = trial.next_dt;
        return Ok(());
    }
    accept_adaptive(system, state, trial)
}

fn accept_adaptive(
    system: &mut SimulationSystem,
    state: &mut AdaptiveState,
    trial: Trial,
) -> Result<(), SimError> {
    system.update(&trial.solution)?;
    state.solution = trial.solution;
    state.time = next_time(state.time, state.dt)?;
    state.samples.push(TransientSample {
        time: state.time,
        solution: state.solution.clone(),
    });
    state.dt = trial.next_dt;
    Ok(())
}

fn try_step(
    system: &mut SimulationSystem,
    solution: &Solution,
    dt: f64,
    base: &TransientConfig,
    config: &AdaptiveConfig,
) -> Result<Trial, SimError> {
    let full = system.solve(solution, Some(dt), base.gmin)?;
    let second = refine(system, solution, dt, base)?;
    let error = error_estimate(&full, &second);
    decide(dt, error, second, config)
}

fn refine(
    system: &mut SimulationSystem,
    solution: &Solution,
    dt: f64,
    base: &TransientConfig,
) -> Result<Solution, SimError> {
    let half = dt / 2.0;
    let first = system.solve(solution, Some(half), base.gmin)?;
    system.update(&first)?;
    system.solve(&first, Some(half), base.gmin)
}

fn decide(
    dt: f64,
    error: f64,
    solution: Solution,
    config: &AdaptiveConfig,
) -> Result<Trial, SimError> {
    if error <= config.tolerance {
        return Ok(accepted_trial(dt, error, solution, config));
    }
    reject_trial(dt, solution, config)
}

fn accepted_trial(dt: f64, error: f64, solution: Solution, config: &AdaptiveConfig) -> Trial {
    Trial {
        accepted: true,
        solution,
        next_dt: grow_dt(dt, error, config),
    }
}

fn reject_trial(dt: f64, solution: Solution, config: &AdaptiveConfig) -> Result<Trial, SimError> {
    if dt <= config.min_dt {
        return Err(SimError::ConvergenceFailure { iteration: 0 });
    }
    Ok(Trial {
        accepted: false,
        solution,
        next_dt: (dt / 2.0).max(config.min_dt),
    })
}

fn error_estimate(full: &Solution, refined: &Solution) -> f64 {
    full.node_voltages
        .iter()
        .zip(&refined.node_voltages)
        .map(|(left, right)| (left - right).abs())
        .fold(0.0, f64::max)
}

fn grow_dt(dt: f64, error: f64, config: &AdaptiveConfig) -> f64 {
    let factor = if error <= f64::EPSILON {
        2.0
    } else {
        0.9 * (config.tolerance / error).sqrt()
    };
    (dt * factor.clamp(0.5, 2.0)).min(config.max_dt)
}

fn validate(base: &TransientConfig, config: &AdaptiveConfig) -> Result<(), SimError> {
    validate_dt(config)?;
    validate_limits(config)?;
    if !base.gmin.is_finite() || base.gmin < 0.0 {
        return Err(SimError::InvalidNetlist(
            "adaptive gmin must be non-negative".to_string(),
        ));
    }
    Ok(())
}

fn validate_dt(config: &AdaptiveConfig) -> Result<(), SimError> {
    if !config.initial_dt.is_finite() || config.initial_dt <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "adaptive initial dt must be positive".to_string(),
        ));
    }
    if !config.min_dt.is_finite() || config.min_dt <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "adaptive min dt must be positive".to_string(),
        ));
    }
    if !config.max_dt.is_finite() || config.max_dt < config.initial_dt {
        return Err(SimError::InvalidNetlist(
            "adaptive max dt is invalid".to_string(),
        ));
    }
    Ok(())
}

fn validate_limits(config: &AdaptiveConfig) -> Result<(), SimError> {
    if !config.tolerance.is_finite() || config.tolerance <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "adaptive tolerance must be positive".to_string(),
        ));
    }
    Ok(())
}

fn next_time(time: Time, dt: f64) -> Result<Time, SimError> {
    let step = Time::from_secs_f64(dt)
        .ok_or_else(|| SimError::InvalidNetlist("adaptive dt is out of range".to_string()))?;
    time.checked_add(step)
        .ok_or_else(|| SimError::Internal("adaptive time overflow".to_string()))
}

#[cfg(test)]
mod tests {
    use super::{AdaptiveConfig, transient_adaptive};
    use crate::test_support::{rc_circuit, rc_circuit_no_source};
    use crate::transient::{TransientConfig, TransientSample};
    use sim_core::NodeId;

    fn base() -> TransientConfig {
        TransientConfig {
            dt: 1e-6,
            steps: 1000,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        }
    }

    #[test]
    fn adaptive_transient_charges_rc() {
        let (circuit, output) = rc_circuit();
        let config = AdaptiveConfig {
            initial_dt: 1e-7,
            min_dt: 1e-10,
            max_dt: 1e-5,
            tolerance: 1e-3,
            steps: 2000,
        };
        let samples = transient_adaptive(&circuit, &base(), &config).expect("transient");
        let last = samples.last().expect("last");
        let voltage = last.solution.voltage(output).expect("output");
        let time = last.time.as_ps() as f64 * 1e-12;
        let expected = 5.0 * (1.0 - (-time / 1e-3).exp());
        assert!(
            (voltage - expected).abs() < 0.1,
            "voltage={voltage} expected={expected}"
        );
    }

    #[test]
    fn adaptive_respects_initial_capacitor_voltage() {
        let (circuit, output) = rc_circuit_no_source();
        let mut base_config = base();
        base_config.initial_voltages = vec![(output, 5.0)];
        let config = AdaptiveConfig {
            initial_dt: 1e-4,
            min_dt: 1e-8,
            max_dt: 2e-4,
            tolerance: 1e-3,
            steps: 8,
        };
        let samples = transient_adaptive(&circuit, &base_config, &config).expect("transient");
        let last = samples.last().expect("last");
        let voltage = last.solution.voltage(output).expect("output");
        assert!(voltage > 0.5, "voltage={voltage}");
    }

    #[test]
    fn adaptive_matches_fixed_step_on_rc() {
        let (circuit, output) = rc_circuit();
        let fixed = crate::transient::transient(&circuit, &base()).expect("fixed");
        let config = AdaptiveConfig {
            initial_dt: 1e-5,
            min_dt: 1e-8,
            max_dt: 1e-4,
            tolerance: 1e-4,
            steps: 1000,
        };
        let adaptive = transient_adaptive(&circuit, &base(), &config).expect("adaptive");
        let target = 1e-3;
        let fixed_voltage = voltage_at(&fixed, output, target);
        let adaptive_voltage = voltage_at(&adaptive, output, target);
        assert!(
            (fixed_voltage - adaptive_voltage).abs() < 0.1,
            "fixed={fixed_voltage} adaptive={adaptive_voltage}"
        );
    }

    fn voltage_at(samples: &[TransientSample], node: NodeId, target: f64) -> f64 {
        for pair in samples.windows(2) {
            let first = pair[0].time.as_secs_f64();
            let second = pair[1].time.as_secs_f64();
            if target >= first && target <= second {
                let left = pair[0].solution.voltage(node).expect("left");
                let right = pair[1].solution.voltage(node).expect("right");
                let ratio = (target - first) / (second - first);
                return left + ratio * (right - left);
            }
        }
        samples
            .last()
            .expect("last")
            .solution
            .voltage(node)
            .expect("last voltage")
    }

    #[test]
    fn adaptive_time_is_monotonic() {
        let (circuit, _) = rc_circuit();
        let config = AdaptiveConfig::default();
        let samples = transient_adaptive(&circuit, &base(), &config).expect("transient");
        for pair in samples.windows(2) {
            assert!(pair[1].time.as_ps() > pair[0].time.as_ps());
        }
    }

    #[test]
    fn adaptive_rejects_bad_tolerance() {
        let (circuit, _) = rc_circuit();
        let config = AdaptiveConfig {
            tolerance: 0.0,
            ..AdaptiveConfig::default()
        };
        assert!(transient_adaptive(&circuit, &base(), &config).is_err());
    }

    #[test]
    fn adaptive_zero_steps_is_empty() {
        let (circuit, _) = rc_circuit();
        let config = AdaptiveConfig {
            steps: 0,
            ..AdaptiveConfig::default()
        };
        assert!(
            transient_adaptive(&circuit, &base(), &config)
                .expect("transient")
                .is_empty()
        );
    }
}
