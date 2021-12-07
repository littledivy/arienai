use alloc_cortex_m::CortexMHeap;

// 64 Kib
const HEAP_SIZE: usize = 1024 * 64;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

pub fn init() {
  unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE) }
}
