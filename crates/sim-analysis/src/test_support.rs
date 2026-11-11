//! Shared circuit builders for analysis tests.

use sim_core::{Circuit, CircuitBuilder, DeviceKind, NodeId};
use std::collections::BTreeMap;

pub(crate) fn params(name: &str, value: f64) -> BTreeMap<String, f64> {
    let mut map = BTreeMap::new();
    map.insert(name.to_string(), value);
    map
}

pub(crate) fn mos_params(width: f64) -> BTreeMap<String, f64> {
    let mut map = BTreeMap::new();
    map.insert("vto".to_string(), 0.7);
    map.insert("kp".to_string(), 1e-3);
    map.insert("lambda".to_string(), 0.02);
    map.insert("w".to_string(), width);
    map.insert("l".to_string(), 1e-6);
    map
}

pub(crate) struct TestCircuit {
    builder: CircuitBuilder,
}

impl TestCircuit {
    pub(crate) fn new() -> Self {
        Self {
            builder: CircuitBuilder::new(),
        }
    }

    pub(crate) fn node(&mut self, name: &str) -> NodeId {
        self.builder.node(name).expect(name)
    }

    pub(crate) fn supply(&mut self, name: &str) -> NodeId {
        self.builder.supply(name).expect(name)
    }

    pub(crate) fn ground(&self) -> NodeId {
        self.builder.ground()
    }

    pub(crate) fn voltage(&mut self, name: &str, positive: NodeId, negative: NodeId, value: f64) {
        let parameters = params("voltage", value);
        self.two_pin(
            name,
            DeviceKind::VoltageSource,
            positive,
            negative,
            parameters,
        );
    }

    pub(crate) fn resistor(&mut self, name: &str, first: NodeId, second: NodeId, value: f64) {
        let parameters = params("resistance", value);
        self.two_pin(name, DeviceKind::Resistor, first, second, parameters);
    }

    pub(crate) fn capacitor(&mut self, name: &str, first: NodeId, second: NodeId, value: f64) {
        let parameters = params("capacitance", value);
        self.two_pin(name, DeviceKind::Capacitor, first, second, parameters);
    }

    pub(crate) fn inductor(&mut self, name: &str, first: NodeId, second: NodeId, value: f64) {
        let parameters = params("inductance", value);
        self.two_pin(name, DeviceKind::Inductor, first, second, parameters);
    }

    pub(crate) fn current(&mut self, name: &str, from: NodeId, to: NodeId, value: f64) {
        let parameters = params("current", value);
        self.two_pin(name, DeviceKind::CurrentSource, from, to, parameters);
    }

    pub(crate) fn diode(&mut self, name: &str, anode: NodeId, cathode: NodeId) {
        self.two_pin(name, DeviceKind::Diode, anode, cathode, BTreeMap::new());
    }

    pub(crate) fn mosfet(
        &mut self,
        name: &str,
        kind: DeviceKind,
        gate: NodeId,
        drain: NodeId,
        source: NodeId,
        width: f64,
    ) {
        let parameters = mos_params(width);
        self.builder
            .device(name, kind, vec![gate, drain, source], parameters)
            .expect(name);
    }

    pub(crate) fn bjt(
        &mut self,
        name: &str,
        kind: DeviceKind,
        collector: NodeId,
        base: NodeId,
        emitter: NodeId,
        beta: f64,
    ) {
        let parameters = params("bf", beta);
        self.builder
            .device(name, kind, vec![collector, base, emitter], parameters)
            .expect(name);
    }

    pub(crate) fn jfet(
        &mut self,
        name: &str,
        kind: DeviceKind,
        drain: NodeId,
        gate: NodeId,
        source: NodeId,
        parameters: (f64, f64),
    ) {
        let mut map = BTreeMap::new();
        map.insert("vto".to_string(), parameters.0);
        map.insert("beta".to_string(), parameters.1);
        self.builder
            .device(name, kind, vec![drain, gate, source], map)
            .expect(name);
    }

    pub(crate) fn build(self) -> Circuit {
        self.builder.build().expect("circuit")
    }

    fn two_pin(
        &mut self,
        name: &str,
        kind: DeviceKind,
        first: NodeId,
        second: NodeId,
        parameters: BTreeMap<String, f64>,
    ) {
        self.builder
            .device(name, kind, vec![first, second], parameters)
            .expect(name);
    }
}

pub(crate) fn voltage_divider() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.voltage("v1", input, ground, 10.0);
    circuit.resistor("r1", input, output, 1000.0);
    circuit.resistor("r2", output, ground, 1000.0);
    (circuit.build(), output)
}

pub(crate) fn inverter() -> (Circuit, NodeId, NodeId) {
    let mut circuit = TestCircuit::new();
    let vdd = circuit.supply("vdd");
    let input = circuit.node("input");
    let output = circuit.node("output");
    let ground = circuit.ground();
    circuit.voltage("vdd_src", vdd, ground, 5.0);
    circuit.voltage("vin", input, ground, 0.0);
    circuit.mosfet("mp", DeviceKind::Pmos, input, output, vdd, 10e-6);
    circuit.mosfet("mn", DeviceKind::Nmos, input, output, ground, 5e-6);
    (circuit.build(), input, output)
}

pub(crate) fn rc_circuit() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.voltage("v1", input, ground, 5.0);
    circuit.resistor("r1", input, output, 1000.0);
    circuit.capacitor("c1", output, ground, 1e-6);
    (circuit.build(), output)
}

pub(crate) fn rl_circuit() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.voltage("v1", input, ground, 5.0);
    circuit.resistor("r1", input, output, 1000.0);
    circuit.inductor("l1", output, ground, 1e-3);
    (circuit.build(), output)
}

pub(crate) fn current_source_circuit() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.current("i1", ground, output, 0.001);
    circuit.resistor("r1", output, ground, 1000.0);
    (circuit.build(), output)
}

pub(crate) fn resistor_to_ground() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.resistor("r1", output, ground, 1000.0);
    (circuit.build(), output)
}

pub(crate) fn common_emitter() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let vcc = circuit.node("vcc");
    let base_drive = circuit.node("base_drive");
    let base = circuit.node("base");
    let collector = circuit.node("collector");
    let ground = circuit.ground();
    circuit.voltage("vcc_src", vcc, ground, 5.0);
    circuit.voltage("vb", base_drive, ground, 2.0);
    circuit.resistor("rb", base_drive, base, 100_000.0);
    circuit.resistor("rc", vcc, collector, 1000.0);
    circuit.bjt("q1", DeviceKind::Npn, collector, base, ground, 100.0);
    (circuit.build(), collector)
}

pub(crate) fn common_emitter_pnp() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let vcc = circuit.node("vcc");
    let base = circuit.node("base");
    let collector = circuit.node("collector");
    let ground = circuit.ground();
    circuit.voltage("vcc_src", vcc, ground, 5.0);
    circuit.voltage("vb", base, ground, 4.3);
    circuit.resistor("rc", collector, ground, 1000.0);
    circuit.bjt("q1", DeviceKind::Pnp, collector, base, vcc, 100.0);
    (circuit.build(), collector)
}

pub(crate) fn common_source_n() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let vcc = circuit.node("vcc");
    let drain = circuit.node("drain");
    let ground = circuit.ground();
    circuit.voltage("vcc_src", vcc, ground, 5.0);
    circuit.resistor("rd", vcc, drain, 1000.0);
    circuit.jfet("j1", DeviceKind::JfetN, drain, ground, ground, (-2.0, 1e-3));
    (circuit.build(), drain)
}

pub(crate) fn common_source_n_biased() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let vcc = circuit.node("vcc");
    let gate = circuit.node("gate");
    let drain = circuit.node("drain");
    let ground = circuit.ground();
    circuit.voltage("vcc_src", vcc, ground, 5.0);
    circuit.voltage("vg", gate, ground, -1.0);
    circuit.resistor("rd", vcc, drain, 1000.0);
    circuit.jfet("j1", DeviceKind::JfetN, drain, gate, ground, (-2.0, 1e-3));
    (circuit.build(), drain)
}

pub(crate) fn common_source_p() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let vcc = circuit.node("vcc");
    let drain = circuit.node("drain");
    let ground = circuit.ground();
    circuit.voltage("vcc_src", vcc, ground, 5.0);
    circuit.resistor("rd", drain, ground, 1000.0);
    circuit.jfet("j1", DeviceKind::JfetP, drain, vcc, vcc, (2.0, 1e-3));
    (circuit.build(), drain)
}

pub(crate) fn diode_clamp() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.voltage("v1", input, ground, 5.0);
    circuit.resistor("r1", input, output, 1000.0);
    circuit.diode("d1", output, ground);
    (circuit.build(), output)
}
