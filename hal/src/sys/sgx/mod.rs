pub mod edge;
pub mod mem;
pub mod task;
pub mod vm;

pub fn exit_enclave(retval: usize) {
    unsafe {
        crate::arch::sgx::ocall_exit(retval.try_into().unwrap());
    }
    unreachable!()
}
