use core::{alloc::Layout, ops::Range};

pub trait AddressSpace {
    fn current() -> Self;
    fn set_current(&self);
    fn alloc_map(&mut self, range: Range<usize>);
    fn virt2phys(&self, ptr: *const ()) -> usize;
    fn phys2virt(&self, addr: usize) -> *const ();
}

// SAFETY: already checked
static ALLOC_LAYOUT_PAGE: Layout =
    unsafe { Layout::from_size_align_unchecked(kconfig::PAGE_SIZE, kconfig::PAGE_SIZE) };

pub fn alloc_page() -> *const u8 {
    unsafe { alloc::alloc::alloc(ALLOC_LAYOUT_PAGE) }
}
