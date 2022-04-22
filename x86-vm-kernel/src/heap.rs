use bootloader::{
    boot_info::{MemoryRegion, MemoryRegionKind},
    BootInfo,
};
use kmalloc::{Kmalloc, LockedLinkedListHeap};

#[global_allocator]
static HEAP: LockedLinkedListHeap = unsafe { LockedLinkedListHeap::uninit() };
const MIN_HEAP_SIZE: u64 = 4 << 20; // 4 MiB

pub fn init(boot_info: &BootInfo) {
    let mut heap_region: Option<&MemoryRegion> = None;
    for memory_region in boot_info.memory_regions.iter() {
        // TODO: the memory layout returned by UEFI firmware is quite complex,
        // use with caution
        if memory_region.kind == MemoryRegionKind::Usable
            && memory_region.end - memory_region.start >= MIN_HEAP_SIZE
        {
            if let Some(_) = heap_region {
                log::warn!("Ignoring extra memory region {:?}", memory_region);
            } else {
                heap_region = Some(memory_region);
            }
        }
    }

    let heap_region = heap_region.unwrap();
    log::debug!("Creating heap at {:?}", heap_region);
    unsafe {
        HEAP.init(
            (hal::cfg::KERNEL_MIRROR_BASE + heap_region.start as usize) as *mut _,
            (heap_region.end - heap_region.start) as usize,
        );
    }
}
