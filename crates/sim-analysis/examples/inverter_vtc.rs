//! Prints the DC transfer curve of a CMOS inverter.

use sim_analysis::{DcConfig, dc_sweep};
use sim_core::{Circuit, CircuitBuilder, DeviceKind, NodeId};
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (circuit, output) = inverter()?;
    let points = dc_sweep(
        &circuit,
        "vin",
        "voltage",
        0.0,
        5.0,
        0.25,
        &DcConfig::default(),
    )?;
    for point in points {
        let voltage = point.solution.voltage(output).unwrap_or(0.0);
        println!("vin={:.2} vout={:.4}", point.sweep, voltage);
    }
    Ok(())
}

fn inverter() -> Result<(Circuit, NodeId), Box<dyn std::error::Error>> {
    let mut builder = CircuitBuilder::new();
    let vdd = builder.supply("vdd")?;
    let input = builder.node("input")?;
    let output = builder.node("output")?;
    let ground = builder.ground();
    build_inverter(&mut builder, vdd, input, output, ground)?;
    Ok((builder.build()?, output))
}

fn build_inverter(
    builder: &mut CircuitBuilder,
    vdd: NodeId,
    input: NodeId,
    output: NodeId,
    ground: NodeId,
) -> Result<(), Box<dyn std::error::Error>> {
    add_source(builder, "vdd_src", vdd, ground, 5.0)?;
    add_source(builder, "vin", input, ground, 0.0)?;
    add_mosfet(builder, "mp", DeviceKind::Pmos, input, output, vdd, 10e-6)?;
    add_mosfet(builder, "mn", DeviceKind::Nmos, input, output, ground, 5e-6)
}

fn add_source(
    builder: &mut CircuitBuilder,
    name: &str,
    positive: NodeId,
    negative: NodeId,
    voltage: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut parameters = BTreeMap::new();
    parameters.insert("voltage".to_string(), voltage);
    builder.device(
        name,
        DeviceKind::VoltageSource,
        vec![positive, negative],
        parameters,
    )?;
    Ok(())
}

fn add_mosfet(
    builder: &mut CircuitBuilder,
    name: &str,
    kind: DeviceKind,
    gate: NodeId,
    drain: NodeId,
    source: NodeId,
    width: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut parameters = BTreeMap::new();
    parameters.insert("vto".to_string(), 0.7);
    parameters.insert("kp".to_string(), 1e-3);
    parameters.insert("lambda".to_string(), 0.02);
    parameters.insert("w".to_string(), width);
    parameters.insert("l".to_string(), 1e-6);
    builder.device(name, kind, vec![gate, drain, source], parameters)?;
    Ok(())
}
