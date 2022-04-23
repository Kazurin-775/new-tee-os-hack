use core::alloc::Layout;

use crate::vm::AddressSpace;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
};
pub use x86_64::{PhysAddr, VirtAddr};

pub struct HeapFrameAlloc;

pub const LAYOUT_4K_PAGE: Layout = unsafe { Layout::from_size_align_unchecked(0x1000, 0x1000) };
pub const MIRROR_BASE_VIRT: VirtAddr = VirtAddr::new_truncate(kconfig::KERNEL_MIRROR_BASE as u64);

unsafe impl FrameAllocator<Size4KiB> for HeapFrameAlloc {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let virt_addr = VirtAddr::from_ptr(unsafe { alloc::alloc::alloc(LAYOUT_4K_PAGE) });
        if virt_addr.is_null() {
            return None;
        }
        let phys_addr = PhysAddr::new(virt_addr - MIRROR_BASE_VIRT);
        Some(PhysFrame::from_start_address(phys_addr).unwrap())
    }
}

pub struct UserAddressSpace {
    pub rpt_ptr: *mut PageTable,
}

impl AddressSpace for UserAddressSpace {
    fn current() -> UserAddressSpace {
        // get root page table address
        let rpt_phys = x86_64::registers::control::Cr3::read().0;
        UserAddressSpace {
            rpt_ptr: (MIRROR_BASE_VIRT + rpt_phys.start_address().as_u64()).as_mut_ptr(),
        }
    }

    fn alloc_map(&mut self, range: core::ops::Range<usize>) {
        assert_eq!(range.start & 0xFFF, 0);
        assert_eq!(range.end & 0xFFF, 0);

        for addr in range.step_by(0x1000) {
            let page = VirtAddr::from_ptr(crate::vm::alloc_page());
            self.map_single(
                VirtAddr::new(addr as u64),
                PhysAddr::new(AddressSpace::virt2phys(self, page.as_ptr()) as u64),
            );
        }
    }

    fn virt2phys(&self, ptr: *const ()) -> usize {
        ptr as usize - MIRROR_BASE_VIRT.as_u64() as usize
    }

    fn phys2virt(&self, addr: usize) -> *const () {
        (MIRROR_BASE_VIRT.as_u64() as usize + addr) as *const ()
    }
}

impl UserAddressSpace {
    /// Borrow the root page table temporarily.
    fn rpt(&self) -> OffsetPageTable<'_> {
        unsafe { OffsetPageTable::new(&mut *self.rpt_ptr, MIRROR_BASE_VIRT) }
    }

    pub fn map_single(&mut self, virt: VirtAddr, phys: PhysAddr) {
        unsafe {
            self.rpt()
                .map_to(
                    Page::<Size4KiB>::from_start_address(virt).unwrap(),
                    PhysFrame::from_start_address(phys).unwrap(),
                    PageTableFlags::PRESENT
                        | PageTableFlags::WRITABLE
                        | PageTableFlags::USER_ACCESSIBLE,
                    &mut HeapFrameAlloc,
                )
                .unwrap()
                .flush();
        }
    }
}

unsafe impl Send for UserAddressSpace {}

impl core::fmt::Debug for UserAddressSpace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UserAddressSpace").finish()
    }
}
