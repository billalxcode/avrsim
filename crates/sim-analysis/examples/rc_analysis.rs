//! Demonstrates DC, transient, and AC analysis on an RC low-pass.

use sim_analysis::{DcConfig, TransientConfig, ac_analysis, dc_operating_point, transient};
use sim_core::{Circuit, CircuitBuilder, DeviceKind, NodeId, Solution};
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut circuit, output) = rc_circuit()?;
    show_dc(&circuit, output)?;
    show_transient(&circuit, output)?;
    circuit.set_device_parameter("v1", "ac", 1.0)?;
    show_ac(&circuit, output)?;
    Ok(())
}

fn show_dc(circuit: &Circuit, output: NodeId) -> Result<(), Box<dyn std::error::Error>> {
    let dc = dc_operating_point(circuit, &DcConfig::default())?;
    println!("dc vout={:.4}", voltage(&dc.solution, output));
    Ok(())
}

fn show_transient(circuit: &Circuit, output: NodeId) -> Result<(), Box<dyn std::error::Error>> {
    let samples = transient(circuit, &transient_config())?;
    let last = samples.last().ok_or("no transient samples")?;
    println!(
        "transient t={} ps vout={:.4}",
        last.time.as_ps(),
        voltage(&last.solution, output)
    );
    Ok(())
}

fn show_ac(circuit: &Circuit, output: NodeId) -> Result<(), Box<dyn std::error::Error>> {
    let cutoff = 1.0 / (2.0 * std::f64::consts::PI * 1000.0 * 1e-6);
    let ac = ac_analysis(circuit, cutoff)?;
    println!(
        "ac magnitude={:.4} phase={:.1} deg",
        ac.magnitude(output).unwrap_or(0.0),
        ac.phase_degrees(output).unwrap_or(0.0)
    );
    Ok(())
}

fn rc_circuit() -> Result<(Circuit, NodeId), Box<dyn std::error::Error>> {
    let mut builder = CircuitBuilder::new();
    let input = builder.node("in")?;
    let output = builder.node("out")?;
    let ground = builder.ground();
    add_source(&mut builder, input, ground)?;
    add_resistor(&mut builder, input, output)?;
    add_capacitor(&mut builder, output, ground)?;
    Ok((builder.build()?, output))
}

fn add_source(
    builder: &mut CircuitBuilder,
    positive: NodeId,
    negative: NodeId,
) -> Result<(), Box<dyn std::error::Error>> {
    add(
        builder,
        "v1",
        DeviceKind::VoltageSource,
        [positive, negative],
        "voltage",
        5.0,
    )
}

fn add_resistor(
    builder: &mut CircuitBuilder,
    first: NodeId,
    second: NodeId,
) -> Result<(), Box<dyn std::error::Error>> {
    add(
        builder,
        "r1",
        DeviceKind::Resistor,
        [first, second],
        "resistance",
        1000.0,
    )
}

fn add_capacitor(
    builder: &mut CircuitBuilder,
    first: NodeId,
    second: NodeId,
) -> Result<(), Box<dyn std::error::Error>> {
    add(
        builder,
        "c1",
        DeviceKind::Capacitor,
        [first, second],
        "capacitance",
        1e-6,
    )
}

fn add(
    builder: &mut CircuitBuilder,
    name: &str,
    kind: DeviceKind,
    pins: [NodeId; 2],
    parameter: &str,
    value: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut parameters = BTreeMap::new();
    parameters.insert(parameter.to_string(), value);
    builder.device(name, kind, pins.to_vec(), parameters)?;
    Ok(())
}

fn transient_config() -> TransientConfig {
    TransientConfig {
        dt: 1e-6,
        steps: 100,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    }
}

fn voltage(solution: &Solution, node: NodeId) -> f64 {
    solution.voltage(node).unwrap_or(0.0)
}
