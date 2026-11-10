//! Stable identifiers for netlist entities.

macro_rules! define_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u32);

        impl $name {
            /// Creates an identifier from its raw value.
            #[must_use]
            pub const fn new(raw: u32) -> Self {
                Self(raw)
            }

            /// Returns the raw identifier value.
            #[must_use]
            pub const fn raw(self) -> u32 {
                self.0
            }
        }
    };
}

define_id!(NodeId, "Identifier for a circuit node.");
define_id!(
    BranchId,
    "Identifier for a voltage-source or inductor branch."
);
define_id!(DeviceId, "Identifier for a device in a netlist.");

impl NodeId {
    /// The ground node. It is always node zero.
    pub const GROUND: Self = Self(0);
}

#[cfg(test)]
mod tests {
    use super::{BranchId, DeviceId, NodeId};

    #[test]
    fn identifiers_round_trip() {
        assert_eq!(NodeId::new(7).raw(), 7);
        assert_eq!(BranchId::new(11).raw(), 11);
        assert_eq!(DeviceId::new(13).raw(), 13);
    }

    #[test]
    fn ground_is_zero() {
        assert_eq!(NodeId::GROUND.raw(), 0);
        assert_eq!(NodeId::GROUND, NodeId::new(0));
    }

    #[test]
    fn identifiers_are_ordered() {
        assert!(NodeId::new(1) < NodeId::new(2));
        assert!(DeviceId::new(10) > DeviceId::new(9));
    }
}
