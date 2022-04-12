pub unsafe fn user_access_begin() {
    crate::arch::x86_vm::security::set_smap(false);
}

pub unsafe fn user_access_end() {
    crate::arch::x86_vm::security::set_smap(true);
}
