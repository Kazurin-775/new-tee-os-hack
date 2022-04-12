#[global_allocator]
static ALLOCATOR: sgx_alloc::System = sgx_alloc::System;

#[alloc_error_handler]
fn default_handler(layout: core::alloc::Layout) -> ! {
    panic!("memory allocation of {} bytes failed", layout.size())
}
