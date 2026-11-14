//! Registry-backed analog emulation.

use sim_analysis::{DcConfig, dc_operating_point};
use sim_core::{CircuitBuilder, DeviceId};
use sim_device::{ComponentRegistry, ComponentRequest};

#[test]
fn registry_voltage_divider_reaches_half_supply() {
    let registry = ComponentRegistry::new();
    let mut builder = CircuitBuilder::new();
    let vcc = builder.node("vcc").expect("vcc");
    let out = builder.node("out").expect("out");
    let ground = builder.ground();
    let source = ComponentRequest::new(
        "source.independent.dc_voltage_source",
        "V1",
        DeviceId::new(0),
        vec![vcc, ground],
    )
    .with_parameter("voltage", 5.0);
    registry.add_to(&mut builder, &source).expect("source");
    let top = ComponentRequest::new(
        "passive.resistor.ideal_resistor",
        "R1",
        DeviceId::new(0),
        vec![vcc, out],
    )
    .with_parameter("resistance", 1000.0);
    registry.add_to(&mut builder, &top).expect("top");
    let bottom = ComponentRequest::new(
        "passive.resistor.ideal_resistor",
        "R2",
        DeviceId::new(0),
        vec![out, ground],
    )
    .with_parameter("resistance", 1000.0);
    registry.add_to(&mut builder, &bottom).expect("bottom");
    let circuit = builder.build().expect("circuit");
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let node = circuit.node_by_name("out").expect("out node").id();
    let voltage = result.solution.voltage(node).expect("voltage");
    assert!((voltage - 2.5).abs() < 1e-6, "voltage {voltage}");
}

#[test]
fn registry_ideal_switch_changes_node_voltage() {
    let closed = switch_voltage(1.0);
    let open = switch_voltage(0.0);
    assert!(closed < 0.1, "closed {closed}");
    assert!(open > 4.9, "open {open}");
}

fn switch_voltage(closed: f64) -> f64 {
    let registry = ComponentRegistry::new();
    let mut builder = CircuitBuilder::new();
    let source_node = builder.node("source").expect("source");
    let out = builder.node("out").expect("out");
    let ground = builder.ground();
    let source = ComponentRequest::new(
        "source.independent.dc_voltage_source",
        "V1",
        DeviceId::new(0),
        vec![source_node, ground],
    )
    .with_parameter("voltage", 5.0);
    registry.add_to(&mut builder, &source).expect("source");
    let series = ComponentRequest::new(
        "passive.resistor.ideal_resistor",
        "R1",
        DeviceId::new(0),
        vec![source_node, out],
    )
    .with_parameter("resistance", 1000.0);
    registry.add_to(&mut builder, &series).expect("series");
    let load = ComponentRequest::new(
        "passive.resistor.ideal_resistor",
        "R2",
        DeviceId::new(0),
        vec![out, ground],
    )
    .with_parameter("resistance", 1_000_000.0);
    registry.add_to(&mut builder, &load).expect("load");
    let switch = ComponentRequest::new(
        "switch.mechanical.spst_switch",
        "SW1",
        DeviceId::new(0),
        vec![out, ground],
    )
    .with_parameter("closed", closed)
    .with_parameter("ron", 0.01);
    registry.add_to(&mut builder, &switch).expect("switch");
    let circuit = builder.build().expect("circuit");
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let node = circuit.node_by_name("out").expect("out node").id();
    result.solution.voltage(node).expect("voltage")
}
