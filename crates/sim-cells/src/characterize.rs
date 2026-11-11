//! Inverter VTC characterization.

/// Voltage transfer curve metrics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VtcMetrics {
    /// Input low voltage.
    pub vil: f64,
    /// Input high voltage.
    pub vih: f64,
    /// Output low voltage.
    pub vol: f64,
    /// Output high voltage.
    pub voh: f64,
    /// High noise margin.
    pub nmh: f64,
    /// Low noise margin.
    pub nml: f64,
}

/// Computes noise margins from input/output samples.
///
/// Samples must be ordered by increasing input voltage.
#[must_use]
pub fn noise_margins(points: &[(f64, f64)]) -> Option<VtcMetrics> {
    let (voh, vol) = output_bounds(points)?;
    let high_threshold = vol + 0.9 * (voh - vol);
    let low_threshold = vol + 0.1 * (voh - vol);
    let vil = max_input(points, high_threshold)?;
    let vih = min_input(points, low_threshold)?;
    Some(VtcMetrics {
        vil,
        vih,
        vol,
        voh,
        nmh: voh - vih,
        nml: vil - vol,
    })
}

fn output_bounds(points: &[(f64, f64)]) -> Option<(f64, f64)> {
    if points.is_empty() {
        return None;
    }
    let mut high = f64::NEG_INFINITY;
    let mut low = f64::INFINITY;
    for &(_, output) in points {
        high = high.max(output);
        low = low.min(output);
    }
    Some((high, low))
}

fn max_input(points: &[(f64, f64)], threshold: f64) -> Option<f64> {
    points
        .iter()
        .filter(|(_, output)| *output >= threshold)
        .map(|(input, _)| *input)
        .fold(None, |current, input| match current {
            Some(value) => Some(value.max(input)),
            None => Some(input),
        })
}

fn min_input(points: &[(f64, f64)], threshold: f64) -> Option<f64> {
    points
        .iter()
        .filter(|(_, output)| *output <= threshold)
        .map(|(input, _)| *input)
        .fold(None, |current, input| match current {
            Some(value) => Some(value.min(input)),
            None => Some(input),
        })
}
