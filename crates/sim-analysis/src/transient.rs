//! Transient analysis using backward Euler.

use crate::system::SimulationSystem;
use sim_core::{Circuit, NodeId, SimError, Solution, Time};

/// Transient solver configuration.
#[derive(Clone, Debug)]
pub struct TransientConfig {
    /// Time step in seconds.
    pub dt: f64,
    /// Number of steps.
    pub steps: usize,
    /// Minimum conductance added to each node.
    pub gmin: f64,
    /// Use a DC operating point as the initial state.
    pub use_dc_op: bool,
    /// Node voltage overrides applied after the initial state.
    pub initial_voltages: Vec<(NodeId, f64)>,
}

impl Default for TransientConfig {
    fn default() -> Self {
        Self {
            dt: 1e-6,
            steps: 100,
            gmin: 1e-12,
            use_dc_op: true,
            initial_voltages: Vec::new(),
        }
    }
}

/// One transient sample.
#[derive(Clone, Debug)]
pub struct TransientSample {
    /// Simulation time.
    pub time: Time,
    /// Solution at this time.
    pub solution: Solution,
}

/// Runs a zero-initial-state transient simulation.
pub fn transient(
    circuit: &Circuit,
    config: &TransientConfig,
) -> Result<Vec<TransientSample>, SimError> {
    validate_config(config)?;
    let mut system = SimulationSystem::new(circuit)?;
    let mut solution = initial_solution(&mut system, config)?;
    let step = step_time(config.dt)?;
    let mut time = Time::ZERO;
    let mut samples = Vec::with_capacity(config.steps);
    for _ in 0..config.steps {
        solution = advance(&mut system, &solution, config)?;
        time = next_time(time, step)?;
        samples.push(sample(time, &solution));
    }
    Ok(samples)
}

fn initial_solution(
    system: &mut SimulationSystem,
    config: &TransientConfig,
) -> Result<Solution, SimError> {
    let initial = system.zero_solution();
    if !config.use_dc_op {
        return Ok(apply_initial_voltages(initial, &config.initial_voltages));
    }
    let dc_config = crate::DcConfig {
        gmin: config.gmin,
        ..crate::DcConfig::default()
    };
    let solution = crate::dc::solve_dc_system(system, initial, &dc_config)?.solution;
    Ok(apply_initial_voltages(solution, &config.initial_voltages))
}

fn apply_initial_voltages(mut solution: Solution, overrides: &[(NodeId, f64)]) -> Solution {
    for (node, voltage) in overrides {
        if let Some(slot) = solution.node_voltages.get_mut(node.raw() as usize) {
            *slot = *voltage;
        }
    }
    solution
}

fn advance(
    system: &mut SimulationSystem,
    solution: &Solution,
    config: &TransientConfig,
) -> Result<Solution, SimError> {
    let next = system.solve(solution, Some(config.dt), config.gmin)?;
    system.update(&next)?;
    Ok(next)
}

fn next_time(time: Time, step: Time) -> Result<Time, SimError> {
    time.checked_add(step)
        .ok_or_else(|| SimError::Internal("transient time overflow".to_string()))
}

fn sample(time: Time, solution: &Solution) -> TransientSample {
    TransientSample {
        time,
        solution: solution.clone(),
    }
}

fn step_time(dt: f64) -> Result<Time, SimError> {
    Time::from_secs_f64(dt)
        .ok_or_else(|| SimError::InvalidNetlist("transient dt is out of range".to_string()))
}

fn validate_config(config: &TransientConfig) -> Result<(), SimError> {
    if !config.dt.is_finite() || config.dt <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "transient dt must be positive".to_string(),
        ));
    }
    if !config.gmin.is_finite() || config.gmin < 0.0 {
        return Err(SimError::InvalidNetlist(
            "transient gmin must be non-negative".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{TransientConfig, transient};
    use crate::test_support::rc_circuit;

    #[test]
    fn rc_charges_toward_supply() {
        let (circuit, output) = rc_circuit();
        let config = TransientConfig {
            dt: 1e-6,
            steps: 1000,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        let samples = transient(&circuit, &config).expect("transient");
        let last = samples.last().expect("last");
        let voltage = last.solution.voltage(output).expect("output");
        assert!((voltage - 3.16).abs() < 0.15, "voltage={voltage}");
    }

    #[test]
    fn transient_rejects_zero_dt() {
        let (circuit, _) = rc_circuit();
        let config = TransientConfig {
            dt: 0.0,
            steps: 1,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        assert!(transient(&circuit, &config).is_err());
    }

    #[test]
    fn transient_rejects_negative_dt() {
        let (circuit, _) = rc_circuit();
        let config = TransientConfig {
            dt: -1e-6,
            steps: 1,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        assert!(transient(&circuit, &config).is_err());
    }

    #[test]
    fn transient_rejects_negative_gmin() {
        let (circuit, _) = rc_circuit();
        let config = TransientConfig {
            dt: 1e-6,
            steps: 1,
            gmin: -1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        assert!(transient(&circuit, &config).is_err());
    }

    #[test]
    fn transient_time_increases() {
        let (circuit, _) = rc_circuit();
        let config = TransientConfig {
            dt: 1e-6,
            steps: 2,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        let samples = transient(&circuit, &config).expect("transient");
        assert_eq!(samples[0].time.as_ps(), 1_000_000);
        assert_eq!(samples[1].time.as_ps(), 2_000_000);
    }

    #[test]
    fn transient_zero_steps_is_empty() {
        let (circuit, _) = rc_circuit();
        let config = TransientConfig {
            dt: 1e-6,
            steps: 0,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        assert!(transient(&circuit, &config).expect("transient").is_empty());
    }

    #[test]
    fn transient_rejects_huge_dt() {
        let (circuit, _) = rc_circuit();
        let config = TransientConfig {
            dt: 1e20,
            steps: 1,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        assert!(transient(&circuit, &config).is_err());
    }

    #[test]
    fn dc_initial_condition_is_steady() {
        let (circuit, output) = rc_circuit();
        let config = TransientConfig {
            dt: 1e-6,
            steps: 5,
            gmin: 1e-12,
            use_dc_op: true,
            initial_voltages: Vec::new(),
        };
        let samples = transient(&circuit, &config).expect("transient");
        let last = samples.last().expect("last");
        let voltage = last.solution.voltage(output).expect("output");
        assert!((voltage - 5.0).abs() < 1e-6);
    }
}
