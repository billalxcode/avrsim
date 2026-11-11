//! Solver benchmark command.

use sim_analysis::{DcConfig, TransientConfig, dc_operating_point, transient};
use sim_core::{Circuit, CircuitBuilder, DeviceKind, NodeId, SimError};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

const TRANSIENT_STEPS: usize = 100;

struct TwoPinSpec {
    name: &'static str,
    kind: DeviceKind,
    parameter: &'static str,
    value: f64,
}

const SOURCE: TwoPinSpec = TwoPinSpec {
    name: "v1",
    kind: DeviceKind::VoltageSource,
    parameter: "voltage",
    value: 5.0,
};

const RESISTOR: TwoPinSpec = TwoPinSpec {
    name: "r1",
    kind: DeviceKind::Resistor,
    parameter: "resistance",
    value: 1000.0,
};

const CAPACITOR: TwoPinSpec = TwoPinSpec {
    name: "c1",
    kind: DeviceKind::Capacitor,
    parameter: "capacitance",
    value: 1e-6,
};

/// Runs the solver benchmark.
pub fn run(args: &[String]) -> Result<(), String> {
    let iterations = parse_iterations(args.first().map(String::as_str))?;
    let circuit = rc_circuit()?;
    let dc_config = DcConfig::default();
    let dc_time = measure(iterations, || dc_operating_point(&circuit, &dc_config))?;
    let transient_config = transient_config();
    let transient_time = measure(iterations, || transient(&circuit, &transient_config))?;
    println!("{}", report("dc", iterations, 1, dc_time));
    println!(
        "{}",
        report("transient", iterations, TRANSIENT_STEPS, transient_time)
    );
    Ok(())
}

fn parse_iterations(raw: Option<&str>) -> Result<usize, String> {
    let value = match raw {
        Some(text) => text
            .parse::<usize>()
            .map_err(|_| "invalid iteration count".to_string())?,
        None => 1000,
    };
    if value == 0 {
        return Err("iterations must be positive".to_string());
    }
    Ok(value)
}

fn measure<F, T>(iterations: usize, mut operation: F) -> Result<Duration, String>
where
    F: FnMut() -> Result<T, SimError>,
{
    let start = Instant::now();
    for _ in 0..iterations {
        operation().map_err(|error| error.to_string())?;
    }
    Ok(start.elapsed())
}

fn report(name: &str, iterations: usize, steps: usize, elapsed: Duration) -> String {
    let seconds = elapsed.as_secs_f64();
    let total_steps = iterations.saturating_mul(steps);
    let per_run = seconds / iterations as f64 * 1e6;
    let per_step = seconds / total_steps as f64 * 1e9;
    format!("{name}: {iterations} runs, {seconds:.3} s, {per_run:.1} us/run, {per_step:.2} ns/step")
}

fn transient_config() -> TransientConfig {
    TransientConfig {
        dt: 1e-6,
        steps: TRANSIENT_STEPS,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    }
}

fn rc_circuit() -> Result<Circuit, String> {
    let mut builder = CircuitBuilder::new();
    let input = builder.node("in").map_err(|error| error.to_string())?;
    let output = builder.node("out").map_err(|error| error.to_string())?;
    let ground = builder.ground();
    add_two_pin(&mut builder, input, ground, &SOURCE)?;
    add_two_pin(&mut builder, input, output, &RESISTOR)?;
    add_two_pin(&mut builder, output, ground, &CAPACITOR)?;
    builder.build().map_err(|error| error.to_string())
}

fn add_two_pin(
    builder: &mut CircuitBuilder,
    first: NodeId,
    second: NodeId,
    spec: &TwoPinSpec,
) -> Result<(), String> {
    let mut parameters = BTreeMap::new();
    parameters.insert(spec.parameter.to_string(), spec.value);
    builder
        .device(
            spec.name,
            spec.kind.clone(),
            vec![first, second],
            parameters,
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{parse_iterations, report};
    use std::time::Duration;

    #[test]
    fn parses_iterations() {
        assert_eq!(parse_iterations(None).expect("default"), 1000);
        assert_eq!(parse_iterations(Some("5")).expect("value"), 5);
    }

    #[test]
    fn rejects_invalid_iterations() {
        assert!(parse_iterations(Some("0")).is_err());
        assert!(parse_iterations(Some("many")).is_err());
    }

    #[test]
    fn report_contains_rates() {
        let text = report("dc", 10, 1, Duration::from_millis(20));
        assert!(text.contains("10 runs"));
        assert!(text.contains("us/run"));
        assert!(text.contains("ns/step"));
    }
}
