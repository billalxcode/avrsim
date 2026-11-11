use super::{Circuit, CircuitBuilder, Device, DeviceKind, Subcircuit};
use crate::NodeId;
use crate::error::SimError;
use std::collections::BTreeMap;

fn params() -> BTreeMap<String, f64> {
    BTreeMap::new()
}

fn inverter_template() -> Circuit {
    let mut builder = CircuitBuilder::new();
    let vdd = builder.node("vdd").expect("vdd");
    let input = builder.node("input").expect("input");
    let output = builder.node("output").expect("output");
    let ground = builder.ground();
    add_inverter_devices(&mut builder, input, output, vdd, ground);
    builder.build().expect("template")
}

fn add_inverter_devices(
    builder: &mut CircuitBuilder,
    input: NodeId,
    output: NodeId,
    vdd: NodeId,
    ground: NodeId,
) {
    builder
        .device("mp", DeviceKind::Pmos, vec![input, output, vdd], params())
        .expect("mp");
    builder
        .device(
            "mn",
            DeviceKind::Nmos,
            vec![input, output, ground],
            params(),
        )
        .expect("mn");
}

fn inverter_subcircuit() -> Subcircuit {
    let template = inverter_template();
    let input = template.node_by_name("input").expect("input").id();
    let output = template.node_by_name("output").expect("output").id();
    let vdd = template.node_by_name("vdd").expect("vdd").id();
    let ports = vec![
        ("input".to_string(), input),
        ("output".to_string(), output),
        ("vdd".to_string(), vdd),
    ];
    Subcircuit::new("inverter", template, ports).expect("subcircuit")
}

fn divider_subcircuit() -> Subcircuit {
    let mut builder = CircuitBuilder::new();
    let input = builder.node("input").expect("input");
    let mid = builder.node("mid").expect("mid");
    let output = builder.node("output").expect("output");
    let ground = builder.ground();
    add_divider_devices(&mut builder, input, mid, output, ground);
    let template = builder.build().expect("template");
    let ports = vec![("input".to_string(), input), ("output".to_string(), output)];
    Subcircuit::new("divider", template, ports).expect("subcircuit")
}

fn add_divider_devices(
    builder: &mut CircuitBuilder,
    input: NodeId,
    mid: NodeId,
    output: NodeId,
    ground: NodeId,
) {
    builder
        .device("r1", DeviceKind::Resistor, vec![input, mid], params())
        .expect("r1");
    builder
        .device("r2", DeviceKind::Resistor, vec![mid, output], params())
        .expect("r2");
    builder
        .device("r3", DeviceKind::Resistor, vec![output, ground], params())
        .expect("r3");
}

#[test]
fn builder_creates_inverter() {
    let circuit = inverter_template();
    assert_eq!(circuit.ground(), NodeId::GROUND);
    assert_eq!(circuit.nodes().len(), 4);
    assert_eq!(circuit.devices().len(), 2);
    assert!(circuit.node_by_name("input").is_some());
    assert!(circuit.device_by_name("mp").is_some());
}

#[test]
fn macro_matches_builder() -> Result<(), SimError> {
    let manual = inverter_template();
    let generated = crate::circuit! {
        nodes { vdd, input, output }
        ground gnd;
        devices {
            mp: DeviceKind::Pmos => [input, output, vdd];
            mn: DeviceKind::Nmos => [input, output, gnd];
        }
    }?;
    assert_eq!(manual, generated);
    Ok(())
}

#[test]
fn duplicate_node_name_is_rejected() {
    let mut builder = CircuitBuilder::new();
    builder.node("a").expect("first node");
    let result = builder.node("a");
    assert!(matches!(result, Err(SimError::InvalidNetlist(_))));
}

#[test]
fn duplicate_device_name_is_rejected() {
    let mut builder = CircuitBuilder::new();
    let a = builder.node("a").expect("a");
    let b = builder.node("b").expect("b");
    let pins = vec![a, b];
    builder
        .device("r1", DeviceKind::Resistor, pins.clone(), params())
        .expect("first device");
    let result = builder.device("r1", DeviceKind::Resistor, pins, params());
    assert!(matches!(result, Err(SimError::InvalidNetlist(_))));
}

#[test]
fn floating_node_is_rejected() {
    let mut builder = CircuitBuilder::new();
    builder.node("unused").expect("node");
    let result = builder.build();
    assert!(matches!(result, Err(SimError::FloatingNode(_))));
}

#[test]
fn dc_path_is_required() {
    let mut builder = CircuitBuilder::new();
    let node = builder.node("ac_only").expect("node");
    let ground = builder.ground();
    builder
        .device("c1", DeviceKind::Capacitor, vec![node, ground], params())
        .expect("capacitor");
    let result = builder.build();
    assert!(matches!(result, Err(SimError::NoDcPath { .. })));
}

#[test]
fn subcircuit_instantiates_with_prefix() {
    let subcircuit = inverter_subcircuit();
    let mut top = CircuitBuilder::new();
    let input = top.node("a").expect("a");
    let output = top.node("y").expect("y");
    let vdd = top.supply("vdd").expect("vdd");
    let connections = [("input", input), ("output", output), ("vdd", vdd)];
    top.instantiate("u1", &subcircuit, &connections)
        .expect("instantiate");
    let circuit = top.build().expect("top");
    assert!(circuit.device_by_name("u1.mp").is_some());
    assert!(circuit.device_by_name("u1.mn").is_some());
    assert_eq!(circuit.node_by_name("a").expect("a").id(), input);
}

#[test]
fn subcircuit_internal_node_is_prefixed() {
    let subcircuit = divider_subcircuit();
    let mut top = CircuitBuilder::new();
    let input = top.node("a").expect("a");
    let output = top.node("y").expect("y");
    let connections = [("input", input), ("output", output)];
    top.instantiate("u1", &subcircuit, &connections)
        .expect("instantiate");
    let circuit = top.build().expect("top");
    assert!(circuit.node_by_name("u1.mid").is_some());
    assert!(circuit.device_by_name("u1.r1").is_some());
}

#[test]
fn subcircuit_missing_port_is_rejected() {
    let subcircuit = inverter_subcircuit();
    let mut top = CircuitBuilder::new();
    let input = top.node("a").expect("a");
    let output = top.node("y").expect("y");
    let connections = [("input", input), ("output", output)];
    let result = top.instantiate("u1", &subcircuit, &connections);
    assert!(matches!(result, Err(SimError::InvalidNetlist(_))));
}

#[test]
fn macro_supports_params() -> Result<(), SimError> {
    let circuit = crate::circuit! {
        nodes { a, b }
        ground gnd;
        devices {
            r1: DeviceKind::Resistor => [a, b], params { resistance: 1000.0 };
            r2: DeviceKind::Resistor => [b, gnd];
        }
    }?;
    let resistor = circuit.device_by_name("r1").expect("r1");
    assert_eq!(resistor.parameters().get("resistance"), Some(&1000.0));
    Ok(())
}

#[test]
fn snapshot_matches_circuit() {
    let circuit = inverter_template();
    let snapshot = circuit.snapshot();
    assert_eq!(circuit, snapshot);
}

#[test]
fn ground_is_zero() {
    let builder = CircuitBuilder::new();
    assert_eq!(builder.ground(), NodeId::GROUND);
    assert_eq!(NodeId::GROUND.raw(), 0);
}

#[test]
fn device_can_be_added_after_build() {
    let mut circuit = inverter_template();
    let output = circuit.node_by_name("output").expect("output").id();
    let ground = circuit.ground();
    circuit
        .add_device(
            "rload",
            DeviceKind::Resistor,
            vec![output, ground],
            params(),
        )
        .expect("add device");
    assert!(circuit.device_by_name("rload").is_some());
}

#[test]
fn added_device_rejects_unknown_node() {
    let mut circuit = inverter_template();
    let ground = circuit.ground();
    let result = circuit.add_device(
        "bad",
        DeviceKind::Resistor,
        vec![NodeId::new(99), ground],
        params(),
    );
    assert!(result.is_err());
}
