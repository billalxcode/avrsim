//! Core simulation primitives: time, units, identifiers, matrices, and errors.
//!
//! This crate has no external dependencies.

#![forbid(unsafe_code)]

pub mod complex;
pub mod complex_matrix;
pub mod error;
pub mod ids;
pub mod matrix;
pub mod mna;
pub mod netlist;
pub mod sparse;
pub mod time;
pub mod tolerance;
pub mod units;

pub use complex::Complex64;
pub use complex_matrix::ComplexMatrix;
pub use error::SimError;
pub use ids::{BranchId, DeviceId, NodeId};
pub use matrix::DenseMatrix;
pub use mna::MnaContext;
pub use netlist::{
    Circuit, CircuitBuilder, Device, DeviceInstance, DeviceKind, Node, NodeKind, Solution,
    Subcircuit,
};
pub use sparse::{SparseMatrix, minimum_degree_order, solve_sparse};
pub use time::Time;
pub use units::{Capacitance, Current, Resistance, Voltage};
