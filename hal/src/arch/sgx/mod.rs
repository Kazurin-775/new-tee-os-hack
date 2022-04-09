pub unsafe fn initialize_edge_caller(utm_base: *mut u8) {
    crate::sys::edge::initialize_edge_caller(utm_base);
}
