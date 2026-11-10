use sim_core::{Circuit, CircuitBuilder, DeviceKind, MnaContext, NodeId, Solution};
use sim_device::model_from_instance;
use std::collections::BTreeMap;

fn params(name: &str, value: f64) -> BTreeMap<String, f64> {
    let mut map = BTreeMap::new();
    map.insert(name.to_string(), value);
    map
}

fn add_two_pin(
    builder: &mut CircuitBuilder,
    name: &str,
    kind: DeviceKind,
    pins: [NodeId; 2],
    parameters: BTreeMap<String, f64>,
) {
    builder
        .device(name, kind, vec![pins[0], pins[1]], parameters)
        .expect(name);
}

fn add_mosfet(
    builder: &mut CircuitBuilder,
    name: &str,
    kind: DeviceKind,
    pins: [NodeId; 3],
    parameters: BTreeMap<String, f64>,
) {
    builder
        .device(name, kind, vec![pins[0], pins[1], pins[2]], parameters)
        .expect(name);
}

fn resistor_circuit() -> (Circuit, NodeId) {
    let mut builder = CircuitBuilder::new();
    let node = builder.node("n").expect("node");
    let ground = builder.ground();
    let pins = [node, ground];
    add_two_pin(
        &mut builder,
        "r1",
        DeviceKind::Resistor,
        pins,
        params("resistance", 1000.0),
    );
    (builder.build().expect("circuit"), node)
}

fn voltage_source_circuit() -> (Circuit, NodeId) {
    let mut builder = CircuitBuilder::new();
    let node = builder.node("n").expect("node");
    let ground = builder.ground();
    let pins = [node, ground];
    add_two_pin(
        &mut builder,
        "v1",
        DeviceKind::VoltageSource,
        pins,
        params("voltage", 5.0),
    );
    (builder.build().expect("circuit"), node)
}

fn current_source_circuit() -> (Circuit, NodeId) {
    let mut builder = CircuitBuilder::new();
    let node = builder.node("n").expect("node");
    let ground = builder.ground();
    add_two_pin(
        &mut builder,
        "r1",
        DeviceKind::Resistor,
        [node, ground],
        params("resistance", 1000.0),
    );
    add_two_pin(
        &mut builder,
        "i1",
        DeviceKind::CurrentSource,
        [ground, node],
        params("current", 1e-3),
    );
    (builder.build().expect("circuit"), node)
}

fn mosfet_circuit() -> Circuit {
    let mut builder = CircuitBuilder::new();
    let vdd = builder.supply("vdd").expect("vdd");
    let gate = builder.node("gate").expect("gate");
    let drain = builder.node("drain").expect("drain");
    let ground = builder.ground();
    add_mosfet_loads(&mut builder, vdd, gate, drain, ground);
    builder.build().expect("circuit")
}

fn add_mosfet_loads(
    builder: &mut CircuitBuilder,
    vdd: NodeId,
    gate: NodeId,
    drain: NodeId,
    ground: NodeId,
) {
    add_gate_load(builder, gate, ground);
    add_drain_load(builder, vdd, drain);
    add_mosfet(
        builder,
        "mn",
        DeviceKind::Nmos,
        [gate, drain, ground],
        params("kp", 1e-3),
    );
}

fn add_gate_load(builder: &mut CircuitBuilder, gate: NodeId, ground: NodeId) {
    add_two_pin(
        builder,
        "rg",
        DeviceKind::Resistor,
        [gate, ground],
        params("resistance", 1000.0),
    );
}

fn add_drain_load(builder: &mut CircuitBuilder, vdd: NodeId, drain: NodeId) {
    add_two_pin(
        builder,
        "rd",
        DeviceKind::Resistor,
        [vdd, drain],
        params("resistance", 1000.0),
    );
}

#[test]
fn resistor_stamps_conductance() {
    let (circuit, node) = resistor_circuit();
    let mut model =
        model_from_instance(circuit.device_by_name("r1").expect("device")).expect("model");
    let mut context = MnaContext::new(1, 0, vec![None, Some(0)], vec![]).expect("context");
    model
        .stamp(&mut context, &Solution::zero(2, 1), None)
        .expect("stamp");
    context.add_rhs(0, 1e-3).expect("rhs");
    let solution = context.solve().expect("solve");
    assert!((solution.voltage(node).expect("voltage") - 1.0).abs() < 1e-9);
}

#[test]
fn voltage_source_stamps_branch() {
    let (circuit, node) = voltage_source_circuit();
    let mut model =
        model_from_instance(circuit.device_by_name("v1").expect("device")).expect("model");
    let mut context = MnaContext::new(1, 1, vec![None, Some(0)], vec![Some(1)]).expect("context");
    model
        .stamp(&mut context, &Solution::zero(2, 2), None)
        .expect("stamp");
    let solution = context.solve().expect("solve");
    assert!((solution.voltage(node).expect("voltage") - 5.0).abs() < 1e-9);
}

#[test]
fn current_source_injects_into_node() {
    let (circuit, node) = current_source_circuit();
    let mut resistor =
        model_from_instance(circuit.device_by_name("r1").expect("r1")).expect("resistor");
    let mut source =
        model_from_instance(circuit.device_by_name("i1").expect("i1")).expect("source");
    let mut context = MnaContext::new(1, 0, vec![None, Some(0)], vec![]).expect("context");
    let initial = Solution::zero(2, 1);
    resistor
        .stamp(&mut context, &initial, None)
        .expect("resistor");
    source.stamp(&mut context, &initial, None).expect("source");
    let solution = context.solve().expect("solve");
    assert!((solution.voltage(node).expect("voltage") - 1.0).abs() < 1e-9);
}

#[test]
fn custom_device_is_unsupported() {
    let mut builder = CircuitBuilder::new();
    let node = builder.node("n").expect("node");
    let ground = builder.ground();
    let kind = DeviceKind::Custom("mystery".to_string());
    builder
        .device("x1", kind, vec![node, ground], BTreeMap::new())
        .expect("x1");
    let circuit = builder.build().expect("circuit");
    let result = model_from_instance(circuit.device_by_name("x1").expect("device"));
    assert!(result.is_err());
}

#[test]
fn mosfet_model_builds() {
    let circuit = mosfet_circuit();
    let model = model_from_instance(circuit.device_by_name("mn").expect("device"));
    assert!(model.is_ok());
}
