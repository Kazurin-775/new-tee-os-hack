pub mod frame;
pub mod gdt;
pub mod qemu;
pub mod security;
pub mod tss;
pub mod vm;

pub unsafe fn initialize_edge_caller(edge_mem: *mut u8) {
    crate::sys::edge::initialize_edge_caller(edge_mem);
}

pub fn arch_init() {
    gdt::GDT.load();
    unsafe {
        gdt::apply_selectors();
    }
    security::enforce();
}
