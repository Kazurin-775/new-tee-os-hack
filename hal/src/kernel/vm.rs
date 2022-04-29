use core::{alloc::Layout, ops::Range};

pub use crate::sys::vm::*;

pub trait AddressSpace {
    fn current() -> Self;
    fn set_current(&self);
    fn alloc_map(&mut self, range: Range<usize>);
    fn unmap_dealloc(&mut self, range: Range<usize>);
    fn virt2phys(&self, ptr: *const ()) -> usize;
    fn phys2virt(&self, addr: usize) -> *const ();
}

pub trait ClonableAddressSpace: AddressSpace {
    /// Create a new address space that contains only the kernel mappings.
    fn create_bare(&self) -> Self;
    /// Copy a memory range from the active address space.
    fn copy_from_current(&mut self, range: Range<usize>);
}

// SAFETY: already checked
static ALLOC_LAYOUT_PAGE: Layout =
    unsafe { Layout::from_size_align_unchecked(kconfig::PAGE_SIZE, kconfig::PAGE_SIZE) };

pub fn alloc_page() -> *mut u8 {
    unsafe { alloc::alloc::alloc(ALLOC_LAYOUT_PAGE) }
}

pub fn dealloc_page(ptr: *mut u8) {
    unsafe {
        alloc::alloc::dealloc(ptr, ALLOC_LAYOUT_PAGE);
    }
}
