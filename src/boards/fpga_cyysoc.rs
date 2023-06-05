//! Constants used for CyySoC FPGA.

pub const CLOCK_FREQ: usize = 100000000;

pub const MMIO: &[(usize, usize)] = [
    (0x6010_0000, 0x1000),      // UART
    (0x6020_0000, 0x10_0000),   // ETH0
    (0x6030_0000, 0x10_0000),   // AXI_ETH_DMA
    (0x0200_0000, 0x1_0000),    // CLINT
    (0x0c00_0000, 0x400_0000),  // PLIC
    (0x6000_0000, 0x2000_0000), // AXI4
];

pub const PHYS_MEM_BASE: usize = 0x8000_0000;
pub const PHYS_MEM_SIZE: usize = 0x8000_0000;
pub const KERN_BASE_ADDR: usize = 0x8020_0000;

pub const GUEST_DTB_ADDR: usize = 0x9000_0000;
pub const GUEST_BIN_ADDR: usize = 0x9020_0000;
pub const GUEST_BIN_SIZE: usize = 0x0800_0000;
