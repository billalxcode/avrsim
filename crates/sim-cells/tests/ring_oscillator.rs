//! Analog ring oscillator verification.

use sim_analysis::{TransientConfig, TransientSample, transient};
use sim_cells::ring_oscillator;
use sim_core::{Circuit, DeviceKind, NodeId};
use std::collections::BTreeMap;

#[test]
fn ring_oscillator_oscillates() {
    let (circuit, first) = powered_ring();
    let config = TransientConfig {
        dt: 1e-8,
        steps: 2000,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: vec![(first, 5.0)],
    };
    let samples = transient(&circuit, &config).expect("transient");
    assert!(count_crossings(&samples, first) > 4);
}

fn powered_ring() -> (Circuit, NodeId) {
    let cell = ring_oscillator().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    let ground = circuit.ground();
    let first = add_load(&mut circuit, "n1", ground);
    add_load(&mut circuit, "n2", ground);
    add_load(&mut circuit, "n3", ground);
    (circuit, first)
}

fn add_load(circuit: &mut Circuit, name: &str, ground: NodeId) -> NodeId {
    let node = circuit.node_by_name(name).expect(name).id();
    let mut parameters = BTreeMap::new();
    parameters.insert("capacitance".to_string(), 1e-9);
    circuit
        .add_device(
            format!("c_{name}"),
            DeviceKind::Capacitor,
            vec![node, ground],
            parameters,
        )
        .expect("capacitor");
    node
}

fn count_crossings(samples: &[TransientSample], node: NodeId) -> usize {
    let mut crossings = 0;
    let mut previous = true;
    for sample in samples {
        let high = sample.solution.voltage(node).expect("voltage") > 2.5;
        if high != previous {
            crossings += 1;
            previous = high;
        }
    }
    crossings
}
