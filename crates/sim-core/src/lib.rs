//! Core simulation primitives: time, units, identifiers, matrices, and errors.
//!
//! This crate has no external dependencies.

#![forbid(unsafe_code)]

pub mod complex;
pub mod error;
pub mod ids;
pub mod matrix;
pub mod netlist;
pub mod time;
pub mod tolerance;
pub mod units;

pub use complex::Complex64;
pub use error::SimError;
pub use ids::{BranchId, DeviceId, NodeId};
pub use matrix::DenseMatrix;
pub use netlist::{Circuit, CircuitBuilder, DeviceKind, Node, NodeKind, Subcircuit};
pub use time::Time;
pub use units::{Capacitance, Current, Resistance, Voltage};
