//! Modified nodal analysis context.

use crate::error::SimError;
use crate::ids::{DeviceId, NodeId};
use crate::matrix::DenseMatrix;
use crate::netlist::Solution;
use std::sync::Arc;

/// MNA matrix, right-hand side, and node/branch mappings.
#[derive(Debug)]
pub struct MnaContext {
    matrix: DenseMatrix,
    rhs: Vec<f64>,
    node_map: Arc<Vec<Option<usize>>>,
    branch_map: Arc<Vec<Option<usize>>>,
    node_count: usize,
    branch_count: usize,
}

impl MnaContext {
    /// Creates an empty MNA system.
    pub fn new(
        node_count: usize,
        branch_count: usize,
        node_map: Vec<Option<usize>>,
        branch_map: Vec<Option<usize>>,
    ) -> Result<Self, SimError> {
        Self::from_shared(
            node_count,
            branch_count,
            Arc::new(node_map),
            Arc::new(branch_map),
        )
    }

    /// Creates an MNA system that shares precomputed mappings.
    pub fn from_shared(
        node_count: usize,
        branch_count: usize,
        node_map: Arc<Vec<Option<usize>>>,
        branch_map: Arc<Vec<Option<usize>>>,
    ) -> Result<Self, SimError> {
        let size = node_count + branch_count;
        Ok(Self {
            matrix: DenseMatrix::zeros(size, size)?,
            rhs: vec![0.0; size],
            node_map,
            branch_map,
            node_count,
            branch_count,
        })
    }

    /// Returns the matrix index for a node.
    #[must_use]
    pub fn node_index(&self, node: NodeId) -> Option<usize> {
        self.node_map.get(node.raw() as usize).copied().flatten()
    }

    /// Returns the matrix index for a device branch.
    #[must_use]
    pub fn branch_index(&self, device: DeviceId) -> Option<usize> {
        self.branch_map
            .get(device.raw() as usize)
            .copied()
            .flatten()
    }

    /// Adds a value to the matrix.
    pub fn add_matrix(&mut self, row: usize, col: usize, value: f64) -> Result<(), SimError> {
        self.matrix.add(row, col, value)
    }

    /// Adds a value when both indexes exist.
    pub fn add_matrix_opt(
        &mut self,
        row: Option<usize>,
        col: Option<usize>,
        value: f64,
    ) -> Result<(), SimError> {
        match (row, col) {
            (Some(row), Some(col)) => self.add_matrix(row, col, value),
            _ => Ok(()),
        }
    }

    /// Adds a value to the right-hand side.
    pub fn add_rhs(&mut self, row: usize, value: f64) -> Result<(), SimError> {
        let entry = self
            .rhs
            .get_mut(row)
            .ok_or(SimError::IndexOutOfBounds { row, col: 0 })?;
        *entry += value;
        Ok(())
    }

    /// Adds a value to the right-hand side when the row exists.
    pub fn add_rhs_opt(&mut self, row: Option<usize>, value: f64) -> Result<(), SimError> {
        match row {
            Some(row) => self.add_rhs(row, value),
            None => Ok(()),
        }
    }

    /// Returns the assembled matrix.
    #[must_use]
    pub fn matrix(&self) -> &DenseMatrix {
        &self.matrix
    }

    /// Returns the assembled right-hand side.
    #[must_use]
    pub fn rhs(&self) -> &[f64] {
        &self.rhs
    }

    /// Solves the system and splits node voltages from branch currents.
    pub fn solve(self) -> Result<Solution, SimError> {
        let values = self.matrix.solve_owned(&self.rhs)?;
        let mut node_voltages = vec![0.0; self.node_map.len()];
        for (raw, slot) in self.node_map.iter().enumerate() {
            if let Some(index) = slot {
                node_voltages[raw] = values[*index];
            }
        }
        let branch_currents = values;
        if branch_currents.len() != self.node_count + self.branch_count {
            return Err(SimError::Internal(
                "matrix solution size mismatch".to_string(),
            ));
        }
        Ok(Solution {
            node_voltages,
            branch_currents,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::MnaContext;
    use crate::ids::{DeviceId, NodeId};

    fn single_node_context() -> MnaContext {
        MnaContext::new(1, 0, vec![None, Some(0)], vec![]).expect("context")
    }

    #[test]
    fn ground_maps_to_none() {
        let context = single_node_context();
        assert_eq!(context.node_index(NodeId::GROUND), None);
        assert_eq!(context.node_index(NodeId::new(1)), Some(0));
    }

    #[test]
    fn missing_node_maps_to_none() {
        let context = single_node_context();
        assert_eq!(context.node_index(NodeId::new(9)), None);
    }

    #[test]
    fn branch_map_uses_absolute_index() {
        let context = MnaContext::new(1, 1, vec![None, Some(0)], vec![Some(1)]).expect("context");
        assert_eq!(context.branch_index(DeviceId::new(0)), Some(1));
    }

    #[test]
    fn solves_single_node_system() {
        let mut context = single_node_context();
        context.add_matrix(0, 0, 1.0).expect("matrix");
        context.add_rhs(0, 2.0).expect("rhs");
        let solution = context.solve().expect("solution");
        assert_eq!(solution.voltage(NodeId::new(1)), Some(2.0));
        assert_eq!(solution.voltage(NodeId::GROUND), Some(0.0));
    }

    #[test]
    fn solves_branch_system() {
        let mut context =
            MnaContext::new(1, 1, vec![None, Some(0)], vec![Some(1)]).expect("context");
        context.add_matrix(0, 0, 1.0).expect("node matrix");
        context.add_matrix(1, 1, 1.0).expect("matrix");
        context.add_rhs(1, 3.0).expect("rhs");
        let solution = context.solve().expect("solution");
        assert_eq!(solution.branch_current(1), Some(3.0));
    }
}
