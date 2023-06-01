use riscv::register::hvip;
use riscv_decode::Instruction;

use crate::guest::vmexit::TrapContext;
use crate::{
    constants::MAX_CONTEXTS, guest::page_table::GuestPageTable, hypervisor::HostVmm,
    page_table::PageTable,
};
use crate::{VmmError, VmmResult};

pub const PLIC_OFFSET: &[(usize, usize)] = &[
    (0x0, 0x1000),                   // Interrupt priority
    (0x1000, 0x1000),                // Interrupt Pending
    (0x2000, 0x200000 - 0x2000),     // Enable
    (0x200000, 0x600000 - 0x200000), // threshold/claim/complete
];

pub struct PlicState {
    pub base_addr: usize,
    pub claim_complete: [u32; MAX_CONTEXTS],
}

impl PlicState {
    pub fn new(base_addr: usize) -> Self {
        Self {
            base_addr,
            claim_complete: [0u32; MAX_CONTEXTS],
        }
    }
}

impl<P: PageTable, G: GuestPageTable> HostVmm<P, G> {
    pub fn handle_plic_access(
        &mut self,
        ctx: &mut TrapContext,
        guest_pa: usize,
        instrution: Instruction,
    ) -> VmmResult {
        let host_plic = self.host_plic.as_mut().unwrap();
        let offset = guest_pa.wrapping_sub(host_plic.base_addr);
        // threshold/claim/complete
        if offset >= 0x200000 && offset < 0x200000 + 0x1000 * MAX_CONTEXTS {
            let hart = (offset - 0x200000) / 0x1000;
            let index = ((offset - 0x200000) & 0xfff) >> 2;
            if index == 0 {
                // threshold
                match instrution {
                    Instruction::Sw(i) => {
                        // guest write threshold register to plic core
                        let value = ctx.x[i.rs2() as usize] as u32;
                        // hdebug!("PLIC write addr@{:#x} -> {:#x}", guest_pa, value);
                        // todo: guest pa -> host pa
                        // htracking!(
                        //     "write PLIC threshold reg, addr: {:#x}, value: {:#x}",
                        //     guest_pa,
                        //     value
                        // );
                        unsafe {
                            core::ptr::write_volatile(guest_pa as *mut u32, value);
                        }
                    }
                    _ => return Err(VmmError::UnexpectedInst),
                }
            } else if index == 1 {
                // claim/complete
                // htracking!("claim/complete");
                match instrution {
                    Instruction::Lw(i) => {
                        // guest read claim from plic core
                        // htracking!(
                        //     "PLIC read addr@{:#x} -> {:#x}",
                        //     guest_pa,
                        //     host_plic.claim_complete[hart]
                        // );
                        ctx.x[i.rd() as usize] = host_plic.claim_complete[hart] as usize;
                    }
                    Instruction::Sw(i) => {
                        // guest write complete to plic core
                        let value = ctx.x[i.rs2() as usize] as u32;
                        // htracking!("Write plic addr@:{:#x} -> {:#x}", guest_pa, value);
                        // todo: guest pa -> host pa
                        unsafe {
                            core::ptr::write_volatile(guest_pa as *mut u32, value);
                        }
                        host_plic.claim_complete[hart] = 0;
                        unsafe {
                            hvip::clear_vseip();
                        }
                    }
                    _ => return Err(VmmError::UnexpectedInst),
                }
            }
        } else {
            panic!("Invalid address: {:#x}", guest_pa);
        }
        Ok(())
    }
}

#[inline(always)]
pub fn is_plic_access(addr: usize) -> bool {
    // let host_vmm = unsafe{ HOST_VMM.get().unwrap().lock() };
    // let plic = if let Some(plic) = &host_vmm.host_machine.plic {
    //     plic
    // }else{
    //     return false
    // };
    // if addr >= plic.base_address && addr <= plic.base_address + plic.size { return true }
    // else{ return false }

    // TODO: use guest machine base address
    return addr >= 0x0c00_0000 && addr < 0x1000_0000;
}
