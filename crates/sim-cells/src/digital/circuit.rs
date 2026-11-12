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
    Buf(NetId, NetId),
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

    /// Adds a buffer that drives an existing net.
    pub fn buf_into(&mut self, input: NetId, output: NetId) {
        self.gates.push(Gate::Buf(input, output));
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
        let components = build_components(self.values.len(), &self.gates, &self.dffs);
        let dirty = vec![true; components.gates.len()];
        DigitalCircuit {
            values: self.values,
            gates: self.gates,
            dffs: self.dffs,
            inputs: self.inputs,
            outputs: self.outputs,
            component_gates: components.gates,
            net_component: components.nets,
            dff_component: components.dffs,
            dirty,
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
    component_gates: Vec<Vec<usize>>,
    net_component: Vec<Option<usize>>,
    dff_component: Vec<Option<usize>>,
    dirty: Vec<bool>,
}

impl DigitalCircuit {
    /// Sets a named input value without settling.
    pub fn set_input(&mut self, name: &str, value: bool) -> Result<(), SimError> {
        let net = self
            .inputs
            .get(name)
            .ok_or_else(|| SimError::InvalidNetlist(format!("unknown input: {name}")))?;
        if self.values[net.raw()] == value {
            return Ok(());
        }
        self.values[net.raw()] = value;
        if let Some(component) = self.net_component[net.raw()] {
            self.dirty[component] = true;
        }
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
        for component in 0..self.component_gates.len() {
            if !self.dirty[component] {
                continue;
            }
            self.settle_component(component)?;
            self.dirty[component] = false;
        }
        Ok(())
    }

    fn settle_component(&mut self, component: usize) -> Result<(), SimError> {
        for iteration in 0..MAX_SETTLE_ITERATIONS {
            let mut changed = false;
            for index in 0..self.component_gates[component].len() {
                let gate = self.component_gates[component][index];
                changed |= evaluate(self.gates[gate], &mut self.values);
            }
            if !changed {
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
        self.dirty.fill(true);
        self.settle()
    }

    fn dff_inputs(&self) -> Vec<bool> {
        self.dffs
            .iter()
            .map(|dff| self.values[dff.data.raw()])
            .collect()
    }

    fn update_dffs(&mut self, next: &[bool]) {
        for (index, (dff, value)) in self.dffs.iter().zip(next).enumerate() {
            if self.values[dff.output.raw()] == *value {
                continue;
            }
            self.values[dff.output.raw()] = *value;
            if let Some(component) = self.dff_component[index] {
                self.dirty[component] = true;
            }
        }
    }
}

struct CircuitComponents {
    gates: Vec<Vec<usize>>,
    nets: Vec<Option<usize>>,
    dffs: Vec<Option<usize>>,
}

struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
        }
    }

    fn find(&mut self, mut index: usize) -> usize {
        while self.parent[index] != index {
            self.parent[index] = self.parent[self.parent[index]];
            index = self.parent[index];
        }
        index
    }

    fn union(&mut self, first: usize, second: usize) {
        let first = self.find(first);
        let second = self.find(second);
        if first != second {
            self.parent[second] = first;
        }
    }
}

fn build_components(values: usize, gates: &[Gate], dffs: &[Dff]) -> CircuitComponents {
    let mut union = UnionFind::new(values);
    for &gate in gates {
        union_gate(&mut union, gate);
    }
    for dff in dffs {
        union.union(dff.data.raw(), dff.output.raw());
    }
    let (component_gates, roots) = assign_gate_components(&mut union, gates);
    let nets = assign_net_components(&mut union, &roots);
    let dffs = dffs.iter().map(|dff| nets[dff.output.raw()]).collect();
    CircuitComponents {
        gates: component_gates,
        nets,
        dffs,
    }
}

fn assign_gate_components(
    union: &mut UnionFind,
    gates: &[Gate],
) -> (Vec<Vec<usize>>, BTreeMap<usize, usize>) {
    let mut roots = BTreeMap::new();
    let mut components: Vec<Vec<usize>> = Vec::new();
    for (index, &gate) in gates.iter().enumerate() {
        let root = union.find(gate_output(gate).raw());
        let component = *roots.entry(root).or_insert_with(|| {
            components.push(Vec::new());
            components.len() - 1
        });
        components[component].push(index);
    }
    (components, roots)
}

fn assign_net_components(
    union: &mut UnionFind,
    roots: &BTreeMap<usize, usize>,
) -> Vec<Option<usize>> {
    (0..union.parent.len())
        .map(|net| roots.get(&union.find(net)).copied())
        .collect()
}

fn union_gate(union: &mut UnionFind, gate: Gate) {
    match gate {
        Gate::Const(_, _) => {}
        Gate::Buf(first, output) | Gate::Not(first, output) => {
            union.union(first.raw(), output.raw());
        }
        Gate::And(first, second, output)
        | Gate::Or(first, second, output)
        | Gate::Xor(first, second, output)
        | Gate::Nand(first, second, output)
        | Gate::Nor(first, second, output) => {
            union.union(first.raw(), second.raw());
            union.union(second.raw(), output.raw());
        }
        Gate::Mux {
            first,
            second,
            select,
            output,
        } => {
            union.union(first.raw(), second.raw());
            union.union(first.raw(), select.raw());
            union.union(select.raw(), output.raw());
        }
    }
}

fn gate_output(gate: Gate) -> NetId {
    match gate {
        Gate::Const(_, output)
        | Gate::Buf(_, output)
        | Gate::Not(_, output)
        | Gate::And(_, _, output)
        | Gate::Or(_, _, output)
        | Gate::Xor(_, _, output)
        | Gate::Nand(_, _, output)
        | Gate::Nor(_, _, output) => output,
        Gate::Mux { output, .. } => output,
    }
}

fn evaluate(gate: Gate, values: &mut [bool]) -> bool {
    match gate {
        Gate::Const(value, output) => set_value(values, output, value),
        Gate::Buf(input, output) => set_value(values, output, values[input.raw()]),
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
    fn buffer_follows_input() {
        let mut builder = DigitalBuilder::new();
        let input = builder.input("a");
        let output = builder.net();
        builder.buf_into(input, output);
        builder.output("y", output);
        let mut circuit = builder.build();
        circuit.set_input("a", true).expect("a");
        circuit.settle().expect("settle");
        assert!(circuit.output("y").expect("y"));
        circuit.set_input("a", false).expect("a");
        circuit.settle().expect("settle");
        assert!(!circuit.output("y").expect("y"));
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
    fn independent_components_settle_after_each_change() {
        let mut builder = DigitalBuilder::new();
        let a = builder.input("a");
        let b = builder.input("b");
        let first = builder.not(a);
        let second = builder.not(b);
        builder.output("first", first);
        builder.output("second", second);
        let mut circuit = builder.build();
        circuit.set_input("a", true).expect("a");
        circuit.settle().expect("settle");
        assert!(!circuit.output("first").expect("first"));
        assert!(circuit.output("second").expect("second"));
        circuit.set_input("b", true).expect("b");
        circuit.settle().expect("settle");
        assert!(!circuit.output("second").expect("second"));
        circuit.set_input("a", false).expect("a");
        circuit.settle().expect("settle");
        assert!(circuit.output("first").expect("first"));
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
