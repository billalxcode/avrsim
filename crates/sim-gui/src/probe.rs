//! Read-only probe model, bounded sample buffers, and decimation helpers.

use crate::model::{ComponentId, WireId};
use crate::sim::Sample;
use eframe::egui;
use std::collections::VecDeque;

/// Maximum number of analog series in one plot.
pub const MAX_SERIES: usize = 8;
/// Default probe sampling interval in CPU cycles.
pub const DEFAULT_SAMPLE_EVERY: u64 = 100;
const BUFFER_SECONDS: f64 = 10.0;
const CLOCK_HZ: f64 = 16_000_000.0;
const MAX_SAMPLES: usize = 100_000;

/// Read-only source selected for a probe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeSource {
    Pin {
        component: ComponentId,
        pin: &'static str,
    },
    /// Reserved for solver-backed net probes.
    #[allow(dead_code)]
    Net { wire: WireId },
    /// Branch current probe.
    Branch {
        component: ComponentId,
        pin: &'static str,
    },
}

/// Per-probe sampling interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sampling {
    pub every_cycles: u64,
}

impl Default for Sampling {
    fn default() -> Self {
        Self {
            every_cycles: DEFAULT_SAMPLE_EVERY,
        }
    }
}

impl Sampling {
    /// Returns the bounded buffer capacity for a 10 second window.
    #[must_use]
    pub fn capacity(self) -> usize {
        let every = self.every_cycles.max(1) as f64;
        let total = (BUFFER_SECONDS * CLOCK_HZ / every) as usize;
        total.clamp(2, MAX_SAMPLES)
    }
}

/// Probe identity and rendering metadata.
#[derive(Clone, Debug)]
pub struct ProbeSpec {
    pub label: String,
    pub unit: &'static str,
    pub color: egui::Color32,
    pub source: ProbeSource,
    pub sampling: Sampling,
}

/// One sampled value on the simulation timeline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProbePoint {
    pub cycle: u64,
    pub value: f64,
}

/// Bounded probe sample history.
#[derive(Clone, Debug)]
pub struct ProbeBuffer {
    spec: ProbeSpec,
    samples: VecDeque<ProbePoint>,
    capacity: usize,
}

impl ProbeBuffer {
    /// Creates an empty buffer with the capacity derived from its sampling.
    #[must_use]
    pub fn new(spec: ProbeSpec) -> Self {
        let capacity = spec.sampling.capacity();
        Self {
            spec,
            samples: VecDeque::new(),
            capacity,
        }
    }

    /// Appends a point, dropping the oldest beyond the capacity.
    pub fn push(&mut self, point: ProbePoint) {
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(point);
    }

    /// Removes all samples.
    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Returns probe metadata.
    #[must_use]
    pub fn spec(&self) -> &ProbeSpec {
        &self.spec
    }

    /// Returns samples in time order.
    #[must_use]
    pub fn samples(&self) -> &VecDeque<ProbePoint> {
        &self.samples
    }

    /// Returns the newest sample.
    #[must_use]
    pub fn latest(&self) -> Option<ProbePoint> {
        self.samples.back().copied()
    }
}

/// Read-only provider used by analysis plugins and the GUI adapter.
pub trait AnalysisProvider {
    /// Returns a pin voltage in volts.
    fn pin_voltage(&self, component: ComponentId, pin: &str, cycle: u64) -> Option<f64>;
    /// Returns a net voltage in volts.
    fn node_voltage(&self, wire: WireId, cycle: u64) -> Option<f64>;
    /// Returns a branch current in amperes.
    fn branch_current(&self, component: ComponentId, pin: &str, cycle: u64) -> Option<f64>;
}

/// Adapter that exposes the current PB5 sample through the read-only API.
#[derive(Clone, Copy, Debug)]
pub struct SampleProvider {
    pub sample: Sample,
}

impl AnalysisProvider for SampleProvider {
    fn pin_voltage(&self, _component: ComponentId, pin: &str, cycle: u64) -> Option<f64> {
        if cycle != self.sample.cycle || pin != "PB5" {
            return None;
        }
        Some(f64::from(self.sample.pb5_millivolts) / 1000.0)
    }

    fn node_voltage(&self, _wire: WireId, _cycle: u64) -> Option<f64> {
        None
    }

    fn branch_current(&self, _component: ComponentId, pin: &str, cycle: u64) -> Option<f64> {
        if cycle != self.sample.cycle || pin != "LED" {
            return None;
        }
        Some(f64::from(self.sample.led_microamps) / 1_000_000.0)
    }
}

fn source_value(provider: &dyn AnalysisProvider, source: ProbeSource, cycle: u64) -> Option<f64> {
    match source {
        ProbeSource::Pin { component, pin } => provider.pin_voltage(component, pin, cycle),
        ProbeSource::Net { wire } => provider.node_voltage(wire, cycle),
        ProbeSource::Branch { component, pin } => provider.branch_current(component, pin, cycle),
    }
}

/// Set of at most eight probes.
#[derive(Clone, Debug, Default)]
pub struct ProbeSet {
    probes: Vec<ProbeBuffer>,
}

impl ProbeSet {
    /// Adds a probe when the series limit is not reached.
    pub fn add(&mut self, spec: ProbeSpec) -> bool {
        if self.probes.len() >= MAX_SERIES {
            return false;
        }
        self.probes.push(ProbeBuffer::new(spec));
        true
    }

    /// Returns all probes.
    #[must_use]
    pub fn probes(&self) -> &[ProbeBuffer] {
        &self.probes
    }

    /// Returns true when a source already has a probe.
    #[must_use]
    pub fn contains(&self, source: ProbeSource) -> bool {
        self.probes.iter().any(|probe| probe.spec.source == source)
    }

    /// Clears sample history but keeps probe definitions.
    pub fn clear_samples(&mut self) {
        for probe in &mut self.probes {
            probe.clear();
        }
    }

    /// Appends one sample to every probe from a read-only provider.
    pub fn sample_from(&mut self, provider: &dyn AnalysisProvider, cycle: u64) {
        for probe in &mut self.probes {
            let every = probe.spec.sampling.every_cycles.max(1);
            if cycle % every != 0 {
                continue;
            }
            let Some(value) = source_value(provider, probe.spec.source, cycle) else {
                continue;
            };
            probe.push(ProbePoint { cycle, value });
        }
    }

    /// Returns the full cycle span across all probes.
    #[must_use]
    pub fn span(&self) -> (u64, u64) {
        let start = self
            .probes
            .iter()
            .filter_map(|probe| probe.samples().front().map(|point| point.cycle))
            .min()
            .unwrap_or(0);
        let end = self
            .probes
            .iter()
            .filter_map(|probe| probe.samples().back().map(|point| point.cycle))
            .max()
            .unwrap_or(start);
        if end <= start {
            (start, start + 1)
        } else {
            (start, end)
        }
    }
}

/// Visible viewport controls for an analog plot.
#[derive(Clone, Copy, Debug)]
pub struct PlotView {
    pub zoom: f32,
    pub pan: f64,
    pub autoscale: bool,
    pub legend: bool,
    pub cursor: Option<u64>,
}

impl Default for PlotView {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: 0.0,
            autoscale: true,
            legend: true,
            cursor: None,
        }
    }
}

/// Returns the visible cycle window after zoom and pan.
#[must_use]
pub fn window(span: (u64, u64), view: PlotView) -> (u64, u64) {
    let (start, end) = span;
    let width = end.saturating_sub(start).max(1);
    let zoom = f64::from(view.zoom.clamp(0.25, 8.0));
    let visible = ((width as f64 / zoom) as u64).max(1);
    let center = start as i64 + width as i64 / 2;
    let shift = (view.pan * visible as f64) as i64;
    let center = (center + shift).max(0) as u64;
    let begin = center.saturating_sub(visible / 2);
    (begin, begin + visible)
}

/// Returns the value range for probes with one unit.
#[must_use]
pub fn value_range_for(
    probes: &[ProbeBuffer],
    window: (u64, u64),
    unit: &str,
) -> Option<(f64, f64)> {
    range_for(probes, window, |probe| probe.spec().unit == unit)
}

fn range_for(
    probes: &[ProbeBuffer],
    window: (u64, u64),
    include: impl Fn(&ProbeBuffer) -> bool,
) -> Option<(f64, f64)> {
    probes
        .iter()
        .filter(|probe| include(probe))
        .filter_map(|probe| probe_range(probe, window))
        .reduce(|(min_a, max_a), (min_b, max_b)| (min_a.min(min_b), max_a.max(max_b)))
        .map(|(min, max)| padded_range(min, max))
}

fn probe_range(probe: &ProbeBuffer, window: (u64, u64)) -> Option<(f64, f64)> {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut found = false;
    for point in probe.samples() {
        if point.cycle < window.0 || point.cycle > window.1 {
            continue;
        }
        min = min.min(point.value);
        max = max.max(point.value);
        found = true;
    }
    found.then_some((min, max))
}

fn padded_range(min: f64, max: f64) -> (f64, f64) {
    if (max - min).abs() < f64::EPSILON {
        return (min - 0.5, max + 0.5);
    }
    (min, max)
}

/// Decimates samples into min/max pairs, preserving extremes.
#[must_use]
pub fn decimate(samples: &VecDeque<ProbePoint>, max_points: usize) -> Vec<ProbePoint> {
    let bucket_size = samples.len().div_ceil(max_points.max(2));
    let mut output = Vec::new();
    let mut bucket = Vec::new();
    for point in samples {
        bucket.push(*point);
        if bucket.len() == bucket_size {
            push_min_max(&mut output, &bucket);
            bucket.clear();
        }
    }
    if !bucket.is_empty() {
        push_min_max(&mut output, &bucket);
    }
    output
}

fn push_min_max(output: &mut Vec<ProbePoint>, bucket: &[ProbePoint]) {
    let Some(min) = bucket.iter().min_by(|a, b| a.value.total_cmp(&b.value)) else {
        return;
    };
    let Some(max) = bucket.iter().max_by(|a, b| a.value.total_cmp(&b.value)) else {
        return;
    };
    let (first, second) = if min.cycle <= max.cycle {
        (min, max)
    } else {
        (max, min)
    };
    output.push(*first);
    if first.cycle != second.cycle {
        output.push(*second);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AnalysisProvider, MAX_SERIES, PlotView, ProbeBuffer, ProbePoint, ProbeSet, ProbeSource,
        ProbeSpec, Sampling, decimate, window,
    };
    use crate::model::{ComponentId, WireId};
    use crate::sim::Sample;
    use std::collections::VecDeque;

    fn spec(every_cycles: u64) -> ProbeSpec {
        ProbeSpec {
            label: "PB5".to_string(),
            unit: "V",
            color: eframe::egui::Color32::WHITE,
            source: ProbeSource::Pin {
                component: ComponentId(1),
                pin: "PB5",
            },
            sampling: Sampling { every_cycles },
        }
    }

    fn point(cycle: u64, value: f64) -> ProbePoint {
        ProbePoint { cycle, value }
    }

    #[test]
    fn buffer_drops_oldest_samples() {
        let mut buffer = ProbeBuffer::new(spec(10_000_000));
        for cycle in 0..20 {
            buffer.push(point(cycle, cycle as f64));
        }
        assert_eq!(buffer.samples().len(), 16);
        assert_eq!(buffer.latest().expect("latest").cycle, 19);
    }

    #[test]
    fn decimation_preserves_extremes() {
        let samples: VecDeque<_> = (0..100).map(|cycle| point(cycle, cycle as f64)).collect();
        let output = decimate(&samples, 10);
        assert!(output.len() <= 20);
        assert!(output.iter().any(|point| point.value == 0.0));
        assert!(output.iter().any(|point| point.value == 99.0));
    }

    #[test]
    fn probe_set_limits_series() {
        let mut set = ProbeSet::default();
        for _ in 0..MAX_SERIES {
            assert!(set.add(spec(100)));
        }
        assert!(!set.add(spec(100)));
    }

    struct CycleProvider;

    impl AnalysisProvider for CycleProvider {
        fn pin_voltage(&self, _component: ComponentId, _pin: &str, cycle: u64) -> Option<f64> {
            Some(cycle as f64)
        }
        fn node_voltage(&self, _wire: WireId, _cycle: u64) -> Option<f64> {
            None
        }
        fn branch_current(&self, _component: ComponentId, _pin: &str, _cycle: u64) -> Option<f64> {
            None
        }
    }

    #[test]
    fn probe_set_samples_from_provider() {
        let mut set = ProbeSet::default();
        assert!(set.add(spec(100)));
        set.sample_from(&CycleProvider, 100);
        assert_eq!(set.probes()[0].latest(), Some(point(100, 100.0)));
    }

    #[test]
    fn probe_set_filters_per_probe_sampling() {
        let mut set = ProbeSet::default();
        assert!(set.add(spec(10)));
        assert!(set.add(spec(20)));
        set.sample_from(&CycleProvider, 10);
        set.sample_from(&CycleProvider, 20);
        assert_eq!(set.probes()[0].samples().len(), 2);
        assert_eq!(set.probes()[1].samples().len(), 1);
    }

    #[test]
    fn sample_provider_exposes_pb5_voltage() {
        let sample = Sample {
            cycle: 7,
            pb5_high: true,
            pb5_millivolts: 4200,
            led_on: true,
            led_microamps: 12_000,
        };
        let provider = super::SampleProvider { sample };
        assert_eq!(provider.pin_voltage(ComponentId(1), "PB5", 7), Some(4.2));
        assert_eq!(provider.pin_voltage(ComponentId(1), "PB5", 8), None);
        assert_eq!(
            provider.branch_current(ComponentId(1), "LED", 7),
            Some(0.012)
        );
    }

    #[test]
    fn plot_window_respects_zoom_and_pan() {
        assert_eq!(window((0, 100), PlotView::default()), (0, 100));
        let zoomed = window(
            (0, 100),
            PlotView {
                zoom: 2.0,
                ..PlotView::default()
            },
        );
        assert_eq!(zoomed, (25, 75));
        let panned = window(
            (0, 100),
            PlotView {
                zoom: 2.0,
                pan: 0.2,
                ..PlotView::default()
            },
        );
        assert!(panned.0 > zoomed.0);
    }

    #[test]
    fn value_range_tracks_visible_samples() {
        let mut buffer = ProbeBuffer::new(spec(100));
        buffer.push(point(10, 1.0));
        buffer.push(point(20, 3.0));
        buffer.push(point(30, 2.0));
        assert_eq!(
            super::value_range_for(std::slice::from_ref(&buffer), (0, 25), "V"),
            Some((1.0, 3.0))
        );
        assert_eq!(
            super::value_range_for(std::slice::from_ref(&buffer), (25, 40), "V"),
            Some((1.5, 2.5))
        );
    }
}
