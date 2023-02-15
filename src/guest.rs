use crate::{mm::MemorySet, page_table::PageTable};

pub struct Guest<P: PageTable> {
    pub gpm: MemorySet<P>,
    pub guest_id: usize
}

impl<P: PageTable> Guest<P> {
    pub fn new(guest_id: usize, gpm: MemorySet<P>) -> Self {
        Self {
            guest_id,
            gpm
        }
    }
}