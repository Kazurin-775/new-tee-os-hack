use bootloader::{
    boot_info::{MemoryRegion, MemoryRegionKind},
    BootInfo,
};
use kmalloc::{Kmalloc, LockedLinkedListHeap};

#[global_allocator]
static HEAP: LockedLinkedListHeap = unsafe { LockedLinkedListHeap::uninit() };

pub fn find_usable_region(boot_info: &BootInfo) -> &MemoryRegion {
    let mut heap_region: Option<&MemoryRegion> = None;
    for memory_region in boot_info.memory_regions.iter() {
        if memory_region.kind == MemoryRegionKind::Usable {
            if let Some(_) = heap_region {
                hal::dbg_println!("Ignoring extra memory region {:?}", memory_region);
            } else {
                heap_region = Some(memory_region);
            }
        }
    }

    heap_region.unwrap()
}

pub fn init(start: usize, end: usize) {
    unsafe {
        HEAP.init(
            (hal::cfg::KERNEL_MIRROR_BASE + start) as *mut _,
            end - start,
        );
    }
}
