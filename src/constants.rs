pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 4;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;

pub mod layout {
    use super::PAGE_SIZE;

    pub const MEMORY_START: usize = 0x80000000;
    pub const MEMORY_END: usize = 0x88000000;

    /// 跳板页虚拟地址
    pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
    /// 上下文切换数据存储虚拟地址
    pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;
}