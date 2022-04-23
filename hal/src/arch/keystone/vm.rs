use core::ops::Range;

use kconfig::*;
use riscv_sv39::{PageManager, PageTableEntry, RootPageTable};
pub use riscv_sv39::{PhysAddr, VirtAddr};

use crate::vm::AddressSpace;

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
        log::debug!("Allocated {:#X} for page table", addr.0);
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

    fn alloc_map(&mut self, range: Range<usize>) {
        assert_eq!(range.start & 0xFFF, 0);
        assert_eq!(range.end & 0xFFF, 0);

        for addr in range.step_by(0x1000) {
            let page = VirtAddr::from_ptr(crate::vm::alloc_page());
            self.map_single(VirtAddr(addr), self.mem_mgr.virt2phys(page));
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
}

unsafe impl Send for UserAddressSpace {}

impl core::fmt::Debug for UserAddressSpace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UserAddressSpace").finish()
    }
}
