//! Two-state digital logic with an ideal clock.

use sim_core::SimError;
use std::collections::BTreeMap;

/// Maximum settle iterations before a combinational loop is reported.
const MAX_SETTLE_ITERATIONS: usize = 10_000;

/// Identifies a digital net.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NetId(usize);

impl NetId {
    /// Returns the raw index.
    #[must_use]
    pub const fn raw(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
enum Gate {
    Const(bool, NetId),
    Not(NetId, NetId),
    And(NetId, NetId, NetId),
    Or(NetId, NetId, NetId),
    Xor(NetId, NetId, NetId),
    Nand(NetId, NetId, NetId),
    Nor(NetId, NetId, NetId),
    Mux {
        first: NetId,
        second: NetId,
        select: NetId,
        output: NetId,
    },
}

#[derive(Clone, Copy, Debug)]
struct Dff {
    data: NetId,
    output: NetId,
}

/// Builds a two-state digital circuit.
pub struct DigitalBuilder {
    values: Vec<bool>,
    gates: Vec<Gate>,
    dffs: Vec<Dff>,
    inputs: BTreeMap<String, NetId>,
    outputs: BTreeMap<String, NetId>,
}

impl DigitalBuilder {
    /// Creates an empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            gates: Vec::new(),
            dffs: Vec::new(),
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
        }
    }

    /// Adds a named input net.
    pub fn input(&mut self, name: &str) -> NetId {
        let net = self.new_net();
        self.inputs.insert(name.to_string(), net);
        net
    }

    /// Exposes a net as a named output.
    pub fn output(&mut self, name: &str, net: NetId) {
        self.outputs.insert(name.to_string(), net);
    }

    /// Adds an unnamed net.
    pub fn net(&mut self) -> NetId {
        self.new_net()
    }

    /// Adds a constant net.
    pub fn constant(&mut self, value: bool) -> NetId {
        let output = self.new_net();
        self.gates.push(Gate::Const(value, output));
        output
    }

    /// Adds an inverter.
    pub fn not(&mut self, input: NetId) -> NetId {
        let output = self.new_net();
        self.gates.push(Gate::Not(input, output));
        output
    }

    /// Adds an inverter that drives an existing net.
    pub fn not_into(&mut self, input: NetId, output: NetId) {
        self.gates.push(Gate::Not(input, output));
    }

    /// Adds an AND gate.
    pub fn and(&mut self, first: NetId, second: NetId) -> NetId {
        let output = self.new_net();
        self.gates.push(Gate::And(first, second, output));
        output
    }

    /// Adds an OR gate.
    pub fn or(&mut self, first: NetId, second: NetId) -> NetId {
        let output = self.new_net();
        self.gates.push(Gate::Or(first, second, output));
        output
    }

    /// Adds an XOR gate.
    pub fn xor(&mut self, first: NetId, second: NetId) -> NetId {
        let output = self.new_net();
        self.gates.push(Gate::Xor(first, second, output));
        output
    }

    /// Adds a NAND gate.
    pub fn nand(&mut self, first: NetId, second: NetId) -> NetId {
        let output = self.new_net();
        self.gates.push(Gate::Nand(first, second, output));
        output
    }

    /// Adds a NOR gate.
    pub fn nor(&mut self, first: NetId, second: NetId) -> NetId {
        let output = self.new_net();
        self.gates.push(Gate::Nor(first, second, output));
        output
    }

    /// Adds a two-input multiplexer.
    pub fn mux(&mut self, first: NetId, second: NetId, select: NetId) -> NetId {
        let output = self.new_net();
        self.gates.push(Gate::Mux {
            first,
            second,
            select,
            output,
        });
        output
    }

    /// Adds a D flip-flop that captures data on every clock tick.
    pub fn dff(&mut self, data: NetId) -> NetId {
        let output = self.new_net();
        self.dff_into(data, output);
        output
    }

    /// Adds a D flip-flop that drives an existing net.
    pub fn dff_into(&mut self, data: NetId, output: NetId) {
        self.dffs.push(Dff { data, output });
    }

    /// Builds the digital circuit.
    #[must_use]
    pub fn build(self) -> DigitalCircuit {
        DigitalCircuit {
            values: self.values,
            gates: self.gates,
            dffs: self.dffs,
            inputs: self.inputs,
            outputs: self.outputs,
        }
    }

    fn new_net(&mut self) -> NetId {
        let net = NetId(self.values.len());
        self.values.push(false);
        net
    }
}

impl Default for DigitalBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A two-state digital circuit with an ideal clock.
pub struct DigitalCircuit {
    values: Vec<bool>,
    gates: Vec<Gate>,
    dffs: Vec<Dff>,
    inputs: BTreeMap<String, NetId>,
    outputs: BTreeMap<String, NetId>,
}

impl DigitalCircuit {
    /// Sets a named input value without settling.
    pub fn set_input(&mut self, name: &str, value: bool) -> Result<(), SimError> {
        let net = self
            .inputs
            .get(name)
            .ok_or_else(|| SimError::InvalidNetlist(format!("unknown input: {name}")))?;
        self.values[net.raw()] = value;
        Ok(())
    }

    /// Reads a named output.
    #[must_use]
    pub fn output(&self, name: &str) -> Option<bool> {
        self.outputs.get(name).map(|net| self.values[net.raw()])
    }

    /// Reads a net value.
    #[must_use]
    pub fn value(&self, net: NetId) -> bool {
        self.values[net.raw()]
    }

    /// Settles combinational logic.
    pub fn settle(&mut self) -> Result<(), SimError> {
        for iteration in 0..MAX_SETTLE_ITERATIONS {
            if !self.evaluate_once() {
                return Ok(());
            }
            if iteration + 1 == MAX_SETTLE_ITERATIONS {
                return Err(SimError::ConvergenceFailure { iteration });
            }
        }
        Ok(())
    }

    /// Applies one ideal clock tick and settles.
    pub fn tick(&mut self) -> Result<(), SimError> {
        self.settle()?;
        let next = self.dff_inputs();
        self.update_dffs(&next);
        self.settle()
    }

    /// Clears all flip-flops and settles.
    pub fn reset(&mut self) -> Result<(), SimError> {
        for dff in &self.dffs {
            self.values[dff.output.raw()] = false;
        }
        self.settle()
    }

    fn evaluate_once(&mut self) -> bool {
        let mut changed = false;
        for gate in &self.gates {
            changed |= evaluate(*gate, &mut self.values);
        }
        changed
    }

    fn dff_inputs(&self) -> Vec<bool> {
        self.dffs
            .iter()
            .map(|dff| self.values[dff.data.raw()])
            .collect()
    }

    fn update_dffs(&mut self, next: &[bool]) {
        for (dff, value) in self.dffs.iter().zip(next) {
            self.values[dff.output.raw()] = *value;
        }
    }
}

fn evaluate(gate: Gate, values: &mut [bool]) -> bool {
    match gate {
        Gate::Const(value, output) => set_value(values, output, value),
        Gate::Not(input, output) => set_value(values, output, !values[input.raw()]),
        Gate::And(first, second, output) => evaluate_and(first, second, output, values),
        Gate::Or(first, second, output) => evaluate_or(first, second, output, values),
        Gate::Xor(first, second, output) => evaluate_xor(first, second, output, values),
        Gate::Nand(first, second, output) => evaluate_nand(first, second, output, values),
        Gate::Nor(first, second, output) => evaluate_nor(first, second, output, values),
        Gate::Mux {
            first,
            second,
            select,
            output,
        } => evaluate_mux(first, second, select, output, values),
    }
}

fn evaluate_and(first: NetId, second: NetId, output: NetId, values: &mut [bool]) -> bool {
    set_value(values, output, values[first.raw()] && values[second.raw()])
}

fn evaluate_or(first: NetId, second: NetId, output: NetId, values: &mut [bool]) -> bool {
    set_value(values, output, values[first.raw()] || values[second.raw()])
}

fn evaluate_xor(first: NetId, second: NetId, output: NetId, values: &mut [bool]) -> bool {
    set_value(values, output, values[first.raw()] ^ values[second.raw()])
}

fn evaluate_nand(first: NetId, second: NetId, output: NetId, values: &mut [bool]) -> bool {
    set_value(
        values,
        output,
        !(values[first.raw()] && values[second.raw()]),
    )
}

fn evaluate_nor(first: NetId, second: NetId, output: NetId, values: &mut [bool]) -> bool {
    set_value(
        values,
        output,
        !(values[first.raw()] || values[second.raw()]),
    )
}

fn evaluate_mux(
    first: NetId,
    second: NetId,
    select: NetId,
    output: NetId,
    values: &mut [bool],
) -> bool {
    let value = if values[select.raw()] {
        values[second.raw()]
    } else {
        values[first.raw()]
    };
    set_value(values, output, value)
}

fn set_value(values: &mut [bool], net: NetId, value: bool) -> bool {
    let slot = &mut values[net.raw()];
    if *slot == value {
        return false;
    }
    *slot = value;
    true
}

#[cfg(test)]
mod tests {
    use super::{DigitalBuilder, DigitalCircuit};
    use sim_core::SimError;

    fn combinational_circuit() -> DigitalCircuit {
        let mut builder = DigitalBuilder::new();
        let a = builder.input("a");
        let b = builder.input("b");
        let not = builder.not(a);
        let and = builder.and(a, b);
        let or = builder.or(a, b);
        let xor = builder.xor(a, b);
        let nand = builder.nand(a, b);
        let nor = builder.nor(a, b);
        builder.output("not", not);
        builder.output("and", and);
        builder.output("or", or);
        builder.output("xor", xor);
        builder.output("nand", nand);
        builder.output("nor", nor);
        builder.build()
    }

    fn combinational(first: bool, second: bool) -> (bool, bool, bool, bool, bool, bool) {
        let mut circuit = combinational_circuit();
        circuit.set_input("a", first).expect("a");
        circuit.set_input("b", second).expect("b");
        circuit.settle().expect("settle");
        combinational_outputs(&circuit)
    }

    fn combinational_outputs(circuit: &DigitalCircuit) -> (bool, bool, bool, bool, bool, bool) {
        (
            circuit.output("not").expect("not"),
            circuit.output("and").expect("and"),
            circuit.output("or").expect("or"),
            circuit.output("xor").expect("xor"),
            circuit.output("nand").expect("nand"),
            circuit.output("nor").expect("nor"),
        )
    }

    #[test]
    fn combinational_gates_follow_truth_tables() {
        assert_eq!(
            combinational(false, false),
            (true, false, false, false, true, true)
        );
        assert_eq!(
            combinational(false, true),
            (true, false, true, true, true, false)
        );
        assert_eq!(
            combinational(true, false),
            (false, false, true, true, true, false)
        );
        assert_eq!(
            combinational(true, true),
            (false, true, true, false, false, false)
        );
    }

    #[test]
    fn mux_selects_inputs() {
        let mut builder = DigitalBuilder::new();
        let a = builder.input("a");
        let b = builder.input("b");
        let select = builder.input("sel");
        let output = builder.mux(a, b, select);
        builder.output("y", output);
        let mut circuit = builder.build();
        circuit.set_input("a", true).expect("a");
        circuit.set_input("b", false).expect("b");
        circuit.set_input("sel", false).expect("sel");
        circuit.settle().expect("settle");
        assert!(circuit.output("y").expect("y"));
        circuit.set_input("sel", true).expect("sel");
        circuit.settle().expect("settle");
        assert!(!circuit.output("y").expect("y"));
    }

    #[test]
    fn dff_captures_on_tick_and_holds() {
        let mut builder = DigitalBuilder::new();
        let data = builder.input("d");
        let output = builder.dff(data);
        builder.output("q", output);
        let mut circuit = builder.build();
        circuit.set_input("d", true).expect("d");
        circuit.tick().expect("tick");
        assert!(circuit.output("q").expect("q"));
        circuit.set_input("d", false).expect("d");
        assert!(circuit.output("q").expect("q"));
        circuit.tick().expect("tick");
        assert!(!circuit.output("q").expect("q"));
    }

    #[test]
    fn reset_clears_flip_flops() {
        let mut builder = DigitalBuilder::new();
        let data = builder.input("d");
        let output = builder.dff(data);
        builder.output("q", output);
        let mut circuit = builder.build();
        circuit.set_input("d", true).expect("d");
        circuit.tick().expect("tick");
        circuit.reset().expect("reset");
        assert!(!circuit.output("q").expect("q"));
    }

    #[test]
    fn unknown_input_is_rejected() {
        let builder = DigitalBuilder::new();
        let mut circuit = builder.build();
        let error = circuit.set_input("missing", true).expect_err("error");
        assert!(matches!(error, SimError::InvalidNetlist(_)));
    }

    #[test]
    fn combinational_loop_reports_failure() {
        let mut builder = DigitalBuilder::new();
        let net = builder.net();
        builder.not_into(net, net);
        builder.output("y", net);
        let mut circuit = builder.build();
        let error = circuit.settle().expect_err("loop");
        assert!(matches!(error, SimError::ConvergenceFailure { .. }));
    }
}
