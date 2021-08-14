pub const PAGE_SIZE: usize = 0x1_000;

// also defined in the target configuration JSON
pub const KERNEL_BASE: usize = 0xFFFF_FFFF_0000_0000;
pub const KERNEL_STACK_BASE: usize = 0xFFFF_FFFF_FFF8_0000;
// also defined in `x86-vm-kernel/Cargo.toml`
// 3 GiB is not sufficient for QEMU
pub const KERNEL_MIRROR_BASE: usize = 0xFFFF_FFF0_0000_0000;
