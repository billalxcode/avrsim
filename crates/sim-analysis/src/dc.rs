//! DC operating point and sweep analysis.

use crate::system::SimulationSystem;
use sim_core::tolerance::approx_eq;
use sim_core::{Circuit, SimError, Solution};

/// DC solver configuration.
#[derive(Clone, Debug)]
pub struct DcConfig {
    /// Maximum Newton iterations.
    pub max_iterations: usize,
    /// Relative convergence tolerance.
    pub rel_tol: f64,
    /// Absolute convergence tolerance.
    pub abs_tol: f64,
    /// Minimum conductance added to each node.
    pub gmin: f64,
    /// Starting conductance for gmin stepping.
    pub gmin_start: f64,
    /// Maximum node-voltage change per Newton step.
    pub max_step: f64,
}

impl Default for DcConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            rel_tol: 1e-6,
            abs_tol: 1e-9,
            gmin: 1e-12,
            gmin_start: 1e-3,
            max_step: 0.5,
        }
    }
}

/// A DC operating point result.
#[derive(Clone, Debug)]
pub struct DcResult {
    /// Converged solution.
    pub solution: Solution,
    /// Number of iterations used.
    pub iterations: usize,
}

/// One point in a DC sweep.
#[derive(Clone, Debug)]
pub struct DcPoint {
    /// Sweep value.
    pub sweep: f64,
    /// Solution at this value.
    pub solution: Solution,
}

/// Solves the DC operating point of a circuit.
pub fn dc_operating_point(circuit: &Circuit, config: &DcConfig) -> Result<DcResult, SimError> {
    let mut system = SimulationSystem::new(circuit)?;
    let initial = system.zero_solution();
    solve_dc_system(&mut system, initial, config)
}

fn dc_operating_point_with_initial(
    circuit: &Circuit,
    initial: Solution,
    config: &DcConfig,
) -> Result<DcResult, SimError> {
    let mut system = SimulationSystem::new(circuit)?;
    solve_dc_system(&mut system, initial, config)
}

pub(crate) fn solve_dc_system(
    system: &mut SimulationSystem,
    initial: Solution,
    config: &DcConfig,
) -> Result<DcResult, SimError> {
    solve_with_gmin_stepping(system, initial, config)
}

fn solve_with_gmin_stepping(
    system: &mut SimulationSystem,
    solution: Solution,
    config: &DcConfig,
) -> Result<DcResult, SimError> {
    if system.is_linear() {
        return solve_dc(system, solution, config.gmin, config);
    }
    let mut current = solution;
    let mut gmin = config.gmin_start.max(config.gmin);
    while gmin > config.gmin {
        current = solve_dc(system, current, gmin, config)?.solution;
        gmin = (gmin * 0.1).max(config.gmin);
    }
    solve_dc(system, current, config.gmin, config)
}

/// Sweeps a device parameter and solves the DC operating point at each step.
pub fn dc_sweep(
    circuit: &Circuit,
    device: &str,
    parameter: &str,
    start: f64,
    stop: f64,
    step: f64,
    config: &DcConfig,
) -> Result<Vec<DcPoint>, SimError> {
    let sweep = SweepSpec {
        device,
        parameter,
        start,
        stop,
        step,
    };
    run_sweep(circuit, &sweep, config)
}

struct SweepSpec<'a> {
    device: &'a str,
    parameter: &'a str,
    start: f64,
    stop: f64,
    step: f64,
}

fn run_sweep(
    circuit: &Circuit,
    sweep: &SweepSpec<'_>,
    config: &DcConfig,
) -> Result<Vec<DcPoint>, SimError> {
    validate_sweep(sweep.start, sweep.stop, sweep.step)?;
    collect_sweep_points(circuit, sweep, config)
}

fn collect_sweep_points(
    circuit: &Circuit,
    sweep: &SweepSpec<'_>,
    config: &DcConfig,
) -> Result<Vec<DcPoint>, SimError> {
    let mut points = Vec::new();
    let mut previous = None;
    let mut value = sweep.start;
    while value <= sweep.stop + sweep.step * 1e-9 {
        ensure_sweep_capacity(points.len())?;
        let point = solve_sweep_point(circuit, sweep, value, config, previous.as_ref())?;
        previous = Some(point.solution.clone());
        points.push(point);
        value += sweep.step;
    }
    Ok(points)
}

fn ensure_sweep_capacity(len: usize) -> Result<(), SimError> {
    if len >= 1_000_000 {
        return Err(SimError::InvalidNetlist("sweep limit exceeded".to_string()));
    }
    Ok(())
}

fn solve_sweep_point(
    circuit: &Circuit,
    sweep: &SweepSpec<'_>,
    value: f64,
    config: &DcConfig,
    initial: Option<&Solution>,
) -> Result<DcPoint, SimError> {
    let mut swept = circuit.clone();
    swept.set_device_parameter(sweep.device, sweep.parameter, value)?;
    let result = match initial {
        Some(initial) => dc_operating_point_with_initial(&swept, initial.clone(), config)?,
        None => dc_operating_point(&swept, config)?,
    };
    Ok(DcPoint {
        sweep: value,
        solution: result.solution,
    })
}

fn solve_dc(
    system: &mut SimulationSystem,
    initial: Solution,
    gmin: f64,
    config: &DcConfig,
) -> Result<DcResult, SimError> {
    let mut solution = initial;
    for iteration in 0..config.max_iterations {
        let raw = system.solve(&solution, None, gmin)?;
        let next = damp_solution(&solution, &raw, config.max_step);
        let converged = solution_close(&solution, &next, config);
        solution = next;
        if converged {
            return finish_dc(system, solution, iteration + 1);
        }
    }
    Err(SimError::ConvergenceFailure {
        iteration: config.max_iterations,
    })
}

fn damp_solution(old: &Solution, new: &Solution, max_step: f64) -> Solution {
    let max_delta = old
        .node_voltages
        .iter()
        .zip(&new.node_voltages)
        .map(|(first, second)| (second - first).abs())
        .fold(0.0, f64::max);
    if max_delta <= max_step || max_step <= 0.0 {
        return new.clone();
    }
    let alpha = max_step / max_delta;
    Solution {
        node_voltages: interpolate(&old.node_voltages, &new.node_voltages, alpha),
        branch_currents: interpolate(&old.branch_currents, &new.branch_currents, alpha),
    }
}

fn interpolate(old: &[f64], new: &[f64], alpha: f64) -> Vec<f64> {
    old.iter()
        .zip(new)
        .map(|(first, second)| first + alpha * (second - first))
        .collect()
}

fn finish_dc(
    system: &mut SimulationSystem,
    solution: Solution,
    iterations: usize,
) -> Result<DcResult, SimError> {
    system.update(&solution)?;
    Ok(DcResult {
        solution,
        iterations,
    })
}

fn solution_close(first: &Solution, second: &Solution, config: &DcConfig) -> bool {
    let voltages = first
        .node_voltages
        .iter()
        .zip(&second.node_voltages)
        .all(|(left, right)| approx_eq(*left, *right, config.rel_tol, config.abs_tol));
    let currents = first
        .branch_currents
        .iter()
        .zip(&second.branch_currents)
        .all(|(left, right)| approx_eq(*left, *right, config.rel_tol, config.abs_tol));
    voltages && currents
}

fn validate_sweep(start: f64, stop: f64, step: f64) -> Result<(), SimError> {
    if !start.is_finite() || !stop.is_finite() || !step.is_finite() || step <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "sweep values must be finite and step must be positive".to_string(),
        ));
    }
    if stop < start {
        return Err(SimError::InvalidNetlist(
            "sweep stop must be greater than or equal to start".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{DcConfig, dc_operating_point, dc_sweep};
    use crate::test_support::{
        common_emitter, common_emitter_pnp, common_source_n, common_source_n_biased,
        common_source_p, diode_clamp, inverter, rc_circuit, rl_circuit, voltage_divider,
    };

    #[test]
    fn voltage_divider_is_half_supply() {
        let (circuit, output) = voltage_divider();
        let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
        let voltage = result.solution.voltage(output).expect("output");
        assert!((voltage - 5.0).abs() < 1e-6);
    }

    #[test]
    fn dc_sweep_visits_expected_points() {
        let (circuit, output) = voltage_divider();
        let points = dc_sweep(
            &circuit,
            "v1",
            "voltage",
            0.0,
            10.0,
            5.0,
            &DcConfig::default(),
        )
        .expect("sweep");
        assert_eq!(points.len(), 3);
        let middle = points[1].solution.voltage(output).expect("out");
        assert!((middle - 2.5).abs() < 1e-6);
    }

    #[test]
    fn dc_sweep_rejects_zero_step() {
        let (circuit, _) = voltage_divider();
        let result = dc_sweep(
            &circuit,
            "v1",
            "voltage",
            0.0,
            1.0,
            0.0,
            &DcConfig::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn dc_sweep_rejects_reverse_range() {
        let (circuit, _) = voltage_divider();
        let result = dc_sweep(
            &circuit,
            "v1",
            "voltage",
            5.0,
            1.0,
            1.0,
            &DcConfig::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn dc_sweep_rejects_non_finite_step() {
        let (circuit, _) = voltage_divider();
        let result = dc_sweep(
            &circuit,
            "v1",
            "voltage",
            0.0,
            1.0,
            f64::NAN,
            &DcConfig::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn dc_config_default_is_reasonable() {
        let config = DcConfig::default();
        assert!(config.max_iterations > 0);
        assert!(config.rel_tol > 0.0);
        assert!(config.abs_tol > 0.0);
        assert!(config.gmin > 0.0);
    }

    #[test]
    fn inverter_vtc_switches() {
        let (circuit, _, output) = inverter();
        let points = dc_sweep(
            &circuit,
            "vin",
            "voltage",
            0.0,
            5.0,
            0.5,
            &DcConfig::default(),
        )
        .expect("sweep");
        let first = points.first().expect("first");
        let last = points.last().expect("last");
        assert!(first.solution.voltage(output).expect("high") > 4.0);
        assert!(last.solution.voltage(output).expect("low") < 1.0);
    }

    #[test]
    fn inverter_operating_point_converges() {
        let (circuit, _, _) = inverter();
        let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
        assert!(result.iterations > 0);
    }

    #[test]
    fn capacitor_is_open_at_dc() {
        let (circuit, output) = rc_circuit();
        let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
        let voltage = result.solution.voltage(output).expect("output");
        assert!((voltage - 5.0).abs() < 1e-6);
    }

    #[test]
    fn inductor_is_short_at_dc() {
        let (circuit, output) = rl_circuit();
        let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
        let voltage = result.solution.voltage(output).expect("output");
        assert!(voltage.abs() < 1e-6);
    }

    #[test]
    fn dc_sweep_unknown_device_is_rejected() {
        let (circuit, _) = voltage_divider();
        let result = dc_sweep(
            &circuit,
            "missing",
            "voltage",
            0.0,
            1.0,
            0.5,
            &DcConfig::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn dc_sweep_single_point_has_solution() {
        let (circuit, _) = voltage_divider();
        let points = dc_sweep(
            &circuit,
            "v1",
            "voltage",
            3.0,
            3.0,
            1.0,
            &DcConfig::default(),
        )
        .expect("sweep");
        assert_eq!(points.len(), 1);
        assert!((points[0].sweep - 3.0).abs() < 1e-12);
    }

    #[test]
    fn dc_sweep_tracks_previous_solution() {
        let (circuit, output) = diode_clamp();
        let points = dc_sweep(
            &circuit,
            "v1",
            "voltage",
            0.1,
            2.0,
            0.1,
            &DcConfig::default(),
        )
        .expect("sweep");
        assert_eq!(points.len(), 20);
        let first = points.first().expect("first");
        let last = points.last().expect("last");
        let first = first.solution.voltage(output).expect("first v");
        let last = last.solution.voltage(output).expect("last v");
        assert!(last > first + 0.1, "first={first} last={last}");
    }

    #[test]
    fn diode_clamp_limits_voltage() {
        let (circuit, output) = diode_clamp();
        let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
        let voltage = result.solution.voltage(output).expect("output");
        assert!(voltage > 0.4 && voltage < 0.9, "voltage={voltage}");
    }

    #[test]
    fn diode_clamp_converges_from_large_supply() {
        let (mut circuit, output) = diode_clamp();
        circuit
            .set_device_parameter("v1", "voltage", 20.0)
            .expect("v1");
        let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
        let voltage = result.solution.voltage(output).expect("output");
        assert!(voltage > 0.4 && voltage < 0.95, "voltage={voltage}");
    }

    #[test]
    fn npn_common_emitter_conducts() {
        let (circuit, output) = common_emitter();
        let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
        let voltage = result.solution.voltage(output).expect("output");
        assert!(voltage > 3.0 && voltage < 4.5, "voltage={voltage}");
    }

    #[test]
    fn higher_beta_lowers_collector_voltage() {
        let (mut circuit, output) = common_emitter();
        circuit.set_device_parameter("q1", "bf", 200.0).expect("bf");
        let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
        let voltage = result.solution.voltage(output).expect("output");
        assert!(voltage < 3.0, "voltage={voltage}");
    }

    #[test]
    fn pnp_common_emitter_conducts() {
        let (circuit, output) = common_emitter_pnp();
        let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
        let voltage = result.solution.voltage(output).expect("output");
        assert!(voltage > 0.2 && voltage < 1.5, "voltage={voltage}");
    }

    #[test]
    fn n_jfet_common_source_conducts() {
        let (circuit, output) = common_source_n();
        let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
        let voltage = result.solution.voltage(output).expect("output");
        assert!(voltage > 1.0 && voltage < 1.8, "voltage={voltage}");
    }

    #[test]
    fn negative_gate_bias_raises_jfet_drain_voltage() {
        let (base, base_output) = common_source_n();
        let (biased, biased_output) = common_source_n_biased();
        let base = dc_operating_point(&base, &DcConfig::default()).expect("dc");
        let biased = dc_operating_point(&biased, &DcConfig::default()).expect("dc");
        let base = base.solution.voltage(base_output).expect("base");
        let biased = biased.solution.voltage(biased_output).expect("biased");
        assert!(biased > base + 0.2, "base={base} biased={biased}");
    }

    #[test]
    fn p_jfet_common_source_conducts() {
        let (circuit, output) = common_source_p();
        let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
        let voltage = result.solution.voltage(output).expect("output");
        assert!(voltage > 3.0 && voltage < 4.5, "voltage={voltage}");
    }
}
