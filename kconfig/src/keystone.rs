pub const PAGE_SIZE: usize = 0x1_000;
// also defined in keystone.lds
pub const KERNEL_BASE: usize = 0xFFFF_FFFF_C000_0000;
pub const KERNEL_PAGE_TABLE_PREALLOC: usize = 0x10_000;

pub const KERNEL_MIRROR_BASE: usize = 0xFFFF_FFFF_0000_0000;
pub const KERNEL_UTM_BASE: usize = 0xFFFF_FFFF_8000_0000;

// This is one byte past the last U-mode byte; in Sv39, it's (1 << 38).
// This is also the first "non-canonical" virtual address.
pub const USER_STACK_END: usize = 0x40_0000_0000;

pub const EPM_SIZE: usize = 0x100_000;
pub const UTM_SIZE: usize = 0x4_000;
pub const EDGE_BUFFER_SIZE: usize = 0x3_000;
pub const KERNEL_EPM_OFFSET: usize = 0x4_000;

// KERNEL_EPM_OFFSET + KERNEL_SIZE must be *smaller* than EPM_SIZE
