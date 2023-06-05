//! Constants used in rCore for qemu

pub const CLOCK_FREQ: usize = 12500000;

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x2000),      // VIRT_TEST/RTC  in virt machine
    (0x1000_1000, 0x8000),      // VirtIO space
    (0x1000_0000, 0x1000),      // UART,
    (0x0200_0000, 0x1_0000),    // CLINT
    (0x0c00_0000, 0x400_0000),  // PLIC
    (0x3000_0000, 0x1000_0000), // PCI
];

pub const PHYS_MEM_BASE: usize = 0x8000_0000;
pub const PHYS_MEM_SIZE: usize = 0xC000_0000;
pub const KERN_BASE_ADDR: usize = 0x8020_0000;

pub const GUEST_DTB_ADDR: usize = 0x9000_0000;
pub const GUEST_BIN_ADDR: usize = 0x9020_0000;
pub const GUEST_BIN_SIZE: usize = 0x0800_0000;