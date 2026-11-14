//! Registry-backed analog emulation.

use sim_analysis::{DcConfig, dc_operating_point};
use sim_core::{Circuit, CircuitBuilder, DeviceId, NodeId};
use sim_device::{ComponentRegistry, ComponentRequest};

const SOURCE: &str = "source.independent.dc_voltage_source";
const RESISTOR: &str = "passive.resistor.ideal_resistor";
const SWITCH: &str = "switch.mechanical.spst_switch";
const OPAMP: &str = "analog.opamp.ideal_opamp";

fn add_source(
    registry: &ComponentRegistry,
    builder: &mut CircuitBuilder,
    name: &str,
    positive: NodeId,
    negative: NodeId,
    voltage: f64,
) {
    let request = ComponentRequest::new(SOURCE, name, DeviceId::new(0), vec![positive, negative])
        .with_parameter("voltage", voltage);
    registry.add_to(builder, &request).expect("source");
}

fn add_resistor(
    registry: &ComponentRegistry,
    builder: &mut CircuitBuilder,
    name: &str,
    first: NodeId,
    second: NodeId,
    resistance: f64,
) {
    let request = ComponentRequest::new(RESISTOR, name, DeviceId::new(0), vec![first, second])
        .with_parameter("resistance", resistance);
    registry.add_to(builder, &request).expect("resistor");
}

fn add_switch(
    registry: &ComponentRegistry,
    builder: &mut CircuitBuilder,
    name: &str,
    first: NodeId,
    second: NodeId,
    closed: f64,
) {
    let request = ComponentRequest::new(SWITCH, name, DeviceId::new(0), vec![first, second])
        .with_parameter("closed", closed)
        .with_parameter("ron", 0.01);
    registry.add_to(builder, &request).expect("switch");
}

fn add_opamp(
    registry: &ComponentRegistry,
    builder: &mut CircuitBuilder,
    name: &str,
    input: NodeId,
    output: NodeId,
    supply: NodeId,
    ground: NodeId,
) {
    let pins = vec![input, output, supply, ground, output];
    let request = ComponentRequest::new(OPAMP, name, DeviceId::new(0), pins);
    registry.add_to(builder, &request).expect("opamp");
}

fn dc_voltage(circuit: &Circuit, name: &str) -> f64 {
    let result = dc_operating_point(circuit, &DcConfig::default()).expect("dc");
    let node = circuit.node_by_name(name).expect("node").id();
    result.solution.voltage(node).expect("voltage")
}

fn divider_circuit() -> Circuit {
    let registry = ComponentRegistry::new();
    let mut builder = CircuitBuilder::new();
    let vcc = builder.node("vcc").expect("vcc");
    let out = builder.node("out").expect("out");
    let ground = builder.ground();
    add_source(&registry, &mut builder, "V1", vcc, ground, 5.0);
    add_resistor(&registry, &mut builder, "R1", vcc, out, 1000.0);
    add_resistor(&registry, &mut builder, "R2", out, ground, 1000.0);
    builder.build().expect("circuit")
}

fn switch_circuit(closed: f64) -> Circuit {
    let registry = ComponentRegistry::new();
    let mut builder = CircuitBuilder::new();
    let source = builder.node("source").expect("source");
    let out = builder.node("out").expect("out");
    let ground = builder.ground();
    add_source(&registry, &mut builder, "V1", source, ground, 5.0);
    add_resistor(&registry, &mut builder, "R1", source, out, 1000.0);
    add_resistor(&registry, &mut builder, "R2", out, ground, 1_000_000.0);
    add_switch(&registry, &mut builder, "SW1", out, ground, closed);
    builder.build().expect("circuit")
}

fn follower_circuit() -> Circuit {
    let registry = ComponentRegistry::new();
    let mut builder = CircuitBuilder::new();
    let input = builder.node("input").expect("input");
    let output = builder.node("output").expect("output");
    let supply = builder.node("supply").expect("supply");
    let ground = builder.ground();
    add_source(&registry, &mut builder, "VIN", input, ground, 2.5);
    add_source(&registry, &mut builder, "VCC", supply, ground, 5.0);
    add_opamp(&registry, &mut builder, "U1", input, output, supply, ground);
    builder.build().expect("circuit")
}

#[test]
fn registry_voltage_divider_reaches_half_supply() {
    let circuit = divider_circuit();
    let voltage = dc_voltage(&circuit, "out");
    assert!((voltage - 2.5).abs() < 1e-6, "voltage {voltage}");
}

#[test]
fn registry_ideal_switch_changes_node_voltage() {
    let closed = dc_voltage(&switch_circuit(1.0), "out");
    let open = dc_voltage(&switch_circuit(0.0), "out");
    assert!(closed < 0.1, "closed {closed}");
    assert!(open > 4.9, "open {open}");
}

#[test]
fn registry_opamp_follower_tracks_input() {
    let voltage = dc_voltage(&follower_circuit(), "output");
    assert!((voltage - 2.5).abs() < 1e-3, "voltage {voltage}");
}
