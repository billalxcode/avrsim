use sim_analysis::{DcConfig, dc_operating_point, dc_sweep};
use sim_cells::{
    CellKind, buffer, d_flip_flop, d_latch, inverter, nand2, nor2, ring_oscillator, sram6t,
    transmission_gate, tri_state_inverter,
};
use sim_core::{Circuit, DeviceKind, NodeId, SimError};
use std::collections::BTreeMap;

fn params(name: &str, value: f64) -> BTreeMap<String, f64> {
    let mut map = BTreeMap::new();
    map.insert(name.to_string(), value);
    map
}

fn add_source(
    circuit: &mut Circuit,
    name: &str,
    node: NodeId,
    voltage: f64,
) -> Result<(), SimError> {
    let ground = circuit.ground();
    circuit.add_device(
        name,
        DeviceKind::VoltageSource,
        vec![node, ground],
        params("voltage", voltage),
    )?;
    Ok(())
}

fn output(circuit: &Circuit, node: NodeId) -> f64 {
    let result = dc_operating_point(circuit, &DcConfig::default()).expect("dc");
    result.solution.voltage(node).expect("output")
}

fn inverter_sweep() -> (NodeId, Vec<sim_analysis::DcPoint>) {
    let cell = inverter().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    let input = cell.port("a").expect("a");
    add_source(&mut circuit, "vin", input, 0.0).expect("source");
    let points = dc_sweep(
        &circuit,
        "vin",
        "voltage",
        0.0,
        5.0,
        0.1,
        &DcConfig::default(),
    )
    .expect("sweep");
    (cell.port("y").expect("y"), points)
}

#[test]
fn inverter_vtc_switches() {
    let (output_port, points) = inverter_sweep();
    let first = points.first().expect("first");
    let last = points.last().expect("last");
    assert!(first.solution.voltage(output_port).expect("high") > 4.0);
    assert!(last.solution.voltage(output_port).expect("low") < 1.0);
}

fn nand_output(first: f64, second: f64) -> f64 {
    let cell = nand2().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    let a = cell.port("a").expect("a");
    let b = cell.port("b").expect("b");
    add_source(&mut circuit, "va", a, first).expect("va");
    add_source(&mut circuit, "vb", b, second).expect("vb");
    output(&circuit, cell.port("y").expect("y"))
}

#[test]
fn nand_truth_table() {
    assert!(nand_output(0.0, 0.0) > 4.0);
    assert!(nand_output(0.0, 5.0) > 4.0);
    assert!(nand_output(5.0, 0.0) > 4.0);
    assert!(nand_output(5.0, 5.0) < 1.0);
}

fn nor_output(first: f64, second: f64) -> f64 {
    let cell = nor2().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    let a = cell.port("a").expect("a");
    let b = cell.port("b").expect("b");
    add_source(&mut circuit, "va", a, first).expect("va");
    add_source(&mut circuit, "vb", b, second).expect("vb");
    output(&circuit, cell.port("y").expect("y"))
}

#[test]
fn nor_truth_table() {
    assert!(nor_output(0.0, 0.0) > 4.0);
    assert!(nor_output(0.0, 5.0) < 1.0);
    assert!(nor_output(5.0, 0.0) < 1.0);
    assert!(nor_output(5.0, 5.0) < 1.0);
}

#[test]
fn buffer_follows_input() {
    let cell = buffer().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    let input = cell.port("a").expect("a");
    let output_port = cell.port("y").expect("y");
    add_source(&mut circuit, "vin", input, 0.0).expect("source");
    assert!(output(&circuit, output_port) < 1.0);
    circuit
        .set_device_parameter("vin", "voltage", 5.0)
        .expect("set input");
    assert!(output(&circuit, output_port) > 4.0);
}

#[test]
fn powered_cell_has_supply() {
    let cell = inverter().expect("cell");
    let circuit = cell.powered(5.0).expect("powered");
    assert!(circuit.device_by_name("vdd_src").is_some());
    assert_eq!(cell.vdd(), cell.port("vdd"));
}
fn inverter_output(input_voltage: f64) -> f64 {
    let cell = inverter().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    let input = cell.port("a").expect("a");
    add_source(&mut circuit, "vin", input, input_voltage).expect("source");
    output(&circuit, cell.port("y").expect("y"))
}

#[test]
fn inverter_truth_table() {
    assert!(inverter_output(0.0) > 4.0);
    assert!(inverter_output(5.0) < 1.0);
}

#[test]
fn inverter_threshold_is_near_half_supply() {
    let (output_port, points) = inverter_sweep();
    let crossing = points
        .iter()
        .find(|point| point.solution.voltage(output_port).expect("out") < 2.5)
        .expect("crossing");
    assert!(crossing.sweep > 2.0 && crossing.sweep < 3.0);
}
#[test]
fn transmission_gate_has_ports() {
    let cell = transmission_gate().expect("cell");
    assert!(cell.port("a").is_some());
    assert!(cell.port("b").is_some());
    assert!(cell.port("en").is_some());
    assert!(cell.port("enb").is_some());
}

#[test]
fn tri_state_inverter_has_ports() {
    let cell = tri_state_inverter().expect("cell");
    assert!(cell.port("a").is_some());
    assert!(cell.port("y").is_some());
    assert!(cell.port("en").is_some());
    assert!(cell.port("enb").is_some());
}

#[test]
fn d_latch_captures_input() {
    let cell = d_latch().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    let data = cell.port("d").expect("d");
    let enable = cell.port("en").expect("en");
    let enable_bar = cell.port("enb").expect("enb");
    add_source(&mut circuit, "vd", data, 5.0).expect("vd");
    add_source(&mut circuit, "ven", enable, 5.0).expect("ven");
    add_source(&mut circuit, "venb", enable_bar, 0.0).expect("venb");
    let output_port = cell.port("q").expect("q");
    assert!(output(&circuit, output_port) > 4.0);
    circuit
        .set_device_parameter("vd", "voltage", 0.0)
        .expect("set data");
    assert!(output(&circuit, output_port) < 1.0);
}

#[test]
fn dff_has_ports() {
    let cell = d_flip_flop().expect("cell");
    assert!(cell.port("d").is_some());
    assert!(cell.port("clk").is_some());
    assert!(cell.port("q").is_some());
}

#[test]
fn sram_has_ports() {
    let cell = sram6t().expect("cell");
    assert!(cell.port("q").is_some());
    assert!(cell.port("qb").is_some());
    assert!(cell.port("wl").is_some());
    assert!(cell.port("bl").is_some());
    assert!(cell.port("blb").is_some());
}

#[test]
fn ring_oscillator_has_output() {
    let cell = ring_oscillator().expect("cell");
    assert!(cell.port("y").is_some());
    assert!(cell.powered(5.0).is_ok());
}

#[test]
fn registry_builds_every_cell() {
    let kinds = [
        CellKind::Inverter,
        CellKind::Nand2,
        CellKind::Nor2,
        CellKind::Buffer,
        CellKind::TransmissionGate,
        CellKind::TriStateInverter,
        CellKind::DLatch,
        CellKind::DFlipFlop,
        CellKind::Sram6t,
        CellKind::RingOscillator,
    ];
    for kind in kinds {
        let cell = kind.build().expect("cell");
        assert!(!kind.name().is_empty());
        assert!(cell.vdd().is_some());
    }
}
