use core::{alloc::Layout, ops::Range};

use crate::vm::{AddressSpace, ClonableAddressSpace};
use x86_64::structures::paging::{
    page_table::PageTableEntry, FrameAllocator, Mapper, OffsetPageTable, Page, PageTable,
    PageTableFlags, PhysFrame, Size4KiB,
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

    fn set_current(&self) {
        let (cur_rpt_phys, flags) = x86_64::registers::control::Cr3::read();
        let my_rpt_phys = self.virt2phys(self.rpt_ptr.cast());
        if cur_rpt_phys.start_address().as_u64() as usize != my_rpt_phys {
            log::trace!(
                "Switching address space from {:?} -> {:#X}",
                cur_rpt_phys.start_address(),
                my_rpt_phys,
            );
            unsafe {
                x86_64::registers::control::Cr3::write(
                    PhysFrame::from_start_address(PhysAddr::new(my_rpt_phys as u64)).unwrap(),
                    flags,
                );
            }
        }
    }

    fn alloc_map(&mut self, range: core::ops::Range<usize>) {
        assert_eq!(range.start & 0xFFF, 0);
        assert_eq!(range.end & 0xFFF, 0);

        for addr in range.step_by(0x1000) {
            let page = VirtAddr::from_ptr(crate::vm::alloc_page());
            log::trace!("Allocated {:#X} for user address {:#X}", page.as_u64(), addr);
            self.map_single(
                VirtAddr::new(addr as u64),
                PhysAddr::new(AddressSpace::virt2phys(self, page.as_ptr()) as u64),
            );
        }
    }

    fn unmap_dealloc(&mut self, range: Range<usize>) {
        assert_eq!(range.start & 0xFFF, 0);
        assert_eq!(range.end & 0xFFF, 0);

        for addr in range.step_by(0x1000) {
            self.unmap_dealloc_single(VirtAddr::new(addr as u64));
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

    pub fn cleanup_bootloader(&mut self) {
        // The `bootloader` crate maps RDSP unconditionally, which interferes
        // with our user address space. Here we manually clean up these regions
        // as a workaround.
        let mut rpt = self.rpt();
        for page in (0x11000..0x13000).step_by(0x1000) {
            let page = Page::<Size4KiB>::from_start_address(VirtAddr::new(page)).unwrap();
            match rpt.unmap(page) {
                Ok(orig) => {
                    log::trace!("Unmapped bootloader region {:?} ({:?})", page, orig.0);
                    orig.1.flush();
                }
                Err(err) => {
                    log::warn!("Failed to unmap bootloader region {:?}: {:?}", page, err);
                }
            }
        }
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

    pub fn unmap_dealloc_single(&mut self, user_virt: VirtAddr) {
        let (phys, flusher) = self
            .rpt()
            .unmap(Page::<Size4KiB>::from_start_address(user_virt).unwrap())
            .expect("address not mapped?!");
        flusher.flush();
        let virt = self.phys2virt(phys.start_address().as_u64() as usize);
        log::trace!("Deallocating and unmapping {:?}", virt);
        // unsafe {
        //     core::slice::from_raw_parts_mut(virt as *mut u8, kconfig::PAGE_SIZE).fill(0xCC);
        // }
        crate::vm::dealloc_page(virt as *mut u8);
    }
}

impl ClonableAddressSpace for UserAddressSpace {
    fn create_bare(&self) -> UserAddressSpace {
        let old_rpt = self.rpt_ptr;
        let new_rpt = crate::vm::alloc_page() as *mut PageTable;
        log::debug!("Allocated new root page table at {:?}", new_rpt);
        unsafe {
            // Clear the new page table
            (*new_rpt).zero();
        }

        // Copy kernel mappings to the new page table
        unsafe {
            let old_rpt = old_rpt as *const PageTableEntry;
            let new_rpt = new_rpt as *mut PageTableEntry;
            core::ptr::copy_nonoverlapping(old_rpt.add(256), new_rpt.add(256), 256);
        }

        UserAddressSpace { rpt_ptr: new_rpt }
    }

    fn copy_from_current(&mut self, range: Range<usize>) {
        assert!((range.start & 0xFFF) == 0 && (range.end & 0xFFF) == 0);
        for addr in range.step_by(0x1000) {
            let page = crate::vm::alloc_page();
            log::trace!("Copying page at {:#X} to VirtAddr({:?})", addr, page);
            unsafe {
                crate::mem::copy_from_user(
                    core::slice::from_raw_parts_mut(page, 0x1000),
                    addr as *const u8,
                );
            }
            let phys = self.virt2phys(page.cast());
            // log::trace!("Mapping VirtAddr({:#X}) to {:?}", addr, phys);
            self.map_single(VirtAddr::new(addr as u64), PhysAddr::new(phys as u64));
        }
    }
}

unsafe impl Send for UserAddressSpace {}

impl core::fmt::Debug for UserAddressSpace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UserAddressSpace").finish()
    }
}
