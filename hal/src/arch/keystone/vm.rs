use core::ops::Range;

use kconfig::*;
use riscv_sv39::{PageManager, PageTableEntry, RootPageTable};
pub use riscv_sv39::{PhysAddr, VirtAddr};

use crate::vm::{AddressSpace, ClonableAddressSpace};

#[derive(Clone, Copy)]
pub struct HeapPageManager {
    epm_base: usize,
}

impl HeapPageManager {
    pub fn new() -> HeapPageManager {
        HeapPageManager {
            epm_base: *crate::arch::keystone::EPM_PHYS.get().unwrap(),
        }
    }

    pub fn phys2virt(&self, phys: PhysAddr) -> VirtAddr {
        VirtAddr(phys.0 - self.epm_base + KERNEL_MIRROR_BASE)
    }

    pub fn virt2phys(&self, virt: VirtAddr) -> PhysAddr {
        PhysAddr(virt.0 - KERNEL_MIRROR_BASE + self.epm_base)
    }
}

impl PageManager for HeapPageManager {
    fn alloc_physical_page(&mut self) -> PhysAddr {
        let addr = VirtAddr::from_ptr(crate::vm::alloc_page());
        log::trace!("Allocated {:#X} for page table", addr.0);
        self.virt2phys(addr)
    }

    unsafe fn map_physical_page(&mut self, phys: PhysAddr) -> *mut () {
        self.phys2virt(phys).as_mut_ptr()
    }
}

pub struct UserAddressSpace {
    pub mem_mgr: HeapPageManager,
    pub inner: RootPageTable<HeapPageManager>,
}

impl AddressSpace for UserAddressSpace {
    fn current() -> UserAddressSpace {
        let rpt_phys = PhysAddr(riscv::register::satp::read().ppn() << 12);
        let mem_mgr = HeapPageManager::new();
        let rpt_virt = mem_mgr.phys2virt(rpt_phys);
        UserAddressSpace {
            mem_mgr,
            inner: RootPageTable::new(rpt_virt.as_mut_ptr(), mem_mgr),
        }
    }

    fn set_current(&self) {
        let cur_ppn = riscv::register::satp::read().ppn();
        let my_ppn = self.virt2phys(self.inner.as_ptr().cast()) >> 12;
        if cur_ppn != my_ppn {
            log::trace!(
                "Switching address space from {:#X} -> {:#X}",
                cur_ppn,
                my_ppn,
            );
            unsafe {
                // TODO: assign an ASID
                riscv::register::satp::set(riscv::register::satp::Mode::Sv39, 0, my_ppn);
                riscv::asm::sfence_vma_all();
            }
        }
    }

    fn alloc_map(&mut self, range: Range<usize>) {
        assert_eq!(range.start & 0xFFF, 0);
        assert_eq!(range.end & 0xFFF, 0);

        for addr in range.step_by(0x1000) {
            let page = VirtAddr::from_ptr(crate::vm::alloc_page());
            log::trace!("Allocated {:#X} for user address {:#X}", page.0, addr);
            self.map_single(VirtAddr(addr), self.mem_mgr.virt2phys(page));
        }
    }

    fn alloc_map_zeroed(&mut self, range: Range<usize>) {
        assert_eq!(range.start & 0xFFF, 0);
        assert_eq!(range.end & 0xFFF, 0);

        for addr in range.step_by(0x1000) {
            let page = VirtAddr::from_ptr(crate::vm::alloc_page_zeroed());
            log::trace!("Allocated {:#X} for user address {:#X}", page.0, addr);
            self.map_single(VirtAddr(addr), self.mem_mgr.virt2phys(page));
        }
    }

    fn unmap_dealloc(&mut self, range: Range<usize>) {
        assert_eq!(range.start & 0xFFF, 0);
        assert_eq!(range.end & 0xFFF, 0);

        for addr in range.step_by(0x1000) {
            self.unmap_dealloc_single(VirtAddr(addr));
        }
    }

    fn virt2phys(&self, ptr: *const ()) -> usize {
        self.mem_mgr.virt2phys(VirtAddr::from_ptr(ptr)).0
    }

    fn phys2virt(&self, addr: usize) -> *const () {
        self.mem_mgr.phys2virt(PhysAddr(addr)).as_ptr()
    }
}

impl UserAddressSpace {
    pub fn map_single(&mut self, virt: VirtAddr, phys: PhysAddr) {
        unsafe {
            self.inner
                .map_4k(virt, PageTableEntry::for_phys(phys).make_user().make_rwx());
        }
    }

    pub fn unmap_dealloc_single(&mut self, user_virt: VirtAddr) {
        let pte = unsafe { self.inner.access_4k(user_virt) };
        let phys = unsafe { (*pte).ppn() } << 12;
        let virt = self.mem_mgr.phys2virt(PhysAddr(phys));
        log::trace!("Deallocating and unmapping {:#X}", virt.0);
        unsafe {
            pte.write(PageTableEntry::invalid());
            // TODO: change ASID
            riscv::asm::sfence_vma(0, user_virt.0);
            // core::slice::from_raw_parts_mut(virt.as_mut_ptr() as *mut u8, PAGE_SIZE).fill(0xCC);
        }
        crate::vm::dealloc_page(virt.as_mut_ptr());
    }
}

impl ClonableAddressSpace for UserAddressSpace {
    fn create_bare(&self) -> Self {
        let old_rpt = self.inner.as_ptr();
        let new_rpt = crate::vm::alloc_page() as *mut PageTableEntry;
        log::debug!("Allocated new root page table at {:?}", new_rpt);

        // Copy kernel mappings to the new page table
        unsafe {
            let src = core::slice::from_raw_parts(old_rpt.add(256), 256);
            core::slice::from_raw_parts_mut(new_rpt.add(256), 256).copy_from_slice(src);
        }

        UserAddressSpace {
            mem_mgr: self.mem_mgr,
            inner: RootPageTable::new(new_rpt.cast(), self.mem_mgr),
        }
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
            let phys = self.mem_mgr.virt2phys(VirtAddr::from_ptr(page));
            // log::trace!("Mapping VirtAdr({:#X}) to {:?}", addr, phys);
            self.map_single(VirtAddr(addr), phys);
        }
    }
}

unsafe impl Send for UserAddressSpace {}

impl core::fmt::Debug for UserAddressSpace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UserAddressSpace").finish()
    }
}
