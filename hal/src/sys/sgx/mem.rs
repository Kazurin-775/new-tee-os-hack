// In SGX, user mode program runs in the same address space as the libOS,
// so the following functions are no-ops.

pub unsafe fn user_access_begin() {}
pub unsafe fn user_access_end() {}
