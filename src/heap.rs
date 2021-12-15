use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::ptr::{self, NonNull};

use linked_list_allocator::Heap;
use riscv::interrupt::Mutex;

pub struct RISCVHeap {
  heap: Mutex<RefCell<Heap>>,
}

impl RISCVHeap {
  pub const fn empty() -> Self {
    Self {
      heap: Mutex::new(RefCell::new(Heap::empty())),
    }
  }

  pub unsafe fn init(&self, start_addr: usize, size: usize) {
    riscv::interrupt::free(|cs| {
      self.heap.borrow(*cs).borrow_mut().init(start_addr, size)
    });
  }

  pub fn used(&self) -> usize {
    riscv::interrupt::free(|cs| self.heap.borrow(*cs).borrow_mut().used())
  }

  /// Returns an estimate of the amount of bytes available.
  pub fn free(&self) -> usize {
    riscv::interrupt::free(|cs| self.heap.borrow(*cs).borrow_mut().free())
  }
}

unsafe impl GlobalAlloc for RISCVHeap {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    riscv::interrupt::free(|cs| {
      self
        .heap
        .borrow(*cs)
        .borrow_mut()
        .allocate_first_fit(layout)
        .ok()
        .map_or(ptr::null_mut(), |allocation| allocation.as_ptr())
    })
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    riscv::interrupt::free(|cs| {
      self
        .heap
        .borrow(*cs)
        .borrow_mut()
        .deallocate(NonNull::new_unchecked(ptr), layout)
    });
  }
}

// 32 Kib
const HEAP_SIZE: usize = 1024 * 32;

#[global_allocator]
static ALLOCATOR: RISCVHeap = RISCVHeap::empty();

extern "C" {
  static mut _sheap: u32;
}

#[inline]
pub fn heap_start() -> *mut u32 {
  unsafe { &mut _sheap }
}

pub fn init() {
  unsafe { ALLOCATOR.init(heap_start() as usize, HEAP_SIZE) }
}
