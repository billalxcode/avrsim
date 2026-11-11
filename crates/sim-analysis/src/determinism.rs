//! Determinism checks for Linux CI.

#[cfg(test)]
mod tests {
    use crate::test_support::rc_circuit;
    use crate::{DcConfig, TransientConfig, dc_operating_point, transient};

    #[test]
    fn dc_operating_point_is_deterministic() {
        let (circuit, output) = rc_circuit();
        let first = dc_operating_point(&circuit, &DcConfig::default()).expect("first");
        let second = dc_operating_point(&circuit, &DcConfig::default()).expect("second");
        let left = first.solution.voltage(output).expect("left");
        let right = second.solution.voltage(output).expect("right");
        assert_eq!(left.to_bits(), right.to_bits());
    }

    #[test]
    fn transient_is_deterministic() {
        let (circuit, output) = rc_circuit();
        let config = TransientConfig {
            dt: 1e-6,
            steps: 100,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        let first = transient(&circuit, &config).expect("first");
        let second = transient(&circuit, &config).expect("second");
        assert_eq!(first.len(), second.len());
        for (left, right) in first.iter().zip(second.iter()) {
            assert_eq!(left.time.as_ps(), right.time.as_ps());
            let left_voltage = left.solution.voltage(output).expect("left");
            let right_voltage = right.solution.voltage(output).expect("right");
            assert_eq!(left_voltage.to_bits(), right_voltage.to_bits());
        }
    }

    #[test]
    fn repeated_dc_sweep_is_deterministic() {
        let (circuit, _) = rc_circuit();
        assert_eq!(sweep_values(&circuit), sweep_values(&circuit));
    }

    #[test]
    fn circuit_snapshot_replays_transient() {
        let (circuit, output) = rc_circuit();
        let snapshot = circuit.snapshot();
        let config = TransientConfig {
            dt: 1e-6,
            steps: 20,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        let first = transient(&circuit, &config).expect("first");
        let second = transient(&snapshot, &config).expect("snapshot");
        assert_eq!(first.len(), second.len());
        for (left, right) in first.iter().zip(second.iter()) {
            assert_eq!(left.time.as_ps(), right.time.as_ps());
            let left = left.solution.voltage(output).expect("left");
            let right = right.solution.voltage(output).expect("right");
            assert_eq!(left.to_bits(), right.to_bits());
        }
    }

    fn sweep_values(circuit: &sim_core::Circuit) -> Vec<f64> {
        crate::dc_sweep(
            circuit,
            "v1",
            "voltage",
            0.0,
            2.0,
            0.5,
            &DcConfig::default(),
        )
        .expect("sweep")
        .iter()
        .map(|point| point.sweep)
        .collect()
    }
}
