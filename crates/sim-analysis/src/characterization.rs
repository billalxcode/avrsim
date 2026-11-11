//! Switching timing measurements from transient waveforms.

use crate::transient::{TransientConfig, TransientSample, transient};
use sim_core::{Circuit, NodeId, SimError};

/// Switching timing results in seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchingTiming {
    /// Output rise time from 10 to 90 percent.
    pub rise_time: f64,
    /// Output fall time from 90 to 10 percent.
    pub fall_time: f64,
    /// Delay from input falling edge to output rising edge.
    pub delay_rise: f64,
    /// Delay from input rising edge to output falling edge.
    pub delay_fall: f64,
}

#[derive(Clone, Copy)]
struct Levels {
    low10: f64,
    middle: f64,
    high90: f64,
}

#[derive(Clone, Copy)]
struct Crossing {
    time: f64,
    index: usize,
}

struct Edges {
    output_rise: f64,
    output_fall: f64,
    rise_start: f64,
    rise_end: f64,
    fall_start: f64,
    fall_end: f64,
}

/// Measures switching timing between an input and an output node.
pub fn measure_switching(
    circuit: &Circuit,
    input: NodeId,
    output: NodeId,
    config: &TransientConfig,
) -> Result<SwitchingTiming, SimError> {
    validate_nodes(circuit, input, output)?;
    let samples = transient(circuit, config)?;
    let input_wave = waveform(&samples, input);
    let output_wave = waveform(&samples, output);
    let levels = output_levels(&output_wave)?;
    let edges = locate_edges(&output_wave, levels)?;
    build_timing(&edges, &input_wave, levels.middle)
}

fn validate_nodes(circuit: &Circuit, input: NodeId, output: NodeId) -> Result<(), SimError> {
    if input == output {
        return Err(SimError::InvalidNetlist(
            "input and output nodes must differ".to_string(),
        ));
    }
    if circuit.node(input).is_none() || circuit.node(output).is_none() {
        return Err(SimError::InvalidNetlist(
            "switching node is missing".to_string(),
        ));
    }
    Ok(())
}

fn waveform(samples: &[TransientSample], node: NodeId) -> Vec<(f64, f64)> {
    samples
        .iter()
        .filter_map(|sample| {
            sample
                .solution
                .voltage(node)
                .map(|voltage| (sample.time.as_secs_f64(), voltage))
        })
        .collect()
}

fn output_levels(wave: &[(f64, f64)]) -> Result<Levels, SimError> {
    let (minimum, maximum) = wave_range(wave);
    let span = maximum - minimum;
    if !span.is_finite() || span <= 1e-12 {
        return Err(SimError::InvalidNetlist(
            "output waveform does not switch".to_string(),
        ));
    }
    Ok(Levels {
        low10: minimum + 0.1 * span,
        middle: minimum + 0.5 * span,
        high90: minimum + 0.9 * span,
    })
}

fn wave_range(wave: &[(f64, f64)]) -> (f64, f64) {
    let minimum = wave
        .iter()
        .map(|(_, value)| *value)
        .fold(f64::INFINITY, f64::min);
    let maximum = wave
        .iter()
        .map(|(_, value)| *value)
        .fold(f64::NEG_INFINITY, f64::max);
    (minimum, maximum)
}

fn locate_edges(output: &[(f64, f64)], levels: Levels) -> Result<Edges, SimError> {
    let output_rise = required(find_crossing(output, 0, levels.middle, true), "output rise")?;
    let output_fall = required(
        find_crossing(output, 0, levels.middle, false),
        "output fall",
    )?;
    let (rise_start, rise_end) = edge_pair(output, levels.low10, levels.high90, true)?;
    let (fall_start, fall_end) = edge_pair(output, levels.high90, levels.low10, false)?;
    Ok(Edges {
        output_rise: output_rise.time,
        output_fall: output_fall.time,
        rise_start: rise_start.time,
        rise_end: rise_end.time,
        fall_start: fall_start.time,
        fall_end: fall_end.time,
    })
}

fn edge_pair(
    wave: &[(f64, f64)],
    start_level: f64,
    end_level: f64,
    rising: bool,
) -> Result<(Crossing, Crossing), SimError> {
    let start = required(find_crossing(wave, 0, start_level, rising), "edge start")?;
    let end = required(
        find_crossing(wave, start.index, end_level, rising),
        "edge end",
    )?;
    Ok((start, end))
}

fn required(crossing: Option<Crossing>, name: &str) -> Result<Crossing, SimError> {
    crossing.ok_or_else(|| SimError::InvalidNetlist(format!("missing {name} crossing")))
}

fn find_crossing(wave: &[(f64, f64)], start: usize, level: f64, rising: bool) -> Option<Crossing> {
    for index in start..wave.len().saturating_sub(1) {
        let (first_time, first_value) = wave[index];
        let (second_time, second_value) = wave[index + 1];
        let crossed = if rising {
            first_value < level && second_value >= level
        } else {
            first_value > level && second_value <= level
        };
        if crossed {
            let ratio = (level - first_value) / (second_value - first_value);
            return Some(Crossing {
                time: first_time + ratio * (second_time - first_time),
                index,
            });
        }
    }
    None
}

fn build_timing(
    edges: &Edges,
    input: &[(f64, f64)],
    middle: f64,
) -> Result<SwitchingTiming, SimError> {
    let rise_input = previous_crossing(input, middle, edges.output_rise)
        .ok_or_else(|| SimError::InvalidNetlist("missing input edge before rise".to_string()))?;
    let fall_input = previous_crossing(input, middle, edges.output_fall)
        .ok_or_else(|| SimError::InvalidNetlist("missing input edge before fall".to_string()))?;
    Ok(SwitchingTiming {
        rise_time: edges.rise_end - edges.rise_start,
        fall_time: edges.fall_end - edges.fall_start,
        delay_rise: edges.output_rise - rise_input,
        delay_fall: edges.output_fall - fall_input,
    })
}

fn previous_crossing(wave: &[(f64, f64)], level: f64, limit: f64) -> Option<f64> {
    let mut previous = None;
    for index in 0..wave.len().saturating_sub(1) {
        let (first_time, first_value) = wave[index];
        let (second_time, second_value) = wave[index + 1];
        let crossed = (first_value < level && second_value >= level)
            || (first_value > level && second_value <= level);
        if crossed {
            let ratio = (level - first_value) / (second_value - first_value);
            let time = first_time + ratio * (second_time - first_time);
            if time < limit {
                previous = Some(time);
            }
        }
    }
    previous
}

#[cfg(test)]
mod tests {
    use super::{SwitchingTiming, measure_switching};
    use crate::test_support::{pulse_rc, rc_circuit};
    use crate::transient::TransientConfig;

    fn config() -> TransientConfig {
        TransientConfig {
            dt: 1e-8,
            steps: 6000,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        }
    }

    #[test]
    fn rc_switching_has_expected_timing() {
        let (circuit, input, output) = pulse_rc();
        let timing = measure_switching(&circuit, input, output, &config()).expect("timing");
        assert_timing(&timing);
    }

    fn assert_timing(timing: &SwitchingTiming) {
        assert!((1.5e-6..3.0e-6).contains(&timing.rise_time));
        assert!((1.5e-6..3.0e-6).contains(&timing.fall_time));
        assert!((0.3e-6..1.2e-6).contains(&timing.delay_rise));
        assert!((0.3e-6..1.2e-6).contains(&timing.delay_fall));
    }

    #[test]
    fn switching_rejects_same_node() {
        let (circuit, output) = rc_circuit();
        assert!(measure_switching(&circuit, output, output, &config()).is_err());
    }

    #[test]
    fn switching_rejects_flat_output() {
        let (circuit, output) = rc_circuit();
        let input = circuit.node_by_name("in").expect("in").id();
        assert!(measure_switching(&circuit, input, output, &config()).is_err());
    }
}
