pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 4;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;

pub use crate::board::CLOCK_FREQ;

pub mod layout {
    use super::PAGE_SIZE;

    pub const MEMORY_START: usize = 0x80000000;
    pub const MEMORY_END: usize = 0x88000000;

    /// 跳板页虚拟地址
    pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
    /// 上下文切换数据存储虚拟地址
    pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

    pub const GUEST_START_PA: usize = 0x8800_0000;
    pub const GUEST_START_VA: usize = 0x8000_0000;

    pub const GUEST_DEFAULT_SIZE: usize = 128 * 1024 * 1024;


    pub use crate::board::MMIO;
}

pub mod csr {
    pub enum VirtualzationMode {
        Host = 0,
        Guest = 1,
    }

    #[derive(PartialEq)]
    pub enum CpuMode {
        M = 0b11,
        S = 0b01,
        U = 0b00
    }

    pub enum PrevisorMode {
        U,
        HS,
        M,
        VU,
        VS
    }

    pub mod hedeleg {
        use core::arch::asm;

        pub const INST_ADDR_MISALIGN: usize = 1 << 0;
        pub const INST_ACCESSS_FAULT: usize = 1 << 1;
        pub const ILLEGAL_INST: usize = 1 << 2;
        pub const BREAKPOINT: usize = 1 << 3;
        pub const LOAD_ADDR_MISALIGNED: usize = 1 << 4;
        pub const LOAD_ACCESS_FAULT: usize = 1 << 5;
        pub const STORE_ADDR_MISALIGNED: usize = 1 << 6;
        pub const STORE_ACCESS_FAULT: usize = 1 << 7;
        pub const ENV_CALL_FROM_U_OR_VU: usize = 1 << 8;
        pub const ENV_CALL_FROM_HS: usize = 1 << 9;
        pub const ENV_CALL_FROM_VS: usize = 1 << 10;
        pub const ENV_CALL_FROM_M: usize = 1 << 11;
        pub const INST_PAGE_FAULT: usize = 1 << 12;
        pub const LOAD_PAGE_FAULT: usize = 1 << 13;
        pub const STORE_PAGE_FAULT: usize = 1 << 15;
        pub const INST_GUEST_PAGE_FAULT: usize = 1 << 20;
        pub const LOAD_GUEST_PAGE_FAULT: usize = 1 << 21;
        pub const VIRTUAL_INST: usize = 1 << 22;
        pub const STORE_GUEST_PAGE_FAULT: usize = 1 << 23;

        pub unsafe fn write(hedeleg: usize) {
            asm!(
                "csrw hedeleg, {}",
                in(reg) hedeleg
            )
        }
    }

    pub mod hideleg {
        use core::arch::asm;
        pub const VSSIP: usize = 1 << 2;
        pub const VSTIP: usize = 1 << 6;
        pub const VSEIP: usize = 1 << 10;
        pub unsafe fn write(hideleg: usize) {
            asm!(
                "csrw hideleg, {}",
                in(reg) hideleg
            )
        }
    }
}