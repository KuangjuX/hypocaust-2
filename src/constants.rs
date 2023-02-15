pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 4;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;

pub mod layout {
    pub const MEMORY_START: usize = 0x80000000;
    pub const MEMORY_END: usize = 0x88000000;
}