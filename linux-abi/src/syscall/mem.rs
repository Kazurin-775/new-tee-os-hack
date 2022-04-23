use hal::cfg::PAGE_SIZE;

use super::SyscallHandler;
use crate::Errno;

pub const SYSCALL_MMAP: SyscallHandler = SyscallHandler::Syscall6(syscall_mmap);
pub const SYSCALL_MUNMAP: SyscallHandler = SyscallHandler::Syscall2(syscall_munmap);

const MAP_SHARED: usize = 0x01;
const MAP_PRIVATE: usize = 0x02;
const MAP_ANONYMOUS: usize = 0x20;
const PROT_NONE: usize = 0x0;

unsafe fn syscall_mmap(
    start: usize,
    len: usize,
    prot: usize,
    flags: usize,
    fd: usize,
    off: usize,
) -> isize {
    // Check the arguments.
    if start != 0 {
        log::warn!("mmap: Placement mmap is not supported. Ignoring the address hint");
    }
    if (flags & MAP_ANONYMOUS) != MAP_ANONYMOUS {
        log::error!("mmap: File-backed mmap is not supported");
        return Errno::ENODEV.as_neg_isize();
    }
    if prot == PROT_NONE {
        log::error!("mmap: Guard pages (PROT_NONE) are not supported");
        return Errno::EINVAL.as_neg_isize();
    }
    if (flags & MAP_SHARED) == MAP_SHARED {
        log::warn!("mmap: Shared mmaps are not supported. Using a private mmap");
    } else if (flags & MAP_PRIVATE) != MAP_PRIVATE {
        log::error!("mmap: Neither of MAP_SHARED or MAP_PRIVATE is specified");
        return Errno::EINVAL.as_neg_isize();
    }

    // Align len to multiples of the page size.
    let len = (len + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

    let ptr = hal::task::current().lock().mm.map_anon(len);
    log::trace!(
        "mmap({:#X}, {:#X}, {:#X}, {:#X}, {}, {:#X}) = {:#X}",
        start,
        len,
        prot,
        flags,
        fd,
        off,
        ptr,
    );
    ptr as isize
}

unsafe fn syscall_munmap(start: usize, len: usize) -> isize {
    log::trace!("munmap({:#X}, {:#X})", start, len);
    if hal::task::current().lock().mm.unmap(start, len) {
        0
    } else {
        Errno::EINVAL.as_neg_isize()
    }
}
