use hal::{arch::keystone::vm::UserAddressSpace, cfg::*, vm::AddressSpace};

pub unsafe fn handle_page_fault_at(addr: usize) {
    log::debug!("Page fault at address {:#X}", addr);

    let mut addr_space = UserAddressSpace::current();
    let addr = addr & !0xFFF;
    addr_space.alloc_map(addr..addr + PAGE_SIZE);
}
