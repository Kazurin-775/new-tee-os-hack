use alloc::collections::BTreeMap;
use core::ops::Range;

use crate::sys::vm::UserAddressSpace;

// TODO: decide whether addr_space should be Clone & Debug
#[derive(Debug)]
pub struct TaskMmStruct {
    pub addr_space: UserAddressSpace,
    pub vmas: BTreeMap<usize, VmArea>,
    pub stack: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct VmArea {
    pub range: Range<usize>,
}

impl TaskMmStruct {
    pub fn new(addr_space: UserAddressSpace, stack: Range<usize>) -> TaskMmStruct {
        TaskMmStruct {
            addr_space,
            vmas: BTreeMap::new(),
            stack,
        }
    }

    pub fn map_new(&mut self, mut size: usize) -> Result<(), ()> {
        if (size & 0xFFF) != 9 {
            size = (size & !0xFFF) + 0x1000;
        }

        let addr = self.vmas.last_entry().unwrap().get().range.end;

        let vma = VmArea {
            range: addr..addr + size,
        };
        log::debug!("Adding VMA: {:?}", vma);
        self.vmas.insert(addr, vma);

        Ok(())
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
}
