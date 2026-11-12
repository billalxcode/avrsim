//! Shared Newton iteration helpers.

use sim_core::Solution;
use sim_core::tolerance::approx_eq;

/// Newton settings for transient stepping.
#[derive(Clone, Copy, Debug)]
pub struct NewtonConfig {
    /// Maximum iterations per step.
    pub max_iterations: usize,
    /// Relative convergence tolerance.
    pub rel_tol: f64,
    /// Absolute convergence tolerance.
    pub abs_tol: f64,
    /// Maximum node-voltage change per iteration.
    pub max_step: f64,
}

impl Default for NewtonConfig {
    fn default() -> Self {
        Self {
            max_iterations: 50,
            rel_tol: 1e-6,
            abs_tol: 1e-9,
            max_step: 0.5,
        }
    }
}

pub(crate) fn damp_solution(old: &Solution, new: &Solution, max_step: f64) -> Solution {
    let delta = max_node_delta(old, new);
    if delta <= max_step || max_step <= 0.0 {
        return new.clone();
    }
    let alpha = max_step / delta;
    Solution {
        node_voltages: interpolate(&old.node_voltages, &new.node_voltages, alpha),
        branch_currents: interpolate(&old.branch_currents, &new.branch_currents, alpha),
    }
}

pub(crate) fn max_node_delta(old: &Solution, new: &Solution) -> f64 {
    old.node_voltages
        .iter()
        .zip(&new.node_voltages)
        .map(|(first, second)| (second - first).abs())
        .fold(0.0, f64::max)
}

pub(crate) fn solution_close(
    first: &Solution,
    second: &Solution,
    rel_tol: f64,
    abs_tol: f64,
) -> bool {
    let voltages = first
        .node_voltages
        .iter()
        .zip(&second.node_voltages)
        .all(|(left, right)| approx_eq(*left, *right, rel_tol, abs_tol));
    let currents = first
        .branch_currents
        .iter()
        .zip(&second.branch_currents)
        .all(|(left, right)| approx_eq(*left, *right, rel_tol, abs_tol));
    voltages && currents
}

fn interpolate(old: &[f64], new: &[f64], alpha: f64) -> Vec<f64> {
    old.iter()
        .zip(new)
        .map(|(first, second)| first + alpha * (second - first))
        .collect()
}
