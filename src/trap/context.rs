use riscv::register::hstatus::{self, Hstatus};

#[repr(C)]
#[derive(Debug)]
/// trap context structure containing sstatus, sepc and registers
pub struct TrapContext {
    /// general regs[0..31]
    pub x: [usize; 32],
    /// CSR hstatus      
    pub hstatus: Hstatus,
    /// CSR sepc
    pub sepc: usize,
    /// Addr of Page Table
    pub kernel_satp: usize,
    /// kernel stack
    pub kernel_sp: usize,
    /// Addr of trap_handler function
    pub trap_handler: usize,
}

impl TrapContext {
    /// set stack pointer to x_2 reg (sp)
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    /// init app context
    pub fn initialize_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        // let mut sstatus = sstatus::read(); // CSR sstatus
        let mut hstatus = hstatus::read();
        hstatus.set_spv(true);
        let mut cx = Self {
            x: [0; 32],
            hstatus,
            sepc: entry,  // entry point of app
            kernel_satp,  // addr of page table
            kernel_sp,    // kernel stack
            trap_handler, // addr of trap_handler function
        };
        cx.set_sp(sp); // app's user stack pointer
        cx // return initial Trap Context of app
    }
}
