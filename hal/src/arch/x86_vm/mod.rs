pub mod gdt;
pub mod qemu;
pub mod security;
pub mod tss;

pub fn arch_init() {
    gdt::GDT.load();
    unsafe {
        gdt::apply_selectors();
    }
    security::enforce();
}
