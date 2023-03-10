
// Maximum number of device ID bits used by the IOMMU.
const DEVICE_ID_BITS: usize = 24;
// Number of bits used to index into the leaf table.
const LEAF_INDEX_BITS: usize = 6;
// Number of bits used to index into intermediate tables.
const NON_LEAF_INDEX_BITS: usize = 9;

/// The device ID. Used to index into the device directory table. For PCI devices behind an IOMMU
/// this is equivalent to the requester ID of the PCI device (i.e. the bits of the B/D/F).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceId(u32);

impl DeviceId {
    /// Creates a new `DeviceId` from the raw `val`.
    pub fn new(val: u32) -> Option<DeviceId> {
        if (val & !((1 << DEVICE_ID_BITS) - 1)) == 0 {
            Some(Self(val))
        } else {
            None
        }
    }

    /// Returns the raw bits of this `DeviceId`.
    pub fn bits(&self) -> u32 {
        self.0
    }

    // Returns the bits from this `DeviceId` used to index at `level`.
    fn level_index_bits(&self, level: usize) -> usize {
        if level == 0 {
            (self.0 as usize) & ((1 << LEAF_INDEX_BITS) - 1)
        } else {
            let shift = LEAF_INDEX_BITS + NON_LEAF_INDEX_BITS * (level - 1);
            ((self.0 as usize) >> shift) & ((1 << NON_LEAF_INDEX_BITS) - 1)
        }
    }
}

