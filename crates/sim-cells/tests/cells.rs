use sim_analysis::{DcConfig, dc_operating_point, dc_sweep};
use sim_cells::{
    CellKind, adder8, and2, buffer, d_flip_flop, d_latch, full_adder, half_adder, inverter, mux2,
    nand2, noise_margins, nor2, or2, ring_oscillator, ripple_carry_adder, sram6t,
    transmission_gate, tri_state_inverter, xor2,
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
    for kind in CellKind::ALL {
        let cell = kind.build().expect("cell");
        assert!(!kind.name().is_empty());
        assert!(cell.vdd().is_some());
    }
}
#[test]
fn inverter_noise_margins_are_positive() {
    let (output_port, points) = inverter_sweep();
    let samples: Vec<(f64, f64)> = points
        .iter()
        .map(|point| {
            (
                point.sweep,
                point.solution.voltage(output_port).unwrap_or(0.0),
            )
        })
        .collect();
    let metrics = noise_margins(&samples).expect("metrics");
    assert!(metrics.voh > 4.0);
    assert!(metrics.vol < 1.0);
    assert!(metrics.nmh > 1.0);
    assert!(metrics.nml > 1.0);
}
fn half_adder_outputs(first: f64, second: f64) -> (f64, f64) {
    let cell = half_adder().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    let a = cell.port("a").expect("a");
    let b = cell.port("b").expect("b");
    add_source(&mut circuit, "va", a, first).expect("va");
    add_source(&mut circuit, "vb", b, second).expect("vb");
    let sum = output(&circuit, cell.port("sum").expect("sum"));
    let carry = output(&circuit, cell.port("carry").expect("carry"));
    (sum, carry)
}

#[test]
fn half_adder_truth_table() {
    let (sum, carry) = half_adder_outputs(0.0, 0.0);
    assert!(sum < 1.0 && carry < 1.0);
    let (sum, carry) = half_adder_outputs(0.0, 5.0);
    assert!(sum > 4.0 && carry < 1.0);
    let (sum, carry) = half_adder_outputs(5.0, 0.0);
    assert!(sum > 4.0 && carry < 1.0);
    let (sum, carry) = half_adder_outputs(5.0, 5.0);
    assert!(sum < 1.0 && carry > 4.0);
}
fn full_adder_outputs(first: f64, second: f64, carry_in: f64) -> (f64, f64) {
    let cell = full_adder().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    let a = cell.port("a").expect("a");
    let b = cell.port("b").expect("b");
    let c = cell.port("cin").expect("cin");
    add_source(&mut circuit, "va", a, first).expect("va");
    add_source(&mut circuit, "vb", b, second).expect("vb");
    add_source(&mut circuit, "vc", c, carry_in).expect("vc");
    let sum = output(&circuit, cell.port("sum").expect("sum"));
    let carry = output(&circuit, cell.port("carry").expect("carry"));
    (sum, carry)
}

#[test]
fn full_adder_truth_table() {
    let cases = [
        (0.0, 0.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, 5.0, 5.0, 0.0),
        (0.0, 5.0, 0.0, 5.0, 0.0),
        (0.0, 5.0, 5.0, 0.0, 5.0),
        (5.0, 0.0, 0.0, 5.0, 0.0),
        (5.0, 0.0, 5.0, 0.0, 5.0),
        (5.0, 5.0, 0.0, 0.0, 5.0),
        (5.0, 5.0, 5.0, 5.0, 5.0),
    ];
    for (a, b, c, expected_sum, expected_carry) in cases {
        let (sum, carry) = full_adder_outputs(a, b, c);
        assert!((sum - expected_sum).abs() < 1.0);
        assert!((carry - expected_carry).abs() < 1.0);
    }
}
fn build_adder(bits: usize) -> (sim_cells::Cell, Circuit) {
    let cell = ripple_carry_adder(bits).expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    for bit in 0..bits {
        add_adder_input(&mut circuit, &cell, bit);
    }
    add_source(&mut circuit, "vcin", cell.port("cin").expect("cin"), 0.0).expect("vcin");
    (cell, circuit)
}

fn add_adder_input(circuit: &mut Circuit, cell: &sim_cells::Cell, bit: usize) {
    add_source(
        circuit,
        &format!("va{bit}"),
        cell.port(&format!("a{bit}")).expect("a"),
        0.0,
    )
    .expect("va");
    add_source(
        circuit,
        &format!("vb{bit}"),
        cell.port(&format!("b{bit}")).expect("b"),
        0.0,
    )
    .expect("vb");
}

fn apply_adder_inputs(circuit: &mut Circuit, bits: usize, first: u8, second: u8, carry: bool) {
    for bit in 0..bits {
        let a = if (first >> bit) & 1 == 1 { 5.0 } else { 0.0 };
        let b = if (second >> bit) & 1 == 1 { 5.0 } else { 0.0 };
        circuit
            .set_device_parameter(&format!("va{bit}"), "voltage", a)
            .expect("va");
        circuit
            .set_device_parameter(&format!("vb{bit}"), "voltage", b)
            .expect("vb");
    }
    let cin = if carry { 5.0 } else { 0.0 };
    circuit
        .set_device_parameter("vcin", "voltage", cin)
        .expect("vcin");
}

fn read_adder(circuit: &Circuit, cell: &sim_cells::Cell, bits: usize) -> (u8, bool) {
    let result = dc_operating_point(circuit, &DcConfig::default()).expect("dc");
    let mut sum = 0;
    for bit in 0..bits {
        let node = cell.port(&format!("sum{bit}")).expect("sum");
        if result.solution.voltage(node).unwrap_or(0.0) > 2.5 {
            sum |= 1 << bit;
        }
    }
    let carry = result
        .solution
        .voltage(cell.port("cout").expect("cout"))
        .unwrap_or(0.0)
        > 2.5;
    (sum, carry)
}

#[test]
fn four_bit_adder_truth_table() {
    let (cell, mut circuit) = build_adder(4);
    let cases = [
        (0, 0, false, 0, false),
        (3, 4, false, 7, false),
        (9, 6, false, 15, false),
        (15, 1, false, 0, true),
    ];
    for (first, second, carry_in, expected_sum, expected_carry) in cases {
        apply_adder_inputs(&mut circuit, 4, first, second, carry_in);
        let (sum, carry) = read_adder(&circuit, &cell, 4);
        assert_eq!(sum, expected_sum);
        assert_eq!(carry, expected_carry);
    }
}

#[test]
fn eight_bit_adder_has_ports() {
    let cell = adder8().expect("cell");
    assert!(cell.port("a7").is_some());
    assert!(cell.port("b7").is_some());
    assert!(cell.port("sum7").is_some());
    assert!(cell.port("cout").is_some());
}

fn gate_output(cell: sim_cells::Cell, inputs: &[(&str, f64)]) -> f64 {
    let mut circuit = cell.powered(5.0).expect("powered");
    for (index, (port, voltage)) in inputs.iter().enumerate() {
        let node = cell.port(port).expect("input port");
        add_source(&mut circuit, &format!("v{index}"), node, *voltage).expect("source");
    }
    output(&circuit, cell.port("y").expect("y"))
}

#[test]
fn and_gate_truth_table() {
    assert!(gate_output(and2().expect("cell"), &[("a", 0.0), ("b", 0.0)]) < 1.0);
    assert!(gate_output(and2().expect("cell"), &[("a", 0.0), ("b", 5.0)]) < 1.0);
    assert!(gate_output(and2().expect("cell"), &[("a", 5.0), ("b", 0.0)]) < 1.0);
    assert!(gate_output(and2().expect("cell"), &[("a", 5.0), ("b", 5.0)]) > 4.0);
}

#[test]
fn or_gate_truth_table() {
    assert!(gate_output(or2().expect("cell"), &[("a", 0.0), ("b", 0.0)]) < 1.0);
    assert!(gate_output(or2().expect("cell"), &[("a", 0.0), ("b", 5.0)]) > 4.0);
    assert!(gate_output(or2().expect("cell"), &[("a", 5.0), ("b", 0.0)]) > 4.0);
    assert!(gate_output(or2().expect("cell"), &[("a", 5.0), ("b", 5.0)]) > 4.0);
}

#[test]
fn xor_gate_truth_table() {
    assert!(gate_output(xor2().expect("cell"), &[("a", 0.0), ("b", 0.0)]) < 1.0);
    assert!(gate_output(xor2().expect("cell"), &[("a", 0.0), ("b", 5.0)]) > 4.0);
    assert!(gate_output(xor2().expect("cell"), &[("a", 5.0), ("b", 0.0)]) > 4.0);
    assert!(gate_output(xor2().expect("cell"), &[("a", 5.0), ("b", 5.0)]) < 1.0);
}

#[test]
fn mux_selects_inputs() {
    let cases = [
        (0.0, 0.0, 0.0, 0.0),
        (5.0, 0.0, 0.0, 5.0),
        (0.0, 5.0, 0.0, 0.0),
        (0.0, 0.0, 5.0, 0.0),
        (5.0, 0.0, 5.0, 0.0),
        (0.0, 5.0, 5.0, 5.0),
    ];
    for (first, second, select, expected) in cases {
        let result = gate_output(
            mux2().expect("cell"),
            &[("a", first), ("b", second), ("sel", select)],
        );
        assert!((result - expected).abs() < 1.0);
    }
}
