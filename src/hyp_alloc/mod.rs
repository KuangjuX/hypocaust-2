mod frame_allocator;
mod heap_allocator;

pub use frame_allocator::{frame_alloc, frame_dealloc, FrameTracker};

/// initiate heap allocator, frame allocator and kernel space
pub fn heap_init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    hdebug!("Heap initialize finished!");
}