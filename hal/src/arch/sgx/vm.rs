use core::ops::Range;

use crate::vm::AddressSpace;

#[derive(Debug)]
pub struct UserAddressSpace;

impl AddressSpace for UserAddressSpace {
    fn current() -> UserAddressSpace {
        UserAddressSpace
    }

    fn set_current(&self) {
        // no-op
    }

    fn alloc_map(&mut self, range: Range<usize>) {
        let result = unsafe {
            sgx_alloc::rsrvmem::alloc_with_addr(
                range.start as *mut u8,
                u32::try_from(range.len() / kconfig::PAGE_SIZE).unwrap(),
            )
        };
        assert_eq!(result as usize, range.start);
    }

    fn alloc_map_zeroed(&mut self, range: Range<usize>) {
        self.alloc_map(range.clone());
        unsafe {
            core::ptr::write_bytes(range.start as *mut u8, 0, range.len());
        }
    }

    fn unmap_dealloc(&mut self, range: Range<usize>) {
        unsafe {
            sgx_alloc::rsrvmem::dealloc(
                range.start as *mut u8,
                u32::try_from(range.len() / kconfig::PAGE_SIZE).unwrap(),
            )
            .expect("deallocation failed?!");
        }
    }

    fn virt2phys(&self, ptr: *const ()) -> usize {
        ptr as usize
    }

    fn phys2virt(&self, addr: usize) -> *const () {
        addr as *const ()
    }
}
