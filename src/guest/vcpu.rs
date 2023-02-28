use alloc::collections::VecDeque;

pub struct VCpu {
    pub hart: usize,
    /// pending interrupts
    pub pending_events: VecDeque<u32>
}

impl VCpu {
    pub fn new(hart: usize) -> Self {
        Self{
            hart,
            pending_events: VecDeque::new()
        }
    }
}