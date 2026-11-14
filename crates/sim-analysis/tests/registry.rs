//! Registry-backed analog emulation.

use sim_analysis::{DcConfig, dc_operating_point};
use sim_core::{Circuit, CircuitBuilder, DeviceId, NodeId};
use sim_device::{ComponentRegistry, ComponentRequest};

const SOURCE: &str = "source.independent.dc_voltage_source";
const RESISTOR: &str = "passive.resistor.ideal_resistor";
const SWITCH: &str = "switch.mechanical.spst_switch";
const ANALOG_SWITCH: &str = "switch.analog.analog_switch";
const RELAY: &str = "switch.relay.electromechanical_relay";
const OPAMP: &str = "analog.opamp.ideal_opamp";
const COMPARATOR: &str = "analog.comparator.comparator";

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

fn add_analog_switch(
    registry: &ComponentRegistry,
    builder: &mut CircuitBuilder,
    name: &str,
    first: NodeId,
    second: NodeId,
    control: NodeId,
    ground: NodeId,
) {
    let pins = vec![first, second, control, ground];
    let request = ComponentRequest::new(ANALOG_SWITCH, name, DeviceId::new(0), pins);
    registry.add_to(builder, &request).expect("analog switch");
}

fn add_relay(
    registry: &ComponentRegistry,
    builder: &mut CircuitBuilder,
    name: &str,
    pins: [NodeId; 5],
) {
    let request = ComponentRequest::new(RELAY, name, DeviceId::new(0), pins.to_vec());
    registry.add_to(builder, &request).expect("relay");
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

fn add_comparator(
    registry: &ComponentRegistry,
    builder: &mut CircuitBuilder,
    name: &str,
    pins: [NodeId; 5],
) {
    let request = ComponentRequest::new(COMPARATOR, name, DeviceId::new(0), pins.to_vec());
    registry.add_to(builder, &request).expect("comparator");
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

fn add_switch_load(
    registry: &ComponentRegistry,
    builder: &mut CircuitBuilder,
    source: NodeId,
    out: NodeId,
    ground: NodeId,
) {
    add_resistor(registry, builder, "R1", source, out, 1000.0);
    add_resistor(registry, builder, "R2", out, ground, 1_000_000.0);
}

fn switch_circuit(closed: f64) -> Circuit {
    let registry = ComponentRegistry::new();
    let mut builder = CircuitBuilder::new();
    let source = builder.node("source").expect("source");
    let out = builder.node("out").expect("out");
    let ground = builder.ground();
    add_source(&registry, &mut builder, "V1", source, ground, 5.0);
    add_switch_load(&registry, &mut builder, source, out, ground);
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

fn analog_switch_circuit(control_voltage: f64) -> Circuit {
    let registry = ComponentRegistry::new();
    let mut builder = CircuitBuilder::new();
    let source = builder.node("source").expect("source");
    let out = builder.node("out").expect("out");
    let control = builder.node("control").expect("control");
    let ground = builder.ground();
    add_source(&registry, &mut builder, "V1", source, ground, 5.0);
    add_source(
        &registry,
        &mut builder,
        "VC",
        control,
        ground,
        control_voltage,
    );
    add_switch_load(&registry, &mut builder, source, out, ground);
    add_analog_switch(&registry, &mut builder, "SW1", out, ground, control, ground);
    builder.build().expect("circuit")
}

struct RelayNodes {
    main: NodeId,
    common: NodeId,
    normally_closed: NodeId,
    coil: NodeId,
    ground: NodeId,
}

fn relay_nodes(builder: &mut CircuitBuilder) -> RelayNodes {
    RelayNodes {
        main: builder.node("main").expect("main"),
        common: builder.node("common").expect("common"),
        normally_closed: builder.node("nc").expect("nc"),
        coil: builder.node("coil").expect("coil"),
        ground: builder.ground(),
    }
}

fn add_relay_load(registry: &ComponentRegistry, builder: &mut CircuitBuilder, nodes: &RelayNodes) {
    add_resistor(registry, builder, "R1", nodes.main, nodes.common, 1000.0);
    add_resistor(
        registry,
        builder,
        "R2",
        nodes.normally_closed,
        nodes.ground,
        1_000_000.0,
    );
}

fn add_relay_coil(
    registry: &ComponentRegistry,
    builder: &mut CircuitBuilder,
    nodes: &RelayNodes,
    voltage: f64,
) {
    add_source(registry, builder, "VC", nodes.coil, nodes.ground, voltage);
}

fn relay_pins(nodes: &RelayNodes) -> [NodeId; 5] {
    [
        nodes.coil,
        nodes.ground,
        nodes.common,
        nodes.ground,
        nodes.normally_closed,
    ]
}

fn relay_circuit(coil_voltage: f64) -> Circuit {
    let registry = ComponentRegistry::new();
    let mut builder = CircuitBuilder::new();
    let nodes = relay_nodes(&mut builder);
    add_source(&registry, &mut builder, "V1", nodes.main, nodes.ground, 5.0);
    add_relay_coil(&registry, &mut builder, &nodes, coil_voltage);
    add_relay_load(&registry, &mut builder, &nodes);
    add_relay(&registry, &mut builder, "K1", relay_pins(&nodes));
    builder.build().expect("circuit")
}

fn comparator_circuit(input_voltage: f64) -> Circuit {
    let registry = ComponentRegistry::new();
    let mut builder = CircuitBuilder::new();
    let input = builder.node("input").expect("input");
    let reference = builder.node("reference").expect("reference");
    let supply = builder.node("supply").expect("supply");
    let output = builder.node("output").expect("output");
    let ground = builder.ground();
    add_source(&registry, &mut builder, "VIN", input, ground, input_voltage);
    add_source(&registry, &mut builder, "VREF", reference, ground, 2.5);
    add_source(&registry, &mut builder, "VCC", supply, ground, 5.0);
    add_comparator(
        &registry,
        &mut builder,
        "CMP",
        [input, reference, supply, ground, output],
    );
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

#[test]
fn registry_comparator_saturates_both_ways() {
    let high = dc_voltage(&comparator_circuit(3.0), "output");
    let low = dc_voltage(&comparator_circuit(2.0), "output");
    assert!(high > 4.9, "high {high}");
    assert!(low < 0.1, "low {low}");
}

#[test]
fn registry_analog_switch_follows_control() {
    let off = dc_voltage(&analog_switch_circuit(0.0), "out");
    let on = dc_voltage(&analog_switch_circuit(5.0), "out");
    assert!(off > 4.9, "off {off}");
    assert!(on < 0.1, "on {on}");
}

#[test]
fn registry_relay_switches_contacts() {
    let released = dc_voltage(&relay_circuit(0.0), "common");
    let energized = dc_voltage(&relay_circuit(5.0), "common");
    assert!(released > 4.9, "released {released}");
    assert!(energized < 0.1, "energized {energized}");
}
