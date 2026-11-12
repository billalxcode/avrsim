//! Transient analysis using backward Euler.

use crate::newton::{NewtonConfig, damp_solution, max_node_delta, solution_close};
use crate::system::SimulationSystem;
use sim_core::{Circuit, NodeId, SimError, Solution, Time};
use sim_device::IntegrationMethod;

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
    /// Maximum node-voltage change at convergence.
    pub residual: f64,
    /// Newton iterations used for this step.
    pub iterations: usize,
}

/// Runs a zero-initial-state transient simulation.
pub fn transient(
    circuit: &Circuit,
    config: &TransientConfig,
) -> Result<Vec<TransientSample>, SimError> {
    transient_with_method(circuit, config, IntegrationMethod::BackwardEuler)
}

/// Runs a transient simulation with an explicit integration method.
pub fn transient_with_method(
    circuit: &Circuit,
    config: &TransientConfig,
    method: IntegrationMethod,
) -> Result<Vec<TransientSample>, SimError> {
    transient_with_solver(circuit, config, method, &NewtonConfig::default())
}

/// Runs a transient simulation with an explicit integration method and Newton settings.
pub fn transient_with_solver(
    circuit: &Circuit,
    config: &TransientConfig,
    method: IntegrationMethod,
    newton: &NewtonConfig,
) -> Result<Vec<TransientSample>, SimError> {
    validate_config(config)?;
    validate_newton(newton)?;
    let mut system = SimulationSystem::new(circuit)?;
    let solution = initial_solution(&mut system, config)?;
    system.update(&solution)?;
    run_steps(&mut system, solution, config, method, newton)
}

pub(crate) fn initial_solution(
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

fn run_steps(
    system: &mut SimulationSystem,
    solution: Solution,
    config: &TransientConfig,
    method: IntegrationMethod,
    newton: &NewtonConfig,
) -> Result<Vec<TransientSample>, SimError> {
    let step = step_time(config.dt)?;
    let mut time = Time::ZERO;
    let mut samples = Vec::with_capacity(config.steps);
    let mut current = solution;
    for _ in 0..config.steps {
        let (next, residual, iterations) = step_sample(system, &current, config, method, newton)?;
        time = next_time(time, step)?;
        samples.push(sample(time, &next, residual, iterations));
        current = next;
    }
    Ok(samples)
}

fn step_sample(
    system: &mut SimulationSystem,
    current: &Solution,
    config: &TransientConfig,
    method: IntegrationMethod,
    newton: &NewtonConfig,
) -> Result<(Solution, f64, usize), SimError> {
    let (next, residual, iterations) =
        solve_step(system, current, config.dt, config.gmin, method, newton)?;
    system.update(&next)?;
    Ok((next, residual, iterations))
}

pub(crate) fn solve_step(
    system: &mut SimulationSystem,
    solution: &Solution,
    dt: f64,
    gmin: f64,
    method: IntegrationMethod,
    newton: &NewtonConfig,
) -> Result<(Solution, f64, usize), SimError> {
    let mut current = solution.clone();
    for iteration in 0..newton.max_iterations {
        let (next, residual, converged) =
            newton_iteration(system, &current, dt, gmin, method, newton)?;
        current = next;
        if converged {
            return Ok((current, residual, iteration + 1));
        }
    }
    Err(convergence_failure(newton))
}

fn convergence_failure(newton: &NewtonConfig) -> SimError {
    SimError::ConvergenceFailure {
        iteration: newton.max_iterations,
    }
}

fn newton_iteration(
    system: &mut SimulationSystem,
    current: &Solution,
    dt: f64,
    gmin: f64,
    method: IntegrationMethod,
    newton: &NewtonConfig,
) -> Result<(Solution, f64, bool), SimError> {
    let raw = system.solve_with_method(current, Some(dt), gmin, method)?;
    let next = damp_solution(current, &raw, newton.max_step);
    let residual = max_node_delta(current, &next);
    let converged = solution_close(current, &next, newton.rel_tol, newton.abs_tol);
    Ok((next, residual, converged))
}

pub(crate) fn validate_newton(config: &NewtonConfig) -> Result<(), SimError> {
    if config.max_iterations == 0 {
        return Err(SimError::InvalidNetlist(
            "transient max iterations must be positive".to_string(),
        ));
    }
    validate_tolerances(config)?;
    validate_step(config)
}

fn validate_tolerances(config: &NewtonConfig) -> Result<(), SimError> {
    if !config.rel_tol.is_finite() || config.rel_tol <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "transient rel_tol must be positive".to_string(),
        ));
    }
    if !config.abs_tol.is_finite() || config.abs_tol <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "transient abs_tol must be positive".to_string(),
        ));
    }
    Ok(())
}

fn validate_step(config: &NewtonConfig) -> Result<(), SimError> {
    if !config.max_step.is_finite() || config.max_step < 0.0 {
        return Err(SimError::InvalidNetlist(
            "transient max_step must be non-negative".to_string(),
        ));
    }
    Ok(())
}

fn next_time(time: Time, step: Time) -> Result<Time, SimError> {
    time.checked_add(step)
        .ok_or_else(|| SimError::Internal("transient time overflow".to_string()))
}

fn sample(time: Time, solution: &Solution, residual: f64, iterations: usize) -> TransientSample {
    TransientSample {
        time,
        solution: solution.clone(),
        residual,
        iterations,
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
    use super::{
        TransientConfig, TransientSample, transient, transient_with_method, transient_with_solver,
    };
    use crate::newton::NewtonConfig;
    use crate::test_support::{
        common_emitter, diode_rectifier, matched_line, mos_capacitor, pulse_divider, rc_circuit,
        rc_circuit_no_source, reverse_diode_rc, rl_circuit, switching_inverter,
    };
    use sim_core::NodeId;
    use sim_device::IntegrationMethod;

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

    #[test]
    fn trapezoidal_rc_charges_toward_supply() {
        let (circuit, output) = rc_circuit();
        let config = TransientConfig {
            dt: 1e-6,
            steps: 1000,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        let samples = transient_with_method(&circuit, &config, IntegrationMethod::Trapezoidal)
            .expect("transient");
        let voltage = samples
            .last()
            .expect("last")
            .solution
            .voltage(output)
            .expect("v");
        assert!((voltage - 3.16).abs() < 0.15, "voltage={voltage}");
    }

    #[test]
    fn gear2_rc_charges_toward_supply() {
        let (circuit, output) = rc_circuit();
        let config = TransientConfig {
            dt: 1e-6,
            steps: 1000,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        let samples =
            transient_with_method(&circuit, &config, IntegrationMethod::Gear2).expect("transient");
        let voltage = samples
            .last()
            .expect("last")
            .solution
            .voltage(output)
            .expect("v");
        assert!((voltage - 3.16).abs() < 0.15, "voltage={voltage}");
    }

    #[test]
    fn trapezoidal_rl_voltage_decays() {
        let (circuit, output) = rl_circuit();
        let config = TransientConfig {
            dt: 1e-7,
            steps: 1000,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        let samples = transient_with_method(&circuit, &config, IntegrationMethod::Trapezoidal)
            .expect("transient");
        let voltage = samples
            .last()
            .expect("last")
            .solution
            .voltage(output)
            .expect("v");
        assert!(voltage.abs() < 0.05, "voltage={voltage}");
    }

    #[test]
    fn gear2_rl_voltage_decays() {
        let (circuit, output) = rl_circuit();
        let config = TransientConfig {
            dt: 1e-7,
            steps: 1000,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        let samples =
            transient_with_method(&circuit, &config, IntegrationMethod::Gear2).expect("transient");
        let voltage = samples
            .last()
            .expect("last")
            .solution
            .voltage(output)
            .expect("v");
        assert!(voltage.abs() < 0.05, "voltage={voltage}");
    }

    #[test]
    fn transmission_line_delays_matched_step() {
        let (circuit, output) = matched_line();
        let config = TransientConfig {
            dt: 1e-7,
            steps: 20,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        let samples = transient(&circuit, &config).expect("transient");
        let early = samples[8].solution.voltage(output).expect("early");
        let late = samples[14].solution.voltage(output).expect("late");
        assert!(early.abs() < 0.01, "early={early}");
        assert!((late - 2.5).abs() < 0.1, "late={late}");
    }

    #[test]
    fn rc_discharge_decays_from_initial_voltage() {
        let (circuit, output) = rc_circuit_no_source();
        let config = TransientConfig {
            dt: 1e-4,
            steps: 100,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: vec![(output, 5.0)],
        };
        let samples = transient(&circuit, &config).expect("transient");
        let early = samples[0].solution.voltage(output).expect("early");
        let last = samples.last().expect("last");
        let last = last.solution.voltage(output).expect("last");
        assert!(early > 4.0, "early={early}");
        assert!(last.abs() < 0.01, "last={last}");
    }

    #[test]
    fn pulse_source_switches_transient_output() {
        let (circuit, output) = pulse_divider();
        let config = TransientConfig {
            dt: 0.25e-6,
            steps: 8,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        let samples = transient(&circuit, &config).expect("transient");
        let high = samples[0].solution.voltage(output).expect("high");
        let low = samples[5].solution.voltage(output).expect("low");
        let again = samples[7].solution.voltage(output).expect("again");
        assert!((high - 2.5).abs() < 1e-6, "high={high}");
        assert!(low.abs() < 1e-6, "low={low}");
        assert!((again - 2.5).abs() < 1e-6, "again={again}");
    }

    #[test]
    fn mosfet_gate_capacitance_charges() {
        let (circuit, output) = mos_capacitor();
        let config = TransientConfig {
            dt: 1e-7,
            steps: 20,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        let samples = transient(&circuit, &config).expect("transient");
        let gate = samples.last().expect("last");
        let gate = gate.solution.voltage(output).expect("gate");
        assert!(gate > 2.8 && gate < 3.3, "gate={gate}");
    }

    #[test]
    fn diode_junction_capacitance_slows_reverse_edge() {
        let config = TransientConfig {
            dt: 1e-7,
            steps: 10,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        let (without, without_output) = reverse_diode_rc(0.0);
        let (with, with_output) = reverse_diode_rc(5e-9);
        let without = transient(&without, &config).expect("without");
        let with = transient(&with, &config).expect("with");
        let without = without.last().expect("last");
        let with = with.last().expect("last");
        let without = without.solution.voltage(without_output).expect("v0");
        let with = with.solution.voltage(with_output).expect("v1");
        assert!(without < -1.9, "without={without}");
        assert!(with > -1.5, "with={with}");
    }

    #[test]
    fn diode_rectifier_transient_reaches_forward_drop() {
        let (circuit, output) = diode_rectifier();
        let config = TransientConfig {
            dt: 1e-7,
            steps: 50,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        let samples = transient(&circuit, &config).expect("transient");
        let voltage = samples
            .last()
            .expect("last")
            .solution
            .voltage(output)
            .expect("v");
        assert!(voltage > 0.4 && voltage < 0.9, "voltage={voltage}");
    }

    #[test]
    fn switching_inverter_transient_crosses_output() {
        let (circuit, output) = switching_inverter();
        let config = TransientConfig {
            dt: 1e-9,
            steps: 4000,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        let samples = transient(&circuit, &config).expect("transient");
        assert!(count_crossings(&samples, output) >= 2);
    }

    #[test]
    fn bjt_common_emitter_transient_reaches_bias() {
        let (circuit, output) = common_emitter();
        let config = TransientConfig {
            dt: 1e-7,
            steps: 20,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        let samples = transient(&circuit, &config).expect("transient");
        let voltage = samples
            .last()
            .expect("last")
            .solution
            .voltage(output)
            .expect("v");
        assert!(voltage > 3.0 && voltage < 4.5, "voltage={voltage}");
    }

    #[test]
    fn transient_rejects_invalid_newton_config() {
        let (circuit, _) = rc_circuit();
        let newton = NewtonConfig {
            max_iterations: 0,
            ..NewtonConfig::default()
        };
        let method = IntegrationMethod::BackwardEuler;
        let config = TransientConfig::default();
        assert!(transient_with_solver(&circuit, &config, method, &newton).is_err());
    }

    fn count_crossings(samples: &[TransientSample], node: NodeId) -> usize {
        let mut previous = samples[0].solution.voltage(node).expect("first") > 2.5;
        let mut crossings = 0;
        for sample in samples.iter().skip(1) {
            let high = sample.solution.voltage(node).expect("voltage") > 2.5;
            if high != previous {
                crossings += 1;
                previous = high;
            }
        }
        crossings
    }
}
