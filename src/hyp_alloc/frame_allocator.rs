//! Implementation of [`FrameAllocator`] which
//! controls all the frames in the operating system.

use crate::page_table::{PhysPageNum, PhysAddr};
use crate::constants::layout::MEMORY_END;
use alloc::vec::Vec;
use spin::{Once, Mutex};
use core::fmt::{self, Debug, Formatter};

/// manage a frame which has the same lifecycle as the tracker
#[derive(Clone)]
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        // page cleaning
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

/// an implementation for frame allocator
pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}
impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some((self.current - 1).into())
        }
    }
    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // validity check
        if ppn >= self.current || self.recycled.iter().any(|&v| v == ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}

type FrameAllocatorImpl = StackFrameAllocator;



pub static mut FRAME_ALLOCATOR: Once<Mutex<FrameAllocatorImpl>> = Once::new();

/// initiate the frame allocator using `einitrd` and `MEMORY_END`
pub fn init_frame_allocator() {
    extern "C" {
        fn einitrd();
    }
    unsafe{
        FRAME_ALLOCATOR.call_once(|| {
            let mut frame_allocator = FrameAllocatorImpl::new();
            frame_allocator.init(
                PhysAddr::from(einitrd as usize).ceil(),
                PhysAddr::from(MEMORY_END).floor(),
            );
            Mutex::new(frame_allocator)
        }); 
    }
}

/// allocate a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    // FRAME_ALLOCATOR
    //     .exclusive_access()
    //     .alloc()
    //     .map(FrameTracker::new)
    unsafe{
        let mut frame_allocator = FRAME_ALLOCATOR.get_mut();
        let mut frame_allocator = frame_allocator.as_mut().unwrap().lock();
        frame_allocator.alloc().map(FrameTracker::new)
    }
}

/// deallocate a frame
pub fn frame_dealloc(ppn: PhysPageNum) {
    unsafe{
        let mut frame_allocator = FRAME_ALLOCATOR.get_mut();
        let mut frame_allocator = frame_allocator.as_mut().unwrap().lock();
        frame_allocator.dealloc(ppn);
    }
}

#[allow(unused)]
/// a simple test for frame allocator
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}
