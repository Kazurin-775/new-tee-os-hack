use alloc::collections::BTreeMap;
use core::ops::Range;

use crate::{kernel::vm::AddressSpace, sys::vm::UserAddressSpace};
#[cfg(feature = "multitasking")]
use crate::vm::ClonableAddressSpace;

// TODO: decide whether addr_space should be Clone & Debug
#[derive(Debug)]
pub struct TaskMmStruct {
    pub addr_space: UserAddressSpace,
    pub vmas: BTreeMap<usize, VmArea>,
    pub stack_zone: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct VmArea {
    pub range: Range<usize>,
}

impl TaskMmStruct {
    pub fn new(addr_space: UserAddressSpace, stack_zone: Range<usize>) -> TaskMmStruct {
        TaskMmStruct {
            addr_space,
            vmas: BTreeMap::new(),
            stack_zone,
        }
    }

    pub fn map_anon(&mut self, mut size: usize) -> usize {
        if (size & 0xFFF) != 0 {
            size = (size & !0xFFF) + 0x1000;
        }

        let addr = self.vmas.last_entry().unwrap().get().range.end;

        let vma = VmArea {
            range: addr..addr + size,
        };
        log::debug!("Adding VMA: {:?}", vma);
        self.addr_space.alloc_map(vma.range.clone());
        self.vmas.insert(addr, vma);

        addr
    }

    pub fn unmap(&mut self, addr: usize, mut len: usize) -> bool {
        if (len & 0xFFF) != 0 {
            len = (len & !0xFFF) + 0x1000;
        }

        if let Some(vma) = self.vmas.get(&addr) {
            if len == vma.range.len() {
                log::debug!("Removing VMA: {:?}", vma);
                self.vmas.remove(&addr);
                // TODO: deallocate memory
                true
            } else {
                log::error!("VMA {:?}'s length is not equal to {}", vma, len);
                false
            }
        } else {
            log::error!("VMA {:#X} does not exist", addr);
            false
        }
    }

    pub fn first_vma_after(&self, addr: usize) -> Option<&VmArea> {
        if let Some((_start, vma)) = self
            .vmas
            .range(..addr)
            .next_back()
            .filter(|(_start, vma)| addr < vma.range.end)
        {
            Some(vma)
        } else if let Some((_start, vma)) = self.vmas.range(addr..).next() {
            Some(vma)
        } else {
            None
        }
    }

    #[cfg(feature = "multitasking")]
    pub fn duplicate(&self) -> TaskMmStruct {
        let mut new_addr_space = self.addr_space.create_bare();

        // TODO: walk the page table instead of VMAs
        for (_start, vma) in &self.vmas {
            new_addr_space.copy_from_current(vma.range.clone());
        }
        new_addr_space.copy_from_current(self.stack_zone.clone());

        TaskMmStruct {
            addr_space: new_addr_space,
            vmas: self.vmas.clone(),
            stack_zone: self.stack_zone.clone(),
        }
    }
}
