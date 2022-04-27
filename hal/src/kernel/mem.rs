pub use crate::sys::mem::{user_access_begin, user_access_end};

pub unsafe fn copy_from_user(kernel_mem: &mut [u8], user_mem: *const u8) {
    // TODO: maybe we need UserMemGuard or with_user_access()?
    user_access_begin();
    {
        let user_mem = core::slice::from_raw_parts(user_mem, kernel_mem.len());
        kernel_mem.copy_from_slice(user_mem);
    }
    user_access_end();
}

pub unsafe fn copy_to_user(kernel_mem: &[u8], user_mem: *mut u8) {
    user_access_begin();
    {
        let user_mem = core::slice::from_raw_parts_mut(user_mem, kernel_mem.len());
        user_mem.copy_from_slice(kernel_mem);
    }
    user_access_end();
}

pub unsafe fn strncpy_from_user(kernel_mem: &mut [u8], user_mem: *const u8) -> usize {
    user_access_begin();

    // naive implementation
    let (mut cur, mut len) = (user_mem, 0);
    while cur.read() != 0 && len < kernel_mem.len() {
        cur = cur.add(1);
        len += 1;
    }
    let effective_len = len;
    if len < kernel_mem.len() && cur.read() == 0 {
        len += 1;
    }

    {
        let user_mem = core::slice::from_raw_parts(user_mem, len);
        kernel_mem[0..len].copy_from_slice(user_mem);
    }

    user_access_end();

    effective_len
}

pub unsafe fn read_from_user<T>(user_mem: *const T) -> T
where
    T: Copy,
{
    user_access_begin();
    let result = user_mem.read();
    user_access_end();
    result
}
