//! Simulation error types.

use crate::ids::NodeId;
use std::fmt;

/// Errors returned by simulation primitives and analyses.
#[derive(Debug)]
pub enum SimError {
    /// The netlist is structurally invalid.
    InvalidNetlist(String),
    /// A node has no path to ground.
    FloatingNode(NodeId),
    /// A node has no DC path to ground.
    NoDcPath { node: NodeId },
    /// A matrix pivot is too small for a stable solve.
    SingularMatrix { row: usize },
    /// An iterative solver did not converge.
    ConvergenceFailure { iteration: usize },
    /// A device or analysis is not implemented.
    UnsupportedDevice(String),
    /// A matrix or vector has an unexpected length.
    DimensionMismatch { expected: usize, actual: usize },
    /// A matrix operation requires a square matrix.
    NotSquare { rows: usize, cols: usize },
    /// A matrix index is outside its bounds.
    IndexOutOfBounds { row: usize, col: usize },
    /// An I/O operation failed.
    Io(std::io::Error),
    /// A text or binary format could not be parsed.
    Parse(String),
    /// An internal invariant was violated.
    Internal(String),
}

impl fmt::Display for SimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNetlist(message) => write!(f, "invalid netlist: {message}"),
            Self::FloatingNode(node) => write!(f, "floating node: {}", node.raw()),
            Self::SingularMatrix { row } => write!(f, "singular matrix at row {row}"),
            Self::ConvergenceFailure { iteration } => {
                write!(f, "convergence failure after {iteration} iterations")
            }
            other => other.fmt_detail(f),
        }
    }
}

impl SimError {
    fn fmt_detail(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedDevice(name) => write!(f, "unsupported device: {name}"),
            Self::DimensionMismatch { expected, actual } => {
                write!(f, "dimension mismatch: expected {expected}, got {actual}")
            }
            Self::NotSquare { rows, cols } => {
                write!(f, "matrix is not square: {rows} x {cols}")
            }
            Self::IndexOutOfBounds { row, col } => {
                write!(f, "index out of bounds: ({row}, {col})")
            }
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Parse(message) => write!(f, "parse error: {message}"),
            Self::Internal(message) => write!(f, "internal error: {message}"),
            Self::NoDcPath { node } => write!(f, "no DC path for node {}", node.raw()),
            _ => Ok(()),
        }
    }
}

impl std::error::Error for SimError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SimError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
