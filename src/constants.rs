pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 4;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;


pub const MAX_GUESTS: usize = 4;
pub const MAX_GUEST_HARTS: usize = 16;
/// Number of contexts for the PLIC. Value is twice the max number of harts because each hart will
/// have on M-mode context and one S-mode context.
pub const MAX_CONTEXTS: usize = 16 * 2;

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
    pub const ustatus: usize = 0x000;
    pub const uie: usize = 0x004;
    pub const utvec: usize = 0x005;
    pub const uscratch: usize = 0x040;
    pub const uepc: usize = 0x041;
    pub const ucause: usize = 0x042;
    pub const utval: usize = 0x043;
    pub const uip: usize = 0x044;
    pub const fflags: usize = 0x001;
    pub const frm: usize = 0x002;
    pub const fcsr: usize = 0x003;
    pub const cycle: usize = 0xc00;
    pub const time: usize = 0xc01;
    pub const instret: usize = 0xc02;
    pub const hpmcounter3: usize = 0xc03;
    pub const hpmcounter4: usize = 0xc04;
    pub const hpmcounter5: usize = 0xc05;
    pub const hpmcounter6: usize = 0xc06;
    pub const hpmcounter7: usize = 0xc07;
    pub const hpmcounter8: usize = 0xc08;
    pub const hpmcounter9: usize = 0xc09;
    pub const hpmcounter10: usize = 0xc0a;
    pub const hpmcounter11: usize = 0xc0b;
    pub const hpmcounter12: usize = 0xc0c;
    pub const hpmcounter13: usize = 0xc0d;
    pub const hpmcounter14: usize = 0xc0e;
    pub const hpmcounter15: usize = 0xc0f;
    pub const hpmcounter16: usize = 0xc10;
    pub const hpmcounter17: usize = 0xc11;
    pub const hpmcounter18: usize = 0xc12;
    pub const hpmcounter19: usize = 0xc13;
    pub const hpmcounter20: usize = 0xc14;
    pub const hpmcounter21: usize = 0xc15;
    pub const hpmcounter22: usize = 0xc16;
    pub const hpmcounter23: usize = 0xc17;
    pub const hpmcounter24: usize = 0xc18;
    pub const hpmcounter25: usize = 0xc19;
    pub const hpmcounter26: usize = 0xc1a;
    pub const hpmcounter27: usize = 0xc1b;
    pub const hpmcounter28: usize = 0xc1c;
    pub const hpmcounter29: usize = 0xc1d;
    pub const hpmcounter30: usize = 0xc1e;
    pub const hpmcounter31: usize = 0xc1f;
    pub const cycleh: usize = 0xc80;
    pub const timeh: usize = 0xc81;
    pub const instreth: usize = 0xc82;
    pub const hpmcounter3h: usize = 0xc83;
    pub const hpmcounter4h: usize = 0xc84;
    pub const hpmcounter5h: usize = 0xc85;
    pub const hpmcounter6h: usize = 0xc86;
    pub const hpmcounter7h: usize = 0xc87;
    pub const hpmcounter8h: usize = 0xc88;
    pub const hpmcounter9h: usize = 0xc89;
    pub const hpmcounter10h: usize = 0xc8a;
    pub const hpmcounter11h: usize = 0xc8b;
    pub const hpmcounter12h: usize = 0xc8c;
    pub const hpmcounter13h: usize = 0xc8d;
    pub const hpmcounter14h: usize = 0xc8e;
    pub const hpmcounter15h: usize = 0xc8f;
    pub const hpmcounter16h: usize = 0xc90;
    pub const hpmcounter17h: usize = 0xc91;
    pub const hpmcounter18h: usize = 0xc92;
    pub const hpmcounter19h: usize = 0xc93;
    pub const hpmcounter20h: usize = 0xc94;
    pub const hpmcounter21h: usize = 0xc95;
    pub const hpmcounter22h: usize = 0xc96;
    pub const hpmcounter23h: usize = 0xc97;
    pub const hpmcounter24h: usize = 0xc98;
    pub const hpmcounter25h: usize = 0xc99;
    pub const hpmcounter26h: usize = 0xc9a;
    pub const hpmcounter27h: usize = 0xc9b;
    pub const hpmcounter28h: usize = 0xc9c;
    pub const hpmcounter29h: usize = 0xc9d;
    pub const hpmcounter30h: usize = 0xc9e;
    pub const hpmcounter31h: usize = 0xc9f;
    pub const mcycle: usize = 0xb00;
    pub const minstret: usize = 0xb02;
    pub const mcycleh: usize = 0xb80;
    pub const minstreth: usize = 0xb82;
    pub const mvendorid: usize = 0xf11;
    pub const marchid: usize = 0xf12;
    pub const mimpid: usize = 0xf13;
    pub const mhartid: usize = 0xf14;
    pub const mstatus: usize = 0x300;
    pub const misa: usize = 0x301;
    pub const medeleg: usize = 0x302;
    pub const mideleg: usize = 0x303;
    pub const mie: usize = 0x304;
    pub const mtvec: usize = 0x305;
    pub const mcounteren: usize = 0x306;
    pub const mtvt: usize = 0x307;
    pub const mucounteren: usize = 0x320;
    pub const mscounteren: usize = 0x321;
    pub const mscratch: usize = 0x340;
    pub const mepc: usize = 0x341;
    pub const mcause: usize = 0x342;
    pub const mbadaddr: usize = 0x343;
    pub const mtval: usize = 0x343;
    pub const mip: usize = 0x344;
    pub const mnxti: usize = 0x345;
    pub const mintstatus: usize = 0x346;
    pub const mscratchcsw: usize = 0x348;
    pub const sstatus: usize = 0x100;
    pub const sedeleg: usize = 0x102;
    pub const sideleg: usize = 0x103;
    pub const sie: usize = 0x104;
    pub const stvec: usize = 0x105;
    pub const scounteren: usize = 0x106;
    pub const stvt: usize = 0x107;
    pub const sscratch: usize = 0x140;
    pub const sepc: usize = 0x141;
    pub const scause: usize = 0x142;
    pub const sbadaddr: usize = 0x143;
    pub const stval: usize = 0x143;
    pub const sip: usize = 0x144;
    pub const snxti: usize = 0x145;
    pub const sintstatus: usize = 0x146;
    pub const sscratchcsw: usize = 0x148;
    pub const sptbr: usize = 0x180;
    pub const satp: usize = 0x180;
    pub const pmpcfg0: usize = 0x3a0;
    pub const pmpcfg1: usize = 0x3a1;
    pub const pmpcfg2: usize = 0x3a2;
    pub const pmpcfg3: usize = 0x3a3;
    pub const pmpaddr0: usize = 0x3b0;
    pub const pmpaddr1: usize = 0x3b1;
    pub const pmpaddr2: usize = 0x3b2;
    pub const pmpaddr3: usize = 0x3b3;
    pub const pmpaddr4: usize = 0x3b4;
    pub const pmpaddr5: usize = 0x3b5;
    pub const pmpaddr6: usize = 0x3b6;
    pub const pmpaddr7: usize = 0x3b7;
    pub const pmpaddr8: usize = 0x3b8;
    pub const pmpaddr9: usize = 0x3b9;
    pub const pmpaddr10: usize = 0x3ba;
    pub const pmpaddr11: usize = 0x3bb;
    pub const pmpaddr12: usize = 0x3bc;
    pub const pmpaddr13: usize = 0x3bd;
    pub const pmpaddr14: usize = 0x3be;
    pub const pmpaddr15: usize = 0x3bf;
    pub const tselect: usize = 0x7a0;
    pub const tdata1: usize = 0x7a1;
    pub const tdata2: usize = 0x7a2;
    pub const tdata3: usize = 0x7a3;
    pub const dcsr: usize = 0x7b0;
    pub const dpc: usize = 0x7b1;
    pub const dscratch: usize = 0x7b2;
    pub const mhpmcounter3: usize = 0xb03;
    pub const mhpmcounter4: usize = 0xb04;
    pub const mhpmcounter5: usize = 0xb05;
    pub const mhpmcounter6: usize = 0xb06;
    pub const mhpmcounter7: usize = 0xb07;
    pub const mhpmcounter8: usize = 0xb08;
    pub const mhpmcounter9: usize = 0xb09;
    pub const mhpmcounter10: usize = 0xb0a;
    pub const mhpmcounter11: usize = 0xb0b;
    pub const mhpmcounter12: usize = 0xb0c;
    pub const mhpmcounter13: usize = 0xb0d;
    pub const mhpmcounter14: usize = 0xb0e;
    pub const mhpmcounter15: usize = 0xb0f;
    pub const mhpmcounter16: usize = 0xb10;
    pub const mhpmcounter17: usize = 0xb11;
    pub const mhpmcounter18: usize = 0xb12;
    pub const mhpmcounter19: usize = 0xb13;
    pub const mhpmcounter20: usize = 0xb14;
    pub const mhpmcounter21: usize = 0xb15;
    pub const mhpmcounter22: usize = 0xb16;
    pub const mhpmcounter23: usize = 0xb17;
    pub const mhpmcounter24: usize = 0xb18;
    pub const mhpmcounter25: usize = 0xb19;
    pub const mhpmcounter26: usize = 0xb1a;
    pub const mhpmcounter27: usize = 0xb1b;
    pub const mhpmcounter28: usize = 0xb1c;
    pub const mhpmcounter29: usize = 0xb1d;
    pub const mhpmcounter30: usize = 0xb1e;
    pub const mhpmcounter31: usize = 0xb1f;
    pub const mhpmevent3: usize = 0x323;
    pub const mhpmevent4: usize = 0x324;
    pub const mhpmevent5: usize = 0x325;
    pub const mhpmevent6: usize = 0x326;
    pub const mhpmevent7: usize = 0x327;
    pub const mhpmevent8: usize = 0x328;
    pub const mhpmevent9: usize = 0x329;
    pub const mhpmevent10: usize = 0x32a;
    pub const mhpmevent11: usize = 0x32b;
    pub const mhpmevent12: usize = 0x32c;
    pub const mhpmevent13: usize = 0x32d;
    pub const mhpmevent14: usize = 0x32e;
    pub const mhpmevent15: usize = 0x32f;
    pub const mhpmevent16: usize = 0x330;
    pub const mhpmevent17: usize = 0x331;
    pub const mhpmevent18: usize = 0x332;
    pub const mhpmevent19: usize = 0x333;
    pub const mhpmevent20: usize = 0x334;
    pub const mhpmevent21: usize = 0x335;
    pub const mhpmevent22: usize = 0x336;
    pub const mhpmevent23: usize = 0x337;
    pub const mhpmevent24: usize = 0x338;
    pub const mhpmevent25: usize = 0x339;
    pub const mhpmevent26: usize = 0x33a;
    pub const mhpmevent27: usize = 0x33b;
    pub const mhpmevent28: usize = 0x33c;
    pub const mhpmevent29: usize = 0x33d;
    pub const mhpmevent30: usize = 0x33e;
    pub const mhpmevent31: usize = 0x33f;
    pub const mhpmcounter3h: usize = 0xb83;
    pub const mhpmcounter4h: usize = 0xb84;
    pub const mhpmcounter5h: usize = 0xb85;
    pub const mhpmcounter6h: usize = 0xb86;
    pub const mhpmcounter7h: usize = 0xb87;
    pub const mhpmcounter8h: usize = 0xb88;
    pub const mhpmcounter9h: usize = 0xb89;
    pub const mhpmcounter10h: usize = 0xb8a;
    pub const mhpmcounter11h: usize = 0xb8b;
    pub const mhpmcounter12h: usize = 0xb8c;
    pub const mhpmcounter13h: usize = 0xb8d;
    pub const mhpmcounter14h: usize = 0xb8e;
    pub const mhpmcounter15h: usize = 0xb8f;
    pub const mhpmcounter16h: usize = 0xb90;
    pub const mhpmcounter17h: usize = 0xb91;
    pub const mhpmcounter18h: usize = 0xb92;
    pub const mhpmcounter19h: usize = 0xb93;
    pub const mhpmcounter20h: usize = 0xb94;
    pub const mhpmcounter21h: usize = 0xb95;
    pub const mhpmcounter22h: usize = 0xb96;
    pub const mhpmcounter23h: usize = 0xb97;
    pub const mhpmcounter24h: usize = 0xb98;
    pub const mhpmcounter25h: usize = 0xb99;
    pub const mhpmcounter26h: usize = 0xb9a;
    pub const mhpmcounter27h: usize = 0xb9b;
    pub const mhpmcounter28h: usize = 0xb9c;
    pub const mhpmcounter29h: usize = 0xb9d;
    pub const mhpmcounter30h: usize = 0xb9e;
    pub const mhpmcounter31h: usize = 0xb9f;
    
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

    pub mod hcounteren {
        use core::arch::asm;

        pub unsafe fn write(hcounteren: u32) {
            asm!(
                "csrw hcounteren, {}",
                in(reg) hcounteren
            )
        }
    }


    pub mod sip {
        /// software interrupts pending
        pub const SSIP: usize = 1 << 1;
        /// timer interrupts pending
        pub const STIP: usize = 1 << 5;
        /// external interrupts pending
        pub const SEIP: usize = 1 << 9;
    }

}

pub mod riscv_regs {
    #[derive(Default)]
    #[repr(C)]
    pub struct GeneralPurposeRegisters([u64; 32]);

    /// Index of risc-v general purpose registers in `GeneralPurposeRegisters`.
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum GprIndex {
        Zero = 0,
        RA,
        SP,
        GP,
        TP,
        T0,
        T1,
        T2,
        S0,
        S1,
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        S2,
        S3,
        S4,
        S5,
        S6,
        S7,
        S8,
        S9,
        S10,
        S11,
        T3,
        T4,
        T5,
        T6,
    }

    impl GprIndex {
        pub fn from_raw(raw: u32) -> Option<Self> {
            use GprIndex::*;
            let index = match raw {
                0 => Zero,
                1 => RA,
                2 => SP,
                3 => GP,
                4 => TP,
                5 => T0,
                6 => T1,
                7 => T2,
                8 => S0,
                9 => S1,
                10 => A0,
                11 => A1,
                12 => A2,
                13 => A3,
                14 => A4,
                15 => A5,
                16 => A6,
                17 => A7,
                18 => S2,
                19 => S3,
                20 => S4,
                21 => S5,
                22 => S6,
                23 => S7,
                24 => S8,
                25 => S9,
                26 => S10,
                27 => S11,
                28 => T3,
                29 => T4,
                30 => T5,
                31 => T6,
                _ => {
                    return None;
                }
            };
            Some(index)
        }
    }

    impl GeneralPurposeRegisters {
        /// Returns the value of the given register.
        pub fn reg(&self, reg_index: GprIndex) -> u64 {
            self.0[reg_index as usize]
        }

        /// Sets the value of the given register.
        pub fn set_reg(&mut self, reg_index: GprIndex, val: u64) {
            if reg_index == GprIndex::Zero {
                return;
            }

            self.0[reg_index as usize] = val;
        }

        /// Returns the argument registers.
        /// This is avoids many calls when an SBI handler needs all of the argmuent regs.
        pub fn a_regs(&self) -> &[u64] {
            &self.0[GprIndex::A0 as usize..=GprIndex::A7 as usize]
        }

        /// Returns the arguments register as a mutable.
        pub fn a_regs_mut(&mut self) -> &mut [u64] {
            &mut self.0[GprIndex::A0 as usize..=GprIndex::A7 as usize]
        }
    }

}